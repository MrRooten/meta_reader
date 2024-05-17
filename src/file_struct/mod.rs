#![allow(unused)]
#![allow(non_camel_case_types)]

use bytes::Bytes;
use chrono::NaiveDateTime;

use crate::utils::MRError;
pub mod elf;
pub mod pe;
pub mod ext4;
pub mod ntfs;
pub mod xfs;
pub mod windows;
pub mod bitlocker;
pub mod vmdk;

pub trait File {
    fn read(&self, start: usize, size: usize) -> Result<Bytes, MRError>;

    fn get_size(&self) -> Result<usize, MRError>;

    fn get_owner(&self) -> Result<String, MRError>;

    fn get_mtime(&self) -> Result<NaiveDateTime, MRError>;

    fn get_ctime(&self) -> Result<NaiveDateTime, MRError>;

    fn get_atime(&self) -> Result<NaiveDateTime, MRError>;
}

pub trait FileSystem {
    fn list_files(&self, path: &str) -> Result<Vec<Box<dyn File>>, MRError>;

    fn open_file(&self, path: &str) -> Result<Box<dyn File>, MRError>;

    fn copy(&self, fs_path: &str, local_path: &str) -> Result<(), MRError>;
}