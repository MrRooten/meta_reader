use std::{collections::HashMap, num::ParseIntError, fmt::Write, fs, str::FromStr};

use indicatif::{ProgressBar, ProgressStyle, ProgressState};
use memchr::memmem;

use crate::{utils::MRError, modules::Hanlder, file_struct::ext4::Ext4};

use super::Ext4Module;

pub fn hex_to_vec_u8(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn vs_contains_sub(haystack: &Vec<u8>, needle: &Vec<u8>) -> Option<usize> {
    memmem::find(haystack, needle)
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
#[derive(PartialEq)]
enum MatchType {
    Equal,
    Regex,
    RegexUtf16,
}

impl Ext4Module {
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
            },
        };

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
            return Err(MRError::new("Not support type: hex, base64, file, string, regex, regex_bytes, regex_utf16"));
        }
        let read_size = self.ext4.get_block_size() as usize;
        let all_zero_vec = Vec::<u8>::with_capacity(read_size + target.len());
        let all_zero_hash = md5::compute(&all_zero_vec);
        //let target = Bytes::from(target);
        let totals = self.ext4.get_block_size() as u64 * self.ext4.get_blocks_count();
        let pb = ProgressBar::new(totals);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .progress_chars("#>-"));
        

        self.ext4.iter_diy_block(
            read_size,
            target.len(),
            3,
            |index, progress, bs| {
                if match_type.eq(&MatchType::Equal) {
                    pb.set_position(progress);
                    let size = vs_contains_sub(&bs, &target);
                    if size.is_some() {
                        let sub = String::from_utf8_lossy(
                            &bs[size.unwrap()..size.unwrap() + target.len()],
                        ).to_string();
                        let s = format!("{} {:?}", progress + size.unwrap() as u64, sub);
                        pb.println(s);
                    }

                } else if match_type.eq(&MatchType::Regex) {
                    pb.set_position(progress);
                    if let Some(rp) = &regex_pattern {
                        let s2 = String::from_utf8_lossy(&bs);

                        for mt in rp.find_iter(&s2) {
                            let s = format!(
                                "utf-8: {} {:?}",
                                progress + mt.start() as u64,
                                mt.as_str()
                            );
                            pb.println(s);
                        }
                    }

                    if let Some(rbp) = &regex_bytes_pattern {
                        for mt in rbp.find_iter(&bs) {
                            let s = format!(
                                "bytes: {} {:?} {}",
                                progress + mt.start() as u64,
                                mt.as_bytes(),
                                String::from_utf8_lossy(mt.as_bytes())
                            );
                            pb.println(s);
                        }
                    }
                } else if match_type.eq(&MatchType::RegexUtf16) {
                    pb.set_position(progress);
                    let s1 = vec_u8_to_utf16string(&bs);
                    if let Some(rp) = &regex_pattern {
                        for mt in rp.find_iter(&s1) {
                            let a = mt.start() % 0x400;
                            let s = format!(
                                "utf-16: {} {:?} {:?}",
                                progress + mt.start() as u64,
                                mt.as_str(),
                                String::from_utf8_lossy(&bs[mt.start()-a..mt.start()-a+30].to_vec())
                            );
                            pb.println(s);
                        }
                    }
                }
                return false;
            }
        );
        pb.finish();

        Ok(())
    }
}

struct Ext4SearchDisk {
    ext4    : Ext4
}

impl Hanlder for Ext4SearchDisk {
    fn run(&self, args: HashMap<String, String>) -> Result<(), MRError> {
        let default_encode = "hex".to_string();
        let encode = match args.get("encode") {
            Some(s) => s,
            None => &default_encode,
        };

        let to_search = match args.get("to_search") {
            Some(s) => s,
            None => {
                return Err(MRError::new("hex=${default:hex,file,base64},to_search=${pattern}"));
            },
        };

        let target: Vec<u8>;
        if encode.eq("hex") {
            target = match hex_to_vec_u8(&to_search) {
                Ok(o) => o,
                Err(_) => {
                    return Err(MRError::new("Not a valid hex"));
                }
            };
        } else if encode.eq("base64") {
            target = match base64::decode(to_search) {
                Ok(o) => o,
                Err(_) => {
                    return Err(MRError::new("Not a valid base64"));
                }
            };
        } else if encode.eq("file") {
            target = fs::read(to_search).unwrap();
        } else if encode.eq("regex") {
            target = Vec::new();
        }
        else {
            return Err(MRError::new("Must post the to_search={}"));
        }

        let read_size = self.ext4.get_block_size() as usize * 0x100;
        let all_zero_vec = Vec::<u8>::with_capacity(read_size + target.len());
        let all_zero_hash = md5::compute(&all_zero_vec);
        //let target = Bytes::from(target);
        let totals = self.ext4.get_block_size() as u64 * self.ext4.get_blocks_count();
        let pb = ProgressBar::new(totals);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .progress_chars("#>-"));
        

        self.ext4.iter_diy_block(
            read_size,
            target.len(),
            3,
            |index, progress, bs| {
                // let hash = md5::compute(&bs);
                // if hash.eq(&all_zero_hash) {
                //     return false;
                // }
                let size = vs_contains_sub(&bs, &target);
                if size.is_some() {
                    let sub = String::from_utf8_lossy(&bs[size.unwrap()..size.unwrap()+target.len()]).to_string();
                    let s = format!("\r{} {:?}", index, sub);
                    pb.println(s);
                }
                pb.set_position(progress);
                return false;
            },
        );
        pb.finish();

        Ok(())
    }

    fn name(&self) -> &str {
        "search_disk"
    }

    fn help(&self) -> &str {
        "encode=${encode_type:hex,file,base64},to_search=${pattern}"
    }
}