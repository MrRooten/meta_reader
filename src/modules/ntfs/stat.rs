use std::collections::HashMap;

use crate::{utils::MRError, file_struct::ntfs::MFTEntry};

use super::NtfsModule;

impl NtfsModule {
    pub fn stat(&mut self, args: HashMap<String,String>) -> Result<(),MRError> {
        let path = args.get("path");
        let index = args.get("index");

        if path.is_none() && index.is_none() {
            return Err(MRError::new("must set path=${path} or index=${index}"));
        }

        let mft = if path.is_some() {
            match self.ntfs.get_mft_by_path(path.unwrap()) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            }

        } else {
            let index = index.unwrap().parse::<u64>().unwrap();
            match self.ntfs.get_mft_entry_by_index(index) {
                Some(o) => o,
                None => {
                    return Err(MRError::new("not found index"));
                }
            }
        };
        
        println!("filename: {:?}",mft.filename());
        println!("\tindex: {:?}", mft.get_index());
        println!("\tfullpath: {:?}", mft.fullpath());
        println!("\tcreation: {:?}", mft.get_creation_time());
        println!("\taccess: {:?}", mft.get_access_time());
        println!("\tmodify: {:?}", mft.get_change_time());
        println!("\tcreation real(from filename): {:?}", mft.filename_creation_time());
        println!("\tstream list: {:?}", mft.get_streams_list());
        Ok(())
    }
}