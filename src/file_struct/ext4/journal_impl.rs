use std::ops::Range;

use bytes::{Buf, Bytes};

use crate::utils::{funcs::sub_bytes, MRErrKind, MRError};

use super::{
    CommitBlock, Ext4, Inode, Journal, JournalBlockTag, JournalDataBlock, JournalDescriptorBlock,
    JournalHeader, JournalSuperBlock, JournalTransaction, JournalTransactionIteration,
};

impl JournalHeader {
    pub fn parse(bs: Bytes) -> Result<JournalHeader, MRError> {
        Ok(Self {
            h_magic: (bs.get(0..4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32(),
            h_blocktype: (bs.get(4..8).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32(),
            h_sequence: (bs.get(8..12).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32(),
        })
    }
}

impl JournalBlockTag {
    pub fn get_flag(&self) -> u32 {
        self.flag
    }

    pub fn get_block_id(&self) -> u64 {
        ((self.blocknr_high as u64) << 32) + self.blocknr as u64
    }

    pub fn parse(bs: Bytes, feature: u32, is_64bit: bool) -> Result<Self, MRError> {
        if feature & 0x10 == 0x10 {
            if bs.len() < 16 {
                return Err(MRError::new("Parse size error"));
            }
            let blocknr = (bs.get(0..4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
            let flags = (bs.get(4..8).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
            let blocknr_high = (bs.get(8..0xc).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
            let checksum = (bs.get(0xc..0x10).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();

            if flags & 0x2 == 0x2 {
                return Ok(Self {
                    blocknr,
                    blocknr_high: 0,
                    size: 16,
                    uuid: Vec::default(),
                    flag: flags,
                });
            } else {
                if bs.len() < 32 {
                    return Err(MRError::new("Parse size error"));
                }
                let uuid = sub_bytes(&bs,0xc..0xc + 16)?.to_vec();
                return Ok(Self {
                    blocknr,
                    blocknr_high,
                    size: 32,
                    uuid,
                    flag: flags,
                });
            }
        } else {
            if bs.len() < 8 {
                return Err(MRError::new("Parse size error"));
            }
            let blocknr = (bs.get(0..4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
            let checksum = (bs.get(4..6).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16();
            let flags = (bs.get(6..8).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16();

            if is_64bit {
                if bs.len() < 12 {
                    return Err(MRError::new("Parse size error"));
                }
                let blocknr_high = (bs.get(8..12).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
                if flags & 0x2 == 0x2 {
                    return Ok(Self {
                        blocknr,
                        blocknr_high,
                        size: 12,
                        uuid: Vec::default(),
                        flag: flags as u32,
                    });
                } else {
                    if bs.len() < 28 {
                        return Err(MRError::new("Parse size error"));
                    }
                    let uuid = sub_bytes(&bs,12..12 + 16)?.to_vec();
                    return Ok(Self {
                        blocknr,
                        blocknr_high,
                        size: 28,
                        uuid,
                        flag: flags as u32,
                    });
                }
            } else {
                if bs.len() < 8 {
                    return Err(MRError::new("Parse size error"));
                }
                if flags & 0x2 == 0x2 {
                    return Ok(Self {
                        blocknr,
                        blocknr_high: 0,
                        size: 8,
                        uuid: Vec::default(),
                        flag: flags as u32,
                    });
                } else {
                    if bs.len() < 24 {
                        return Err(MRError::new("Parse size error"));
                    }
                    let uuid = sub_bytes(&bs,8..8 + 16)?.to_vec();
                    return Ok(Self {
                        blocknr,
                        blocknr_high: 0,
                        size: 24,
                        uuid,
                        flag: flags as u32,
                    });
                }
            }
        }
        Err(MRError::new("Error size"))
    }
}

impl JournalDescriptorBlock {
    pub fn get_block_count(&self) -> usize {
        self.open_coded_array.len()
    }

    pub fn parse(bs: Bytes, sb: &JournalSuperBlock, ext4: &Ext4) -> Result<Self, MRError> {
        let header = JournalHeader::parse(bs.slice(0..0xc))?;
        if header.h_magic != 0xC03B3998 {
            return Err(MRError::new("Not a valid descriptor"));
        }
        let mut base_addr = 0xc;
        let mut tags = vec![];
        while base_addr < bs.len() {
            let test = bs.slice(base_addr..).to_vec();
            let tag = match JournalBlockTag::parse(
                bs.slice(base_addr..),
                sb.get_feature_incompat(),
                ext4.is_64bit(),
            ) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };
            base_addr += tag.size;
            let flag = tag.get_flag();
            tags.push(tag);
            if flag & 0x8 == 0x8 {
                break;
            }
        }
        Ok(Self {
            header,
            open_coded_array: tags,
        })
    }
}

impl Journal {
    pub fn parse(ext4: &Ext4, offset: usize) -> Result<Self, MRError> {
        let reader = ext4.get_reader();
        let sb_bs = reader.read_n(offset, 0x100 + 16 * 48).unwrap();
        let sb = JournalSuperBlock::parse(Bytes::from(sb_bs), ext4)?;
        Ok(Self {
            super_block: sb,
            ext4: Some(ext4 as *const Ext4),
            offset,
        })
    }

    fn get_ext4(&self) -> &Ext4 {
        unsafe { &*self.ext4.unwrap() }
    }

    pub fn iter_transaction<F>(&self, f: &mut F)
    where
        F: FnMut(&JournalTransaction),
    {

            let i = 1;
            let ext4 = self.get_ext4();
            let reader = ext4.get_reader();

            let mut base_offset = self.offset + self.super_block.s_blocksize as usize;
            for _dummy in 0..self.super_block.s_maxlen {
                let bs = reader
                    .read_n(base_offset, self.super_block.s_blocksize as usize)
                    .unwrap();
                let bs = Bytes::from(bs);
                let desc = match JournalDescriptorBlock::parse(bs, &self.super_block, ext4) {
                    Ok(o) => o,
                    Err(e) => {
                        base_offset += self.super_block.s_blocksize as usize;
                        continue;
                    }
                };
                let count = desc.get_block_count();
                let mut vs = vec![];
                let mut i = 1;
                for tag in &desc.open_coded_array {
                    let offset = base_offset + i * self.super_block.s_blocksize as usize;
                    let range = Range {
                        start: offset,
                        end: offset + self.super_block.s_blocksize as usize,
                    };
                    i += 1;
                    vs.push(JournalDataBlock::new(tag.get_block_id(), range));
                }
                let bs = reader.read_n(base_offset, 0x3c).unwrap();
                let commit_block = CommitBlock::parse(Bytes::from(bs)).unwrap();
                let transaction = JournalTransaction {
                    desc_block: desc,
                    data_blocks: vs,
                    revocation_blocks: Vec::default(),
                    commit_block,
                };
                f(&transaction);
                base_offset += (count + 2) * self.super_block.s_blocksize as usize;
            }
        
    }

    pub fn reset_iter(&mut self) {
        unimplemented!()
    }

    pub fn find_blocks(&self, block_id: u64) -> Vec<JournalDataBlock> {
        let mut vs = vec![];
        self.iter_transaction(&mut |x| {
            let data_blocks = &x.data_blocks;
            for block in data_blocks {
                if block_id == block.block_id {
                    vs.push(block.clone());
                }
            }
        });
        vs
    }

    pub fn find_inodes(&self, id: u32) -> Result<Vec<Inode>, MRError> {
        let mut result = vec![];
        let ext4 = self.get_ext4();
        let reader = ext4.get_reader();
        let block_size = ext4.get_block_size();
        let s_inodes_per_group = ext4.get_s_inodes_per_group()?;
        let index = (id - 1) / s_inodes_per_group;
        let offset = index * ext4.get_s_inode_size() as u32;
        let gdt = match ext4.get_inode_belong_gdt(id) {
            Ok(o) => o,
            Err(e) => {
                return Ok(result);
            }
        };
        let inode_table = gdt.get_inode_table();
        let sb = ext4.get_super_block().unwrap();
        let index = (id - 1) % sb.s_inodes_per_group;
        let inode_block = index / 0x10;
        let inode_block_offset = index % 0x10;
        let inode_table_block = self.find_blocks(inode_table + inode_block as u64);
        for i in inode_table_block {
            let mut base = i.range.start + 0x100 * inode_block_offset as usize;
            let bs = reader.read_n(base, 0x100).unwrap();
            let inode = Inode::parse(&Bytes::from(bs), ext4, base as u64)?;
            result.push(inode);
        }
        Ok(result)
    }

    pub fn iter_files<F>(&self, mut f: F)
    where
        F: FnMut(u32, &Inode),
    {
        let ext4 = self.get_ext4();
        let reader = ext4.get_reader();
        let gdts = ext4.get_descs().unwrap();
        self.iter_transaction(&mut |transaction| {
            let data_blocks = &transaction.data_blocks;
            for block in data_blocks {
                let block_id = block.block_id;
                let inodes_per_group = ext4.get_s_inodes_per_group().unwrap();
                let num_blocks = inodes_per_group as usize * 0x100 / ext4.get_block_size();
                if !gdts.iter().any(|x| {
                    block_id >= x.get_inode_table() && block_id <= x.get_inode_table() + num_blocks as u64
                }) {
                    continue;
                }
                let mut base_addr = 0;
                let mut count = 0;
                let bs = reader.read_range(block.range.clone()).unwrap();
                let bs = Bytes::from(bs);
                while base_addr < ext4.get_block_size() {
                    if block.block_id.checked_mul(ext4.get_block_size() as u64).is_none() {
                        base_addr += 0x100;
                        count += 1;
                        continue;
                    }
                    let real_addr = block.block_id as usize*ext4.get_block_size() + count * 0x100;
                    let id = match ext4.get_inode_id_by_addr(real_addr) {
                        Ok(o) => o,
                        Err(e) => {
                            base_addr += 0x100;
                            count += 1;
                            continue;
                        }
                    };
                    let inode_bs = bs.slice(base_addr..base_addr+0x100);
                    let inode = Inode::parse(&inode_bs, ext4, base_addr as u64).unwrap();
                    f(id, &inode);
                    base_addr += 0x100;
                    count += 1;
                }
            }
        });
    }
}

impl JournalSuperBlock {
    pub fn parse(bs: Bytes, ext4: &Ext4) -> Result<Self, MRError> {
        let header = JournalHeader::parse(bs.slice(0..0xc))?;
        let s_blocksize = (bs.get(0xc..0x10).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_maxlen = (bs.get(0x10..0x14).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_first = (bs.get(0x14..0x18).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_max_transaction = (bs.get(0x48..0x4c).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_errno = (bs.get(0x20..0x24).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_max_trans_data = (bs.get(0x4c..0x50).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_feature_compat = (bs.get(0x24..0x28).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_feature_incompat = (bs.get(0x28..0x2c).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        let s_feature_ro_compat = (bs.get(0x2c..0x30).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32();
        Ok(Self {
            header,
            s_blocksize,
            s_maxlen,
            s_first,
            s_errno,
            s_max_transaction,
            s_max_trans_data,
            s_feature_compat,
            s_feature_incompat,
            s_feature_ro_compat,
        })
    }

    pub fn get_feature_incompat(&self) -> u32 {
        self.s_feature_incompat
    }
}

impl CommitBlock {
    pub fn parse(bs: Bytes) -> Result<CommitBlock, MRError> {
        Ok(Self {
            header: JournalHeader::parse(bs.slice(0..0xc))?,
            commit_sec: (bs.get(0x30..0x38).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64(),
            commit_nsec: (bs.get(0x38..0x3c).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32(),
        })
    }
}

impl JournalDataBlock {
    pub fn new(block_id: u64, range: Range<usize>) -> Self {
        Self {
            range,
            block_id,
        }
    }

    pub fn get_range(&self) -> Range<usize> {
        self.range.clone()
    }
}
