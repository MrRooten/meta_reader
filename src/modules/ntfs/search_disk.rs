use std::{collections::HashMap, num::ParseIntError, fmt::Write, fs};

use bytes::Bytes;
use indicatif::{ProgressBar, ProgressStyle, ProgressState};

use crate::utils::MRError;

use super::NtfsModule;

pub fn hex_to_vec_u8(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn vs_contains_sub(haystack: &Vec<u8>, needle: &Vec<u8>) -> Option<usize> {
    let m1 = md5::compute(needle);
    haystack.windows(needle.len()).position(|window| window.eq(needle))
}

impl NtfsModule {
    pub fn search_disk(&mut self, args: HashMap<String, String>) -> Result<(), MRError> {
        let default_encode = "hex".to_string();
        let encode = match args.get("encode") {
            Some(s) => s,
            None => &default_encode,
        };

        let to_search = match args.get("to_search") {
            Some(s) => s,
            None => &default_encode,
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
        } else if encode.eq("string") {
            target = to_search.as_bytes().to_vec();
        }
        else {
            return Err(MRError::new("Must post the to_search={}"));
        }
        let read_size = self.ntfs.get_cluster_size() as usize * 0x100;
        let all_zero_vec = Vec::<u8>::with_capacity(read_size + target.len());
        let all_zero_hash = md5::compute(&all_zero_vec);
        //let target = Bytes::from(target);
        let totals = self.ntfs.get_sector_bytes_num() * self.ntfs.get_sector_num();
        let pb = ProgressBar::new(totals);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .progress_chars("#>-"));
        

        self.ntfs.iter_diy_block(
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
                    let s = format!("\r{} {:?}", progress + size.unwrap() as u64, sub);
                    pb.println(s);
                }
                pb.set_position(progress);
                return false;
            },
        );
        pb.finish();

        Ok(())
    }
}
