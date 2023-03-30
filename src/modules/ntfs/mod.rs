#![allow(unused)]
use std::collections::HashMap;

use crate::{file_struct::ntfs::Ntfs, utils::MRError};
pub mod stat;
pub mod deleted_files;
pub mod search_disk;
pub mod search_files_content;
pub mod dump_usn;
pub struct NtfsModule {
    ntfs    : Ntfs,
    file    : String,
    func    : HashMap<String,Box<dyn Fn(HashMap<String,String>)>>
}

impl NtfsModule {
    pub fn new(file: &str) -> Result<NtfsModule, MRError> {
        let ntfs = match Ntfs::open(file) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        Ok(Self {
            ntfs: ntfs,
            file: file.to_string(),
            func: Default::default(),
        })
    }
}