#![allow(unused)]
use std::collections::HashMap;

use crate::{file_struct::ext4::Ext4, utils::MRError};

pub mod list_deleted_files;
pub mod list_files;
pub mod journal_recover_file;
pub mod list_journal_recoverable;
pub mod read_file;
pub mod list_recoverable_inodes;
pub mod search_deleted_files;
pub mod search_recoverable_files;
pub mod search_disk;
pub struct Ext4Module {
    ext4    : Ext4,
    file    : String,
    func    : HashMap<String,Box<dyn Fn(HashMap<String,String>)>>
}

impl Ext4Module {
    pub fn new(file: &str) -> Result<Ext4Module, MRError> {
        let ext4 = match Ext4::open(file) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            ext4: ext4,
            file: file.to_string(),
            func: Default::default(),
        })
    }
}

