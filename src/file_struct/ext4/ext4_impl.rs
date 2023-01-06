use bytes::{Bytes, Buf};

use crate::utils::{file::MRFile, MRError, funcs::i_to_m};

use super::{Ext4, SuperBlock, GroupDescriptor, Inode, Block};

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
        if super_block.s_desc_size == 64 {
            super_block.is_64bit = true;
        } else {
            super_block.is_64bit = false;
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

    pub fn get_inode_by_id(&self, id: u32) -> Inode {
        let s_inodes_per_group = self.get_s_inodes_per_group();
        let index = (id - 1) / s_inodes_per_group;
        let offset = index * self.get_s_inode_size() as u32;
        let gdts = self.get_descs().unwrap();
        let gdt = &gdts[index as usize];
        gdt.get_inode(id)
    }

    pub fn get_block_by_id(&self, id: u32) -> Block {
        unimplemented!()
    }

    pub fn get_inode_by_fname(&self, fname: &str) -> Inode {
        unimplemented!()
    }

    pub fn get_root_inode(&self) -> Inode {
        self.get_inode_by_id(2)
    }

    pub fn get_user_quota_inode(&self) -> Inode {
        self.get_inode_by_id(3)
    }

    pub fn get_group_quota_inode(&self) -> Inode {
        self.get_inode_by_id(4)
    }

    pub fn get_bootloader_inode(&self) -> Inode {
        self.get_inode_by_id(5)
    }

    pub fn get_undelete_dir_inode(&self) -> Inode {
        self.get_inode_by_id(6)
    }

    pub fn get_journal_inode(&self) -> Inode {
        self.get_inode_by_id(7)
    }

    pub fn get_block_size(&self) -> usize {
        self.block_size
    }

    pub fn find_unreferenced_idx(&self) {
        unimplemented!()
    }
}

