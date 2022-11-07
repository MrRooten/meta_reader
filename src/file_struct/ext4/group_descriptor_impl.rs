use bytes::Buf;

use super::*;

impl GroupDescriptor {
    pub fn get_block_bitmap(&self) -> &DataBlockBitmap {
        unimplemented!()
    }

    pub fn get_inode_bitmap(&self) -> &InodeBitmap {
        unimplemented!()
    }

    pub fn get_inode_table(&self) -> &Inode {
        unimplemented!()
    }

    pub fn get_data_blocks(&self) -> Vec<Bytes> {
        unimplemented!()
    }

    pub fn parse(bs: Bytes) -> Self {
        let mut s = Self::default();
        s.bg_block_bitmap_lo = (&bs[0..4]).get_u32_le();
        s.bg_inode_bitmap_lo = (&bs[4..8]).get_u32_le();
        s
    }
}