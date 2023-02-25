use std::{collections::HashMap, fmt::Write, fs, num::ParseIntError, str::FromStr};

use bytes::Bytes;
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::{utils::{MRError, funcs::i_to_m}, file_struct::ntfs::Ntfs};

use super::NtfsModule;
use memchr::memmem;
pub fn hex_to_vec_u8(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn vs_contains_sub(haystack: &Vec<u8>, needle: &Vec<u8>) -> Option<usize> {
    memmem::find(haystack, needle)
}

fn ref_file(ntfs: &mut Ntfs, offset: u64, drive: &str) -> ColoredString {
    let mft = ntfs.search_addr_belong(offset);
    if let Some(mft) = mft {
        let filename = match mft.fullpath() {
            Some(s) => s,
            None => {
                return format!("lcn:{}", offset / ntfs.get_cluster_size()).bright_red();
            }
        };
        return format!("{}\\{}",drive, filename).bright_blue();
    }
    format!("lcn:{}", offset / ntfs.get_cluster_size()).bright_red()
}


#[derive(PartialEq)]
enum MatchType {
    Equal,
    Regex,
    RegexUtf16,
}

fn vec_u8_to_utf16string(bytes: &Vec<u8>) -> String {
    let title: Vec<u16> = bytes
        .chunks_exact(2)
        .into_iter()
        .map(|a| u16::from_ne_bytes([a[0], a[1]]))
        .collect();
    let title = title.as_slice();
    let title = String::from_utf16_lossy(title);
    title
}

fn sec_to_s(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
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
        let default_to_file = "false".to_string();
        let to_file = match args.get("ref_file") {
            Some(s) => s,
            None => &default_to_file,
        };
        let bool_to_file;
        if to_file.eq("true") {
            bool_to_file = true;
            let mft_mft = self.ntfs.get_datas_of_mft();
            let mut total = 0;
            for mft in mft_mft {
                total += mft.get_datasize();
            }
            
            let pb = ProgressBar::new(total);
            let mut save_offset = 0;
            println!("Loading mft....");
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}), {eta}")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{}s", sec_to_s(state.eta().as_secs())).unwrap())
                .progress_chars("#>-"));
            self.ntfs.cache_mfts(|offset| {
                pb.set_position(offset);
                save_offset = offset;
            });
            pb.finish();
            println!("Loaded {} mft", save_offset / self.ntfs.get_mft_size() as u64);
        } else {
            bool_to_file = false;
        }

        let target: Vec<u8>;
        let mut regex_bytes_pattern = None::<regex::bytes::Regex>;
        let mut regex_pattern = None::<regex::Regex>;
        if encode.eq("hex") {
            target = match hex_to_vec_u8(&to_search) {
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
        } else if encode.eq("regex") {
            regex_pattern = Some(regex::Regex::from_str(to_search).unwrap());
            target = Vec::new();
            match_type = MatchType::Regex;
        } else if encode.eq("regex_bytes") {
            regex_bytes_pattern = Some(regex::bytes::Regex::from_str(to_search).unwrap());
            target = Vec::new();
            match_type = MatchType::Regex;
        } else if encode.eq("regex_utf16") {
            regex_pattern = Some(regex::Regex::from_str(to_search).unwrap());
            target = Vec::new();
            match_type = MatchType::RegexUtf16;
        } else {
            return Err(MRError::new(
                "Not support type: hex, base64, file, string, regex, regex_bytes, regex_utf16",
            ));
        }
        let read_size = self.ntfs.get_cluster_size() as usize * 0x1000;
        let all_zero_vec = Vec::<u8>::with_capacity(read_size + target.len());
        let all_zero_hash = md5::compute(&all_zero_vec);
        //let target = Bytes::from(target);
        let totals = self.ntfs.get_sector_bytes_num() * self.ntfs.get_sector_num();
        let pb = ProgressBar::new(totals);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}), {eta}")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{}s", sec_to_s(state.eta().as_secs())).unwrap())
                .progress_chars("#>-"));
        let pb2 = &pb;
        let mut count = 0;
        let ntfs = &self.ntfs;
        let drive = self.file.as_str();
        self.ntfs
            .iter_diy_block(read_size, target.len(), 3, move |index, progress, bs| {
                if match_type.eq(&MatchType::Equal) {
                    pb2.set_position(progress);
                    let size = vs_contains_sub(&bs, &target);
                    if size.is_some() {
                        let sub = String::from_utf8_lossy(
                            &bs[size.unwrap()..size.unwrap() + target.len()],
                        )
                        .to_string();
                        let s = format!("{} {:?} -> ref_file: {}", progress + size.unwrap() as u64, sub, 
                            ref_file(i_to_m(ntfs), progress + size.unwrap() as u64, drive));
                        pb2.println(s);
                    }
                } else if match_type.eq(&MatchType::Regex) {
                    pb2.set_position(progress);
                    if let Some(rp) = &regex_pattern {
                        let s2 = String::from_utf8_lossy(&bs);

                        for mt in rp.find_iter(&s2) {
                            let s = format!(
                                "utf-8: {} {:?} -> ref_file: {}",
                                progress + mt.start() as u64,
                                mt.as_str(),
                                ref_file(i_to_m(ntfs), progress + mt.start() as u64, drive)
                            );
                            pb2.println(s);
                        }
                    }

                    if let Some(rbp) = &regex_bytes_pattern {
                        for mt in rbp.find_iter(&bs) {
                            let s = format!(
                                "utf-8: {} {:?} -> ref_file: {}",
                                progress + mt.start() as u64,
                                mt.as_bytes(),
                                ref_file(i_to_m(ntfs), progress + mt.start() as u64, drive)
                            );
                            pb2.println(s);
                        }
                    }
                } else if match_type.eq(&MatchType::RegexUtf16) {
                    pb2.set_position(progress);
                    let s1 = vec_u8_to_utf16string(&bs);
                    if let Some(rp) = &regex_pattern {
                        for mt in rp.find_iter(&s1) {
                            let a = mt.start() % 0x400;
                            let s = format!(
                                "utf-16: {} {:?} -> ref_file: {}",
                                progress + mt.start() as u64,
                                mt.as_str(),
                                ref_file(i_to_m(ntfs), progress + mt.start() as u64, drive)
                            );
                            pb2.println(s);
                        }
                    }
                }
                return false;
            });
        pb.finish();

        Ok(())
    }
}
