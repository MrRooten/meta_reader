use bytes::{Bytes, Buf};

use crate::utils::{file::MRFile, funcs::sub_bytes, MRError};

use super::{RegFile, RegFileHeader, HiveBin, HiveBinCell};

impl HiveBinCell {
    pub fn new(file: &MRFile, offset: u32) -> Result<HiveBinCell, MRError> {
        unimplemented!()
    }
}

impl HiveBin {
    pub fn new(file: &mut MRFile, offset: u32) -> Result<HiveBin, MRError> {
        let bs = match file.read_n(offset as usize, 32) {
            Ok(o) => o, 
            Err(e) => {
                return Err(e);
            }
        };
        let bs = Bytes::from(bs);
        let sign = sub_bytes(&bs,0..4)?.to_vec();
        if !String::from_utf8_lossy(&sign).eq("hbin") {
            return Err(MRError::new("Not a valid Hive bin"));
        }
        let offset = (sub_bytes(&bs,4..8)?).get_u32_le();
        let size = (sub_bytes(&bs,8..12)?).get_u32_le();
        let offset_of_start = offset;
        Ok(Self {
            sign,
            offset,
            size,
            offset_of_file: offset_of_start,
        })
    }
}

impl RegFileHeader {
    pub fn from_bytes(bs: Bytes) -> Result<RegFileHeader,MRError> {
        let sign = sub_bytes(&bs,0..4)?.to_vec();
        if !String::from_utf8_lossy(&sign).eq("regf") {
            return Err(MRError::new(""));
        }
        let primary_seq_num = (sub_bytes(&bs,4..8)?).get_u32_le();
        let second_seq_num = (sub_bytes(&bs,8..12)?).get_u32_le();
        let last_modify = (sub_bytes(&bs,12..20)?).get_u64_le();
        let major_version = (sub_bytes(&bs,20..24)?).get_u32_le();
        let minor_version = (sub_bytes(&bs,24..28)?).get_u32_le();
        let root_key_offset = (sub_bytes(&bs,36..40)?).get_u32_le();
        let hive_bins_data_size = (sub_bytes(&bs,40..44)?).get_u32_le();
        Ok(Self {
            sign,
            primary_seq_num,
            second_seq_num,
            last_modify,
            major_version,
            minor_version,
            root_key_offset,
            hive_bins_data_size,
        })
    }
}

impl RegFile {
    pub fn from_file(f: &str) -> Result<RegFile, MRError> {
        let mr_file = match MRFile::new(f) {
            Ok(o) => o, 
            Err(e) => {
                return Err(e);
            }
        };

        unimplemented!()
    }

    pub fn get_reader(&self) -> &MRFile {
        &self.file
    }


}