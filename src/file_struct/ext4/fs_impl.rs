use bytes::Bytes;

use crate::{file_struct::{FileSystem, File}, utils::MRError};

use super::{Ext4, Inode};

impl File for Inode {
    fn read(&self, start: usize, size: usize) -> Result<Bytes, MRError> {
        let mut result = vec![];
        let extents = match self.get_flat_extents() {
            Ok(s) => s,
            Err(e) => {
                return Err(e);
            }
        };

        let mut rest_size = size;
        let mut cur_start = start;
        let ext4 = unsafe { &(*self.ext4.unwrap()) };
        let reader = ext4.get_reader();
        for ext in &extents {
            if rest_size == 0 {
                break;
            }
            if ext.get_start() + ext.get_len() > cur_start {
                let read_size = {
                    let rest_ext_size = (ext.get_len() + ext.get_start()) - cur_start;
                    if rest_ext_size >= rest_size {
                        rest_size
                    } else {
                        rest_size - rest_ext_size
                    }
                };
                let mut bs = match reader.read_n(ext.get_start() + cur_start, read_size) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(e);
                    }
                };
                rest_size -= read_size;
                cur_start += read_size;
                result.append(&mut bs);
            }


        }

        Ok(Bytes::from(result))
    }

    fn get_size(&self) -> Result<usize, MRError> {
        unimplemented!()
    }

    fn get_owner(&self) -> Result<String, MRError> {
        todo!()
    }

    fn get_mtime(&self) -> Result<chrono::NaiveDateTime, MRError> {
        Ok(self.get_mtime())
    }

    fn get_ctime(&self) -> Result<chrono::NaiveDateTime, MRError> {
        Ok(self.get_ctime())
    }

    fn get_atime(&self) -> Result<chrono::NaiveDateTime, MRError> {
        Ok(self.get_atime())
    }
}

impl FileSystem for Ext4 {
    fn list_files(&self, path: &str) -> Result<Vec<Box<dyn File>>, MRError> {
        todo!()
    }

    fn open_file(&self, path: &str) -> Result<Box<dyn File>, MRError> {
        todo!()
    }

    fn copy(&self, fs_path: &str, local_path: &str) -> Result<(), MRError> {
        todo!()
    }
}