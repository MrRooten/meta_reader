use std::{
    collections::HashMap, fmt::Write, fs, num::ParseIntError, ops::Range, rc::Rc, str::FromStr,
};

use bytes::{Buf, Bytes};
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::{
    file_struct::ntfs::{FileTime, MFTEntry, Ntfs, USNChangeJournalEntry},
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

fn check(mfts: &[(Range<usize>, u64)]) -> Vec<Range<usize>> {
    let mut res = vec![];
    let mut i = 0;

    for j in 0..mfts.len()-1 {
        if mfts[i].0.end >= mfts[j].0.start {
            continue;
        } else {
            res.push(Range {
                start: mfts[i].0.start,
                end: mfts[j].0.end
            });
            i = j;
        }
    }

    res
}



fn match_filetime(filetime: Bytes) -> bool {
    let update_time = FileTime::parse_from_u64((&filetime[0..8]).get_u64());
    let scond = update_time.to_native_date();
    if scond.is_none() {
        return false;
    }
    true
}

fn match_usn_struct(bs: &Bytes) -> Option<usize> {
    let entry_size = (&bs[0..4]).get_u32_le();
    if entry_size > 78 + 255 || entry_size % 4 != 0 {
        return None
    }
    let majar_ver = (&bs[4..6]).get_u16_le();
    let minjar_ver = (&bs[6..8]).get_u16_le();

    match majar_ver {
        3 => {
            if entry_size <= 60 {
                return None;
            }
        },
        2 => {
            if entry_size <= 78 {
                return None
            }
        },
        _ => {
            return None
        }
    };
    

    if majar_ver != 3 && majar_ver != 2 {
        return None
    }

    if minjar_ver != 0 {
        return None
    }

    let filetime = bs.slice(32..40);
    if !match_filetime(filetime) {
        return None
    }

    let name_offset = match majar_ver {
        3 => {
            let v = (&bs[74..76]).get_u16_le();
            if v != 76 {
                return None;
            }

            v
        },
        2 => {
            let v = (&bs[58..60]).get_u16_le();
            if v != 60 {
                return None;
            }

            v
        },
        _ => {
            return None
        }
    };

    let name_size = match majar_ver {
        3 => {
            (&bs[72..74]).get_u16_le()
        },
        2 => {
            (&bs[56..58]).get_u16_le()
            
        },
        _ => {
            return None
        }
    };

    let v = match majar_ver {
        3 => {
            let v = 78 + name_size as usize;
            if v > bs.len() {
                return None
            }
            v
        },
        2 => {
            let v = 60 + name_size as usize;
            if v > bs.len() {
                return None
            }
            v
        },
        _ => {
            return None;
        }
    };



    let name = match majar_ver {
        3 => {
            vec_u8_to_utf16string(bs.get(78..78+name_size as usize).unwrap())
        },
        2 => {
            vec_u8_to_utf16string(bs.get(60..60+name_size as usize).unwrap())
        },
        _ => {
            return None;
        }
    };

    Some(entry_size as usize)
}



impl NtfsModule {
    pub fn search_usn(&mut self, args: HashMap<String, String>) -> Result<(), MRError> {
        let match_type: MatchType;

        // let mut _mfts = vec![];

        let mft_mft = self.ntfs.get_datas_of_mft();
        let mut total = 0;
        for mft in &*mft_mft.borrow() {
            total += mft.get_datasize();
        }

        // let pb = ProgressBar::new(total);
        // let mut save_offset = 0;
        // println!("Loading mft....");
        // pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}), {eta}")
        //         .unwrap()
        //         .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{}s", sec_to_s(state.eta().as_secs())).unwrap())
        //         .progress_chars("#>-"));
        // _mfts = cache_mfts(&mut self.ntfs, |offset, entry| {
        //     pb.set_position(offset);
        //     let value = entry.get_data_value();
        //     if value.is_none() {
        //         return false;
        //     }

        //     if let Some(datas) = value {
        //         if datas.get_datas().is_empty() {
        //             return false;
        //         }
        //     }
        //     save_offset = offset;
        //     true
        // });

        let bitmap = self.ntfs.get_bitmap().unwrap();


        let ranges = bitmap.generate_unalloc()?;
        // let (mut mfts, _totals) = generate_range(&_mfts, 0x1000, totals as usize);
        
        // mfts.sort_by(|a, b| {
        //     a.start.cmp(&b.start)
        // });


        // pb.finish();
        // println!(
        //     "Loaded {} Master Entries",
        //     save_offset / self.ntfs.get_mft_size() as u64
        // );


        let mut regex_bytes_pattern = None::<regex::bytes::Regex>;
        let mut regex_pattern = None::<regex::Regex>;

        let read_size = self.ntfs.get_cluster_size() * 0x100;
        //let target = Bytes::from(target);
        
        let pb = ProgressBar::new(ranges.1 as u64 * self.ntfs.get_cluster_size());
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}), {eta}")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{}s", sec_to_s(state.eta().as_secs())).unwrap())
                .progress_chars("#>-"));
        let pb2 = &pb;
        let mut count = 0;
        let ntfs = &self.ntfs;
        let drive = self.file.as_str();
        self.ntfs.iter_sp_block(
            &ranges.0,
            read_size as usize,
            0,
            3,
            move |index, progress, bs, read_size| {
                pb2.set_position(read_size);
                let bs = Bytes::from(bs);
                let mut offset = 0;
                while offset < bs.len() {
                    let t = bs.slice(offset..bs.len());
                    if t.len() < 58 {
                        return false
                    }
                    match match_usn_struct(&t) {
                        Some(s) => {                    
                            offset += s;
                            
                        },
                        None => {
                            offset += 0x10;
                            continue;
                        }
                    }

                    let entry = match USNChangeJournalEntry::parse(t) {
                        Ok(s) => s,
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    };
                    println!("{:?}  {:?}", entry.get_time_string(), entry);
                

                }
                false
            },
        );
        pb.finish();

        Ok(())
    }
}
