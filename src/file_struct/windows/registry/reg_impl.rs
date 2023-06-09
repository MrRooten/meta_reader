use bytes::{Bytes, Buf};

use crate::utils::{MRError, file::MRFile};

use super::{RegFile, RegFileHeader, HiveBin, HiveBinCell};

impl HiveBinCell {
    pub fn new(file: &MRFile, offset: u32) -> Result<HiveBinCell, MRError> {
        unimplemented!()
    }
}

impl HiveBin {
    pub fn new(file: &MRFile, offset: u32) -> Result<HiveBin, MRError> {
        let bs = match file.read_n(offset as usize, 32) {
            Ok(o) => o, 
            Err(e) => {
                return Err(e);
            }
        };
        let bs = Bytes::from(bs);
        let sign = bs[0..4].to_vec();
        if String::from_utf8_lossy(&sign).eq("hbin") == false {
            return Err(MRError::new("Not a valid Hive bin"));
        }
        let offset = (&bs[4..8]).get_u32_le();
        let size = (&bs[8..12]).get_u32_le();
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
        let sign = bs[0..4].to_vec();
        if String::from_utf8_lossy(&sign).eq("regf") == false {
            return Err(MRError::new(""));
        }
        let primary_seq_num = (&bs[4..8]).get_u32_le();
        let second_seq_num = (&bs[8..12]).get_u32_le();
        let last_modify = (&bs[12..20]).get_u64_le();
        let major_version = (&bs[20..24]).get_u32_le();
        let minor_version = (&bs[24..28]).get_u32_le();
        let root_key_offset = (&bs[36..40]).get_u32_le();
        let hive_bins_data_size = (&bs[40..44]).get_u32_le();
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
        return &self.file
    }


}