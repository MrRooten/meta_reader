use std::collections::HashMap;

use crate::{utils::MRError};

use super::NtfsModule;

impl NtfsModule {
    pub fn deleted_files(&mut self, args: HashMap<String,String>) -> Result<(),MRError> {
        let path = match args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_path}"));
            }
        };
        let mft = match self.ntfs.get_mft_by_path(path) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let parent_index = mft.get_index();
        self.ntfs.iter_mft(|index, res, is_deleted, ntfs| {
            let res = match res {
                Ok(o) => o,
                Err(e) => {
                    return ;
                }
            };
            if !res.contains_attr(0x30) {
                return ;
            }
            
            
                //if parent_index as i64 == res.get_parent_index() {
            if (res.get_flags() == 0 || res.get_flags() == 2) && res.get_parent_index() == parent_index as i64 {
                println!("{} {:?} {}", res.get_index(), res.filename(), res.get_flags());
            }
            
        });
        Ok(())
    }
}