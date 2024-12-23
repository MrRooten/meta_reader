use std::{collections::HashMap, fmt::Write};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::{modules::ntfs::{search_disk::sec_to_s, MatchType}, utils::MRError};

use super::NtfsModule;

impl NtfsModule {
    pub fn search_files_content(&mut self, args: HashMap<String, String>) -> Result<(), MRError> {
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
        let mft_mft = self.ntfs.get_datas_of_mft();
        let mut total = 0;
        for mft in &*mft_mft.borrow() {
            total += mft.get_datasize();
        }

        self.ntfs.iter_mft(|index, mft_entry, is_deleted, ntfs| {

        });
        unimplemented!()
    }
}
