use std::{
    collections::HashMap, fmt::Write, fs, num::ParseIntError, ops::Range, rc::Rc, str::FromStr,
};

use bytes::Bytes;
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::{
    file_struct::ntfs::{MFTEntry, Ntfs},
    utils::MRError,
};

use super::{MatchType, NtfsModule};
use memchr::memmem;
pub fn hex_to_vec_u8(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn vs_contains_sub(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memmem::find(haystack, needle)
}

fn ref_file(
    mfts: &[(Range<usize>, u64)],
    ntfs: &Ntfs,
    offset: u64,
    drive: &str,
    is_ref_file: bool,
) -> ColoredString {
    if !is_ref_file {
        return format!("lcn:{}", offset / ntfs.get_cluster_size()).bright_red();
    }
    let mft = search_addr_belong(mfts, offset);

    if let Some(mft) = mft {
        let mft = match ntfs.get_mft_entry_by_index(mft) {
            Some(s) => s,
            None => {
                return format!("lcn:{}", offset / ntfs.get_cluster_size()).bright_red();
            }
        };
        let filename = match mft.fullpath() {
            Some(s) => s,
            None => {
                return format!("lcn:{}", offset / ntfs.get_cluster_size()).bright_red();
            }
        };
        return format!("{}\\{}", drive, filename).bright_blue();
    }
    format!("lcn:{}", offset / ntfs.get_cluster_size()).bright_red()
}

fn vec_u8_to_utf16string(bytes: &[u8]) -> String {
    let title: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|a| u16::from_ne_bytes([a[0], a[1]]))
        .collect();
    let title = title.as_slice();

    String::from_utf16_lossy(title)
}

