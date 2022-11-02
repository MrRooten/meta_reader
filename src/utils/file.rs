use std::{fs::File, io::{BufReader, SeekFrom, Read, Seek}};

use super::{MRError, funcs::i_to_m};

#[derive(Debug,Default)]
pub struct MRFile {
    path    : String,
    reader  : Option<BufReader<File>>
}

impl MRFile {
    pub fn new(p: &str) -> Result<MRFile,MRError> {
        let f = File::open(p);
        let f = match f {
            Ok(file) => file,
            Err(err) => {
                return Err(MRError::from(Box::new(err)));
            }
        };
        let f = BufReader::new(f);
        Ok(MRFile {
            path: p.to_string(),
            reader: Some(f),
        })
    }

    pub fn read_n(&self,addr: usize,n: usize) -> Result<Vec<u8>,MRError> {
        let reader = match &self.reader {
            Some(reader) => {
                reader
            },
            None => {
                return Err(MRError::new("error"));
            }
        };
        i_to_m(reader).seek(SeekFrom::Start(addr as u64));
        let mut result = vec![0u8;n];
        let ret = i_to_m(reader).read_exact(&mut result);
        let result = match ret {
            Ok(ret) => {
                result
            },
            Err(e) => {
                return Err(MRError::from(Box::new(e)));
            }
        };
        Ok(result)
    }
}