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
            if res.contains_attr(0x30) == false {
                return ;
            }
            
            
                //if parent_index as i64 == res.get_parent_index() {
                  
            let mft = ntfs.get_mft_entry_by_index(index);
            if let Some(s) = mft {
                if s.filename().is_some() && res.filename().is_some() {
                    let name = s.filename().unwrap();
                    let name2 = res.filename().unwrap();
                    if name.eq(&name2) == false {
                        println!("{} {} {}", index, name, name2);
                    }
                }
            }

            if index % 9333 == 0 {
                println!("{}", index);
            }
            
        });
        Ok(())
    }
}