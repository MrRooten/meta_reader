#![allow(unused)]
use std::{cell::RefCell, fs::File, io::{BufReader, Read, Seek, SeekFrom}, ops::Range, path::Path};

use super::{MRError};

#[derive(Debug)]
pub struct MRFile {
    path    : String,
    reader  : RefCell<BufReader<File>>
}

impl MRFile {
    pub fn new<P>(p: P) -> Result<MRFile,MRError>
    where P: AsRef<Path> + ToString {
        let s = p.to_string();
        let f = File::open(p);
        let f = match f {
            Ok(file) => file,
            Err(err) => {
                return Err(MRError::from(Box::new(err)));
            }
        };
        let f = RefCell::new(BufReader::new(f));
        Ok(MRFile {
            path: s,
            reader: f,
        })
    }

    pub fn read_n(&self,addr: usize,n: usize) -> Result<Vec<u8>,MRError> {
        let mut reader = self.reader.borrow_mut();
        if let Err(e) = reader.seek(SeekFrom::Start(addr as u64)) {
            return Err(MRError::from(Box::from(e)));
        }
        let mut result = vec![0u8;n];
        let ret = reader.read_exact(&mut result);
        let result = match ret {
            Ok(_ret) => {
                result
            },
            Err(e) => {
                return Err(MRError::from(Box::new(e)));
            }
        };
        Ok(result)
    }

    pub fn read_range(&self, range: Range<usize>) -> Result<Vec<u8>,MRError> {
        self.read_n(range.start, range.end-range.start)
    }   
}

pub fn filesize_to_human_string(size: usize) -> String {
    let result;
    if size > 1024*1024 {
        let human_size = size as f64 / (1024*1024) as f64;
        result = format!("{:.2} MB", human_size);
    } else if size > 1024 {
        let human_size = size as f64 / (1024) as f64;
        result = format!("{:.2} KB", human_size);
    } else {
        let human_size = size as f64;
        result = format!("{} B", human_size);
    }


    result
}