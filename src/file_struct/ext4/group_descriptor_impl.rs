use bytes::Buf;
use std::ops::Range;

use crate::utils::{funcs::i_to_m, MRError};

use super::*;

impl GroupDescriptor {
    pub fn get_block_bitmap(&self) -> Range<usize> {
        unsafe {
            let ext4 = &(*self.ext4_to_self.unwrap());
            let block_size = ext4.get_block_size();
            let mut offset = self.bg_block_bitmap_lo as usize;
            if self.is_64bit {
                offset = (self.bg_block_bitmap_hi as usize) << 32 | offset;
            }
            Range {
                start: offset,
                end: offset + block_size,
            }
        }
    }


    pub fn get_inode_bitmap(&self) -> Range<usize> {
        unsafe {
            let ext4 = &(*self.ext4_to_self.unwrap());
            let block_size = ext4.get_block_size();
            let mut offset = self.bg_inode_bitmap_lo as usize;
            if self.is_64bit {
                offset = (self.bg_inode_bitmap_hi as usize) << 32 | offset;
            }
            let offset = offset * block_size;
            Range {
                start: offset,
                end: offset + block_size,
            }
        }
    }

    pub fn count_free_inodes(&self) -> u32 {
        ((self.bg_free_inodes_count_hi as u32) << 16) + self.bg_free_inodes_count_lo as u32
    }

    pub fn get_inode(&self, id: u32) -> Result<Inode, MRError> {
        unsafe {
            let ext4 = &(*self.ext4_to_self.unwrap());
            let reader = i_to_m(ext4).get_reader();
            let block_size = ext4.get_block_size();
            let mut offset = self.bg_inode_table_lo as usize;
            if self.is_64bit {
                offset = (self.bg_inode_table_hi as usize) << 32 | offset;
            }

            let sb = ext4.get_super_block().unwrap();
            let table_size = (sb.s_inode_size as u32 * sb.s_inodes_per_group) as usize;
            let s_inodes = sb.s_inodes_per_group;
            let count_free_inodes = self.count_free_inodes();
            if s_inodes < count_free_inodes {
                return Err(MRError::new("Error with count_free_inodes"));
            }
            let len = s_inodes - count_free_inodes;
            let index = (id - 1) % sb.s_inodes_per_group;
            let mut base = offset * ext4.get_block_size() + index as usize * 0x100;
            let bs = reader.read_n(base, 0x100).unwrap();
            let inode = Inode::parse(&Bytes::from(bs), ext4, base as u64);
            Ok(inode)
        }
    }

    pub fn get_inode_table(&self) -> u64 {
        let mut offset = self.bg_inode_table_lo as usize;
        if self.is_64bit {
            offset = (self.bg_inode_table_hi as usize) << 32 | offset;
        }
        offset as u64
    }

    pub fn iter_inodes(&self, f: fn(Inode)) {
        unsafe {
            let ext4 = &(*self.ext4_to_self.unwrap());
            let reader = i_to_m(ext4).get_reader();
            let block_size = ext4.get_block_size();
            let mut offset = self.bg_inode_table_lo as usize;
            if self.is_64bit {
                offset = (self.bg_inode_table_hi as usize) << 32 | offset;
            }
            let sb = ext4.get_super_block().unwrap();
            let table_size = (sb.s_inode_size as u32 * sb.s_inodes_per_group) as usize;
            let s_inodes = sb.s_inodes_per_group;
            let len = s_inodes - self.count_free_inodes();
            let mut i = 0;
            let mut base = offset * ext4.get_block_size();
            while i < len {
                let bs = reader.read_n(base, 0x100).unwrap();
                let inode = Inode::parse(&Bytes::from(bs), ext4, base as u64);
                f(inode);
                base += 0x100;
                i += 1;
            }
        }
    }

    pub fn get_data_blocks(&self) -> Vec<Bytes> {
        unimplemented!()
    }

    pub fn parse(bs: Bytes, ext4_self: &Ext4) -> Self {
        let mut s = Self::default();
        s.ext4_to_self = Some(ext4_self as *const Ext4);
        s.bg_block_bitmap_lo = (&bs[0..4]).get_u32_le();
        s.bg_inode_bitmap_lo = (&bs[4..8]).get_u32_le();
        s.bg_inode_table_lo = (&bs[8..12]).get_u32_le();
        s.bg_free_blocks_count_lo = (&bs[12..14]).get_u16_le();
        s.bg_free_inodes_count_lo = (&bs[14..16]).get_u16_le();
        let s_block = ext4_self.get_super_block().unwrap();
        s.is_64bit = s_block.is_64bit;
        if s_block.is_64bit {
            s.bg_block_bitmap_hi = (&bs[0x20..0x24]).get_u32_le();
            s.bg_inode_bitmap_hi = (&bs[0x24..0x28]).get_u32_le();
            s.bg_inode_table_hi = (&bs[0x28..0x2c]).get_u32_le();
            s.bg_free_blocks_count_hi = (&bs[0x2c..0x2e]).get_u16_le();
            s.bg_free_inodes_count_hi = (&bs[0x2e..0x30]).get_u16_le();
        }
        s
    }
}
