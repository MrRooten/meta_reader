use std::ops::Range;

use bytes::{Bytes, Buf};

use crate::utils::{file::MRFile, MRError, funcs::i_to_m};

use super::{Ext4, SuperBlock, GroupDescriptor, Inode, Block, DirectoryEntry, Journal};

impl Ext4 {
    pub fn open(path: &str) -> Result<Self,MRError> {
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

    fn set_super_block(&mut self) -> Result<&SuperBlock,MRError> {
        let mut super_block = SuperBlock::default();
        let sbytes = self.reader.read_n(1024, 1024).expect("error");
        let sbytes = Bytes::from(sbytes);
        super_block.s_inodes_count = (&sbytes[0..4]).get_u32_le();
        super_block.s_block_count = (&sbytes[4..8]).get_u32_le();
        super_block.s_log_block_size = (&sbytes[0x18..0x1c]).get_u32_le();
        super_block.s_inodes_per_group = (&sbytes[0x28..0x2c]).get_u32_le();
        super_block.s_creator_os = (&sbytes[0x48..0x4c]).get_u32_le();
        super_block.s_inode_size = (&sbytes[0x58..0x5a]).get_u16_le();
        super_block.s_reserved_gdt_blocks = (&sbytes[0xce..0xd0]).get_u16_le();
        super_block.s_desc_size = (&sbytes[0xfe..0x100]).get_u16_le();
        if super_block.s_desc_size > 32 {
            super_block.is_64bit = true;
        } else {
            super_block.is_64bit = false;
            super_block.s_desc_size = 32;
        }
        //if 64bit feature is set
        if super_block.is_64bit {
            super_block.s_log_groups_per_flex = (&sbytes[0x174..0x175]).get_u8();
        }
        self.super_block = Some(super_block);
        match &self.super_block {
            Some(o) => {
                return Ok(o);
            },
            None => {
                return Err(MRError::new("Can not parse super block"));
            }
        }
    }

    pub fn get_super_block(&self) -> Result<&SuperBlock,MRError> {
        if self.super_block.is_some() {
            match &self.super_block {
                Some(s) => {
                    return Ok(s);
                },
                None => {
                }
            }
        }

        let ret = i_to_m(self).set_super_block();
        match ret {
            Ok(o) => {
                return Ok(o);
            },
            Err(e) => {
                return Err(e);
            }
        }

    }

    fn set_descs(&mut self) -> Result<&Vec<GroupDescriptor>,MRError> {
        let mut result: Vec<GroupDescriptor> = Vec::default();
        let super_block = self.get_super_block();
        let super_block = match super_block {
            Ok(s) => s,
            Err(e) => {
                return Err(MRError::from(Box::new(e)));
            }
        };
        let block_size = num::pow(2,(10+super_block.s_log_block_size) as usize);
        let descs_size = super_block.s_desc_size as usize;
        if block_size != 1024 && block_size != 2048 && block_size != 4096 && block_size != 64*1024 {
            return Err(MRError::new("Parse error: block_size is not right"));
        }
        
        let len = super_block.s_log_groups_per_flex as usize * descs_size;
        if len % descs_size != 0 {
            return Err(MRError::new("Parse error: Group Descriptor size is not right"));
        }
        
        let sbytes = self.reader.read_n(block_size, len).expect("error");
        let sbytes = Bytes::from(sbytes);
        let mut i = 0;
        let mut count = 0;
        self.block_size = block_size;
        loop {
            let gdt = self.reader.read_n(block_size + i, descs_size).expect("error");
            let gdt = Bytes::from(gdt);
            if (&gdt[0..4]).get_u32_le() == 0 {
                break;
            }
            result.push(GroupDescriptor::parse(gdt,&self));
            i += descs_size;
            count += 1;
        }
        self.group_descriptors = Some(result);
        
        match &self.group_descriptors {
            Some(o) => {
                return Ok(o);
            },
            None => {
                return Err(MRError::new("Can not parse group descriptors"));
            }
        }
    }

    fn set_inode_tables(&mut self) -> Result<&Vec<Inode>,MRError> {
        let descs = self.get_descs().unwrap();

        unimplemented!()
    }

    pub fn get_reader(&mut self) -> &MRFile {
        &self.reader
    }


    pub fn get_descs(&self) -> Result<&Vec<GroupDescriptor>,MRError> {
        if self.group_descriptors.is_some() {
            match &self.group_descriptors {
                Some(s) => {
                    return Ok(s);
                },
                None => {
                }
            }
        }

        let ret = i_to_m(self).set_descs();
        match ret {
            Ok(o) => {
                return Ok(o);
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn get_desc_size(&self) -> u16 {
        self.get_super_block().unwrap().s_desc_size
    }

    pub fn get_s_inodes_per_group(&self) -> u32 {
        self.get_super_block().unwrap().s_inodes_per_group
    }

    pub fn get_reserved_gdt_num(&self) -> u16 {
        self.get_super_block().unwrap().s_reserved_gdt_blocks
    }

    pub fn get_s_inode_size(&self) -> u16 {
        self.get_super_block().unwrap().s_inode_size
    }

    pub fn get_inode_by_id(&self, id: u32) -> Result<Inode, MRError> {
        let s_inodes_per_group = self.get_s_inodes_per_group();
        let index = (id - 1) / s_inodes_per_group;
        let offset = index * self.get_s_inode_size() as u32;
        let gdts = self.get_descs().unwrap();
        let gdt = &gdts.get(index as usize);
        if let None = gdt {
            return Err(MRError::new("No such a gdt"));
        }

        let gdt = gdt.unwrap();

        gdt.get_inode(id)
    }

    pub fn is_inode_existed(id: u32) -> bool {
        unimplemented!()
    }

    pub fn get_inode_belong_gdt(&self, id: u32) -> Result<&GroupDescriptor, MRError> {
        let s_inodes_per_group = self.get_s_inodes_per_group();
        let index = (id - 1) / s_inodes_per_group;
        let offset = index * self.get_s_inode_size() as u32;
        let gdts = self.get_descs().unwrap();
        let gdt = &gdts.get(index as usize);
        if let None = gdt {
            return Err(MRError::new("No such a gdt"));
        }

        let gdt = gdt.unwrap();
        Ok(gdt)
    }

    pub fn get_inode_id_by_addr(&self, addr: usize) -> Result<u32, MRError> {
        let gdts = self.get_descs().unwrap();
        let per_group = self.get_super_block().unwrap().s_inodes_per_group as usize;
        let max_distance = per_group * 0x100;
        let mut count = 0;
        for gdt in gdts {
            let table_offset = gdt.get_inode_table();
            if table_offset.checked_mul(self.get_block_size() as u64).is_none() {
                continue;
            }
            if addr < (gdt.get_inode_table() as usize*self.get_block_size()) {
                continue;
            }
            let distance = addr - (gdt.get_inode_table() as usize*self.get_block_size());
            if distance < max_distance {
                let result = count*per_group + distance/0x100;
                return Ok(result as u32);
            }
        }
        return Err(MRError::new("Not found inode"));
    }

    pub fn is_64bit(&self) -> bool {
        self.get_super_block().unwrap().is_64bit
    } 

    pub fn get_block_by_id(&self, id: u32) -> Block {
        unimplemented!()
    }

    pub fn get_inode_by_fname(&self, fname: &str) -> Result<Inode, MRError> {
        let entries = fname.split("/").collect::<Vec<&str>>();
        let entries = entries[1..].to_vec();
        let mut cur_inode = self.get_inode_by_id(2).unwrap();
        if fname.eq("/") {
            return Ok(cur_inode);
        }
        let mut count = 0;
        let e_len = entries.len();
        for entry in entries {
            count += 1;
            

            if cur_inode.is_dir() == false  {
                if count == e_len {
                    break;
                }
                return Err(MRError::new("Not a dir"));
            }

            let inode = match cur_inode.get_sub_inode_by_name(entry) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };

            cur_inode = self.get_inode_by_id(inode).unwrap();
            if fname.ends_with("/") {
                if count == e_len - 1 {
                    break;
                }
            }
        }
        Ok(cur_inode)
    }
    
    pub fn is_inode_taken(&mut self, inode: u32) -> bool {
        let index = (inode - 1) % self.get_super_block().unwrap().s_inodes_per_group;
        let gdt = match self.get_inode_belong_gdt(inode) {
            Ok(o) => o,
            Err(e) => {
                return false;
            }
        };
        let range = gdt.get_inode_bitmap();
        let inode_bitmap = self.read_raw(range).unwrap();
        let bit_index = index / 8;
        let bit_offset = index % 8;
        let inode_byte = *inode_bitmap.get(bit_index as usize).unwrap();
        if ((inode_byte >> bit_offset)  & 0x01) == 1 {
            return true;
        }
        false
    }


    pub fn get_root_inode(&self) -> Result<Inode, MRError> {
        self.get_inode_by_id(2)
    }

    pub fn get_user_quota_inode(&self) -> Result<Inode, MRError> {
        self.get_inode_by_id(3)
    }

    pub fn get_group_quota_inode(&self) -> Result<Inode, MRError> {
        self.get_inode_by_id(4)
    }

    pub fn get_bootloader_inode(&self) -> Result<Inode, MRError> {
        self.get_inode_by_id(5)
    }

    pub fn get_undelete_dir_inode(&self) -> Result<Inode, MRError> {
        self.get_inode_by_id(6)
    }

    pub fn get_journal_inode(&self) -> Result<Inode, MRError> {
        self.get_inode_by_id(8)
    }

    pub fn get_block_size(&self) -> usize {
        self.block_size
    }

    pub fn find_unreferenced_idx(&self) {
        unimplemented!()
    }

    pub fn get_jbd2(&self) -> Result<Journal,MRError> {
        let inode = self.get_journal_inode().unwrap();
        let extents = match inode.get_flat_extents() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        let extent = &extents[0];
        Journal::parse(self, extent.get_start()*self.get_block_size())
    }

    pub fn read_raw(&mut self, range: Range<usize>) -> Result<Vec<u8>, MRError> {
        self.reader.read_range(range)
    }
}

