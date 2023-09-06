use crate::file_struct::{File, FileSystem};

use super::{MFTEntry, Ntfs};

impl File for MFTEntry {
    fn read(&self, start: usize, size: usize) -> Result<bytes::Bytes, crate::utils::MRError> {
        todo!()
    }

    fn get_size(&self) -> Result<usize, crate::utils::MRError> {
        todo!()
    }

    fn get_owner(&self) -> Result<String, crate::utils::MRError> {
        todo!()
    }

    fn get_mtime(&self) -> Result<chrono::NaiveDateTime, crate::utils::MRError> {
        todo!()
    }

    fn get_ctime(&self) -> Result<chrono::NaiveDateTime, crate::utils::MRError> {
        todo!()
    }

    fn get_atime(&self) -> Result<chrono::NaiveDateTime, crate::utils::MRError> {
        todo!()
    }
}

impl FileSystem for Ntfs {
    fn list_files(&self, path: &str) -> Result<Vec<Box<dyn File>>, crate::utils::MRError> {
        todo!()
    }

    fn open_file(&self, path: &str) -> Result<Box<dyn File>, crate::utils::MRError> {
        todo!()
    }

    fn copy(&self, fs_path: &str, local_path: &str) -> Result<(), crate::utils::MRError> {
        todo!()
    }
}