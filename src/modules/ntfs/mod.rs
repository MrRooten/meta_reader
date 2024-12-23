#![allow(unused)]
use std::{collections::HashMap, path::{Path, self}};

use crate::{file_struct::ntfs::Ntfs, utils::MRError};
pub mod stat;
pub mod deleted_files;
pub mod search_disk;
pub mod search_files_content;
pub mod dump_usn;
pub mod search_usn;

type NtfsFunc = Box<dyn Fn(HashMap<String,String>)>;

pub struct NtfsModule {
    ntfs    : Ntfs,
    file    : String,
    func    : HashMap<String,NtfsFunc>
}

impl NtfsModule {
    pub fn new<P>(file: P) -> Result<NtfsModule, MRError> 
    where P: AsRef<path::Path> + ToString {
        let s = file.to_string();
        let ntfs = match Ntfs::open(file) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        Ok(Self {
            ntfs,
            file: s,
            func: Default::default(),
        })
    }
}

#[derive(PartialEq)]
pub enum MatchType {
    Equal,
    Regex,
    RegexUtf16,
}