pub fn sec_to_s(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

fn cache_mfts<F>(ntfs: &mut Ntfs, mut filter: F) -> Vec<(Range<usize>, u64)>
where
    F: FnMut(u64, &MFTEntry) -> bool,
{
    let mut result = Vec::new();
    let mut cache = Vec::new();
    ntfs.iter_mft(|index, entry, is_deleted, ntfs| {
        let entry = match entry {
            Ok(o) => o,
            Err(_) => {
                return;
            }
        };
        let flag = filter(entry.get_index() * ntfs.get_mft_size() as u64, &entry);
        if flag {
            let entry = Rc::new(entry);
            let data_value = entry.get_data_value();
            if let Some(value) = data_value {
                let datas = value.get_datas();
                for data in datas {
                    let v = (
                        Range {
                            start: data.get_start_addr() as usize,
                            end: data.get_start_addr() as usize + data.get_datasize() as usize,
                        },
                        entry.get_index(),
                    );
                    cache.push(v);
                }
            }
        }
    });
    cache.sort_by(|k, v| k.0.start.cmp(&v.0.start));
    result.extend(cache);
    result
}

pub fn search_addr_belong(mfts: &[(Range<usize>, u64)], addr: u64) -> Option<u64> {
    let addr = addr as usize;

    let mut start = 0;
    let mut end = mfts.len();
    let mut middle = (start + end) / 2;
    let mut mft = &mfts[middle];
    while start <= end {
        if addr > mft.0.end {
            start = middle + 1;
            middle = (start + end) / 2;
            mft = &mfts[middle];
        } else if addr < mft.0.start {
            end = middle - 1;
            middle = (start + end) / 2;
            mft = &mfts[middle];
        } else if (start == end && (addr > mft.0.start && addr < mft.0.end))
            || (addr >= mft.0.start && addr <= mft.0.end)
        {
            return Some(mft.1);
        } else {
            return None;
        }
    }
    None
}
impl NtfsModule {
    pub fn search_disk(&mut self, args: HashMap<String, String>) -> Result<(), MRError> {
        let match_type: MatchType;
        let default_encode = "string".to_string();
        let encode = match args.get("encode") {
            Some(s) => s,
            None => &default_encode,
        };

        let to_search = match args.get("to_search") {
            Some(s) => s,
            None => {
                return Err(MRError::new("search_disk encode=${default:hex,base64,file,string,regex,regex_bytes,regex_utf16},to_search=${value}"));
            }
        };
        let default_to_file = "true".to_string();
        let to_file = match args.get("ref_file") {
            Some(s) => s,
            None => &default_to_file,
        };
        let bool_to_file;
        let mut _mfts = vec![];
        if to_file.eq("true") {
            bool_to_file = true;
            let mft_mft = self.ntfs.get_datas_of_mft();
            let mut total = 0;
            for mft in &*mft_mft.borrow() {
                total += mft.get_datasize();
            }

            let pb = ProgressBar::new(total);
            let mut save_offset = 0;
            println!("Loading mft....");
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}), {eta}")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{}s", sec_to_s(state.eta().as_secs())).unwrap())
                .progress_chars("#>-"));
            _mfts = cache_mfts(&mut self.ntfs, |offset, entry| {
                // pb.set_position(offset);
                let value = entry.get_data_value();
                if value.is_none() {
                    return false;
                }

                if let Some(datas) = value {
                    if datas.get_datas().is_empty() {
                        return false;
                    }
                }
                save_offset = offset;
                true
            });
            pb.finish();
            println!(
                "Loaded {} Master Entries",
                save_offset / self.ntfs.get_mft_size() as u64
            );
        } else {
            bool_to_file = false;
        }

        let target: Vec<u8>;
        let mut regex_bytes_pattern = None::<regex::bytes::Regex>;
        let mut regex_pattern = None::<regex::Regex>;

        if encode.eq("hex") {
            target = match hex_to_vec_u8(to_search) {
                Ok(o) => o,
                Err(_) => {
                    return Err(MRError::new("Not a valid hex"));
                }
            };
            match_type = MatchType::Equal;
        } else if encode.eq("base64") {
            target = match base64::decode(to_search) {
                Ok(o) => o,
                Err(_) => {
                    return Err(MRError::new("Not a valid base64"));
                }
            };
            match_type = MatchType::Equal;
        } else if encode.eq("file") {
            target = fs::read(to_search).unwrap();
            match_type = MatchType::Equal;
        } else if encode.eq("string") {
            target = to_search.as_bytes().to_vec();
            match_type = MatchType::Equal;
        } else if encode.eq("u16string") {
            let mut v: Vec<u16> = to_search.encode_utf16().collect();
            target = unsafe { v.align_to::<u8>().1.to_vec() };
            match_type = MatchType::Equal;
        } else if encode.eq("regex") {
            regex_bytes_pattern = Some(regex::bytes::Regex::from_str(to_search).unwrap());
            target = Vec::new();
            match_type = MatchType::Regex;
        } else {
            return Err(MRError::new(
                "Not support type: hex, base64, file, string, regex, regex_bytes, regex_utf16, u16string",
            ));
        }
        let read_size = self.ntfs.get_cluster_size() as usize * 0x1000;
        let all_zero_vec = Vec::<u8>::with_capacity(read_size + target.len());
        let all_zero_hash = md5::compute(all_zero_vec);
        //let target = Bytes::from(target);
        let totals = self.ntfs.get_sector_bytes_num() * self.ntfs.get_sector_num();
        // let pb = ProgressBar::new(totals);
        // pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}), {eta}")
        //         .unwrap()
        //         .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{}s", sec_to_s(state.eta().as_secs())).unwrap())
        //         .progress_chars("#>-"));
        // let pb2 = &pb;
        let mut count = 0;
        let ntfs = &self.ntfs;
        let drive = self.file.as_str();
        self.ntfs
            .iter_diy_block(read_size, target.len(), 3, move |index, progress, bs| {
                if match_type.eq(&MatchType::Equal) {
                    // pb2.set_position(progress);
                    let size = vs_contains_sub(&bs, &target);
                    if size.is_some() {
                        let sub = String::from_utf8_lossy(
                            &bs[size.unwrap()..size.unwrap() + target.len()],
                        )
                        .to_string();
                        let s = format!(
                            "{} {:?} -> ref_file: {}",
                            progress + size.unwrap() as u64,
                            sub,
                            ref_file(
                                &_mfts,
                                ntfs,
                                progress + size.unwrap() as u64,
                                drive,
                                bool_to_file
                            )
                        );
                        println!("{}" ,s);
                        // pb2.println(s);
                    }
                } else if match_type.eq(&MatchType::Regex) {
                    // pb2.set_position(progress);

                    if let Some(rbp) = &regex_bytes_pattern {
                        for mt in rbp.find_iter(&bs) {
                            let rs = String::from_utf8_lossy(mt.as_bytes());
                            let s = format!(
                                "{} {:?} -> ref_file: {}",
                                progress + mt.start() as u64,
                                rs,
                                ref_file(
                                    &_mfts,
                                    ntfs,
                                    progress + mt.start() as u64,
                                    drive,
                                    bool_to_file
                                )
                            );
                            println!("{}" ,s);
                            // pb2.println(s);
                        }
                    }
                }
                false
            });
        // pb.finish();

        Ok(())
    }
}
