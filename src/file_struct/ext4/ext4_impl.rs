use bytes::{Bytes, Buf};

use crate::utils::{file::MRFile, MRError};

use super::{Ext4, SuperBlock};


impl Ext4 {
    fn open(path: &str) -> Result<Self,MRError> {
        let mr_file = MRFile::new(path);
        let mr_file = match mr_file {
            Ok(file) => file,
            Err(e) => {
                return Err(MRError::from(Box::new(e)));
            }
        };

        Ok(Ext4 {
            reader : mr_file,
            ..Default::default()
        })
    }

    fn set_super_block(&mut self) -> Result<(),MRError> {
        let mut super_block = SuperBlock::default();
        let sbytes = self.reader.read_n(1024, 1024).expect("error");
        let sbytes = Bytes::from(sbytes);
        super_block.s_inodes_count = (&sbytes[0..4]).get_u32_le();
        super_block.s_block_count = (&sbytes[4..8]).get_u32_le();
        super_block.s_log_block_size = (&sbytes[8..12]).get_u32_le();
        super_block.s_creator_os = (&sbytes[0x48..0x4c]).get_u32_le();
        unimplemented!()
    }
    pub fn get_super_block(&self) -> Result<&SuperBlock,MRError> {
        unimplemented!()
    }
}