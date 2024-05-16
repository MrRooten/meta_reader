use bytes::{Buf, Bytes};
use chrono::NaiveDateTime;

use crate::utils::{file::MRFile, funcs::sub_bytes, MRError};

use super::{
    DirectoryEntry, Ext4, Extent, ExtentHeader, ExtentIdx, ExtentNode, ExtentNodeType, ExtentTree,
    FileMode, FileType, Inode,
};

impl Extent {
    pub fn parse(bs: &Bytes) -> Result<Extent, MRError> {
        Ok(Extent {
            ee_block: (sub_bytes(bs, 0..4)?).get_u32_le(),
            ee_len: (sub_bytes(bs, 4..6)?).get_u16_le(),
            ee_start_hi: (sub_bytes(bs, 6..8)?).get_u16_le(),
            ee_start_lo: (sub_bytes(bs, 8..12)?).get_u32_le(),
        })
    }

    pub fn get_start(&self) -> usize {
        ((self.ee_start_hi as usize) << 32) + self.ee_start_lo as usize
    }

    pub fn get_len(&self) -> usize {
        self.ee_len as usize
    }
}

impl ExtentIdx {
    pub fn parse(bs: &Bytes) -> Result<Self, MRError> {
        Ok(Self {
            ei_block: (sub_bytes(bs, 0..4)?).get_u32_le(),
            ei_leaf_lo: (sub_bytes(bs, 4..8)?).get_u32_le(),
            ei_leaf_hi: (sub_bytes(bs, 8..0xa)?).get_u16_le(),
            ei_unused: (sub_bytes(bs, 10..12)?).get_u16_le(),
        })
    }
}

const EXTENT_SIZE: usize = 12;
const EXTENT_HEADER_SIZE: usize = 12;
const EXTENT_IDX_SIZE: usize = 12;

impl ExtentNode {
    pub fn parse_header(bs: &Bytes, offset: u64) -> Result<ExtentNode, MRError> {
        let eh_magic = (sub_bytes(bs, 0..2)?).get_u16_le();
        let en_entries = (sub_bytes(bs, 2..4)?).get_u16_le();
        let en_max = (sub_bytes(bs, 4..6)?).get_u16_le();
        let en_depth = (sub_bytes(bs, 6..8)?).get_u16_le();
        let en_generation = (sub_bytes(bs, 8..12)?).get_u32_le();
        let mut extents = vec![];
        let mut idx_items = vec![];
        let t = if en_depth == 0 {
            ExtentNodeType::ExtentType
        } else {
            ExtentNodeType::IdxType
        };

        let header = ExtentHeader {
            eh_magic,
            eh_entries: en_entries,
            eh_max: en_max,
            eh_depth: en_depth,
            eh_generation: en_generation,
        };

        Ok(Self {
            header,
            idx_items,
            extents,
            node_type: t,
            base_addr: offset,
        })
    }

    pub fn parse(bs: &Bytes, offset: u64) -> Result<ExtentNode, MRError> {
        let eh_magic = (sub_bytes(bs, 0..2)?).get_u16_le();
        let en_entries = (sub_bytes(bs, 2..4)?).get_u16_le();
        let en_max = (sub_bytes(bs, 4..6)?).get_u16_le();
        let en_depth = (sub_bytes(bs, 6..8)?).get_u16_le();
        let en_generation = (sub_bytes(bs, 8..12)?).get_u32_le();

        let mut extents = vec![];
        let mut idx_items = vec![];
        let t = if en_depth == 0 {
            let mut index = 12;
            for i in 0..en_max {
                let v = bs[index..index + 12].to_vec();
                let t = Bytes::from(v);
                let extent = Extent::parse(&t)?;
                if extent.ee_start_lo == 0 && extent.ee_start_hi == 0 {
                    continue;
                }
                extents.push(extent);
                index += 12;
            }
            ExtentNodeType::ExtentType
        } else {
            let mut index = 12;
            for i in 0..en_max {
                if index + 12 > bs.len() {
                    break;
                }
                let v = bs[index..index + 12].to_vec();
                let t = Bytes::from(v);
                let idx = ExtentIdx::parse(&t)?;
                if idx.ei_leaf_lo == 0 && idx.ei_leaf_hi == 0 {
                    continue;
                }
                idx_items.push(idx);
                index += 12;
            }
            ExtentNodeType::IdxType
        };

        let header = ExtentHeader {
            eh_magic,
            eh_entries: en_entries,
            eh_max: en_max,
            eh_depth: en_depth,
            eh_generation: en_generation,
        };

        Ok(Self {
            header,
            idx_items,
            extents,
            node_type: t,
            base_addr: offset,
        })
    }
}

impl Inode {
    pub fn get_extent_tree(&self) -> Result<ExtentTree, MRError> {
        ExtentTree::parse(&Bytes::from(self.i_block.clone()), self.base_addr + 0x28)
    }

    pub fn is_deleted(&self) -> bool {
        if self.i_block[2] == 0 && self.i_block[3] == 0 && self.get_size() != 0 {
            return true;
        }

        false
    }

    pub fn get_uid(&self) -> u16 {
        self.i_uid
    }

    pub fn is_empty(&self) -> Result<bool, MRError> {
        let tree = self.get_extent_tree()?;
        if tree.extents.len() != 0 || tree.idx_items.len() != 0 {
            return Ok(false);
        }

        Ok(true)
    }
    #[allow(deprecated)]
    pub fn get_atime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.i_atime as i64, 0).unwrap()
    }

    #[allow(deprecated)]
    pub fn get_ctime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.i_ctime as i64, 0).unwrap()
    }

    #[allow(deprecated)]
    pub fn get_dtime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.i_dtime as i64, 0).unwrap()
    }

    #[allow(deprecated)]
    pub fn get_mtime(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.i_mtime as i64, 0).unwrap()
    }

    #[allow(deprecated)]
    pub fn get_birth(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.i_crtime as i64, 0).unwrap()
    }
    pub fn get_sub_dirs(&self) -> Result<Vec<DirectoryEntry>, MRError> {
        if !self.is_dir() {
            return Err(MRError::new("Not a dir"));
        }

        let value = self.get_extents_value().unwrap();
        let ext4 = self.get_ext4();
        let mut cur = &value;
        let mut result = vec![];
        let mut base_block = 0;
        while base_block < value.len() {
            let mut base_addr = 0;
            while base_addr < ext4.get_block_size() {
                let bs = value.slice(base_block..base_block + ext4.get_block_size());
                let entry = DirectoryEntry::parse_with_len_return(&bs.slice(base_addr..), 0);
                if let Err(e) = entry.0 {
                    break;
                }
                let dir = entry.0.unwrap();
                if dir.get_id() <= 11 {
                    base_addr += entry.1;
                    continue;
                }
                result.push(dir);
                base_addr += entry.1
            }
            base_block += ext4.get_block_size();
        }
        Ok(result)
    }

    fn align_of_4(&self, n: usize) -> usize {
        if n % 4 == 0 {
            n
        } else {
            (4 - (n % 4)) + n
        }
    }
    //Get sub dirs by raw way, means may search the file may not existed(deleted)
    pub fn get_sub_dirs_raw(&self) -> Result<Vec<DirectoryEntry>, MRError> {
        if !self.is_dir() {
            return Err(MRError::new("Not a dir"));
        }

        let value = self.get_extents_value().unwrap();
        let ext4 = self.get_ext4();
        let mut cur = &value;

        let mut result = vec![];
        let mut base_block = 0;
        while base_block < value.len() {
            let mut base_addr = 0;
            let bs = value.slice(base_block..base_block + ext4.get_block_size());
            while base_addr < ext4.get_block_size() {
                let entry = DirectoryEntry::parse_with_len_return(&bs.slice(base_addr..), 0);
                let dir = match entry.0 {
                    Ok(o) => o,
                    Err(e) => {
                        base_addr += 4;
                        continue;
                    }
                };
                let len = dir.name_len;
                if dir.get_id() <= 11 {
                    base_addr += 4;
                    continue;
                }
                result.push(dir);
                base_addr += 4;
            }
            base_block += ext4.get_block_size();
        }
        Ok(result)
    }

    pub fn iter_blocks<F>(&self, f: F) -> Result<(), MRError>
    where
        F: Fn(Bytes),
    {
        let extents = match self.get_flat_extents() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let ext4 = self.get_ext4();
        let reader = ext4.get_reader();

        for extent in extents {
            let mut base_addr = extent.get_start() * ext4.get_block_size();
            let end_addr = base_addr + (extent.ee_len as usize) * ext4.get_block_size();
            let mut i = 0;
            while i < extent.ee_len {
                let bs = match reader.read_n(base_addr, ext4.get_block_size()) {
                    Ok(o) => o,
                    Err(e) => {
                        return Err(e);
                    }
                };
                let bs = Bytes::from(bs);
                f(bs);
                base_addr += ext4.get_block_size();
                i += 1;
            }
        }

        Ok(())
    }

    fn get_ext4(&self) -> &Ext4 {
        unsafe { &*self.ext4.unwrap() }
    }

    pub fn get_sub_inode_by_name(&self, name: &str) -> Result<u32, MRError> {
        if !self.is_dir() {
            return Err(MRError::new("Not a dir"));
        }

        let value = self.get_extents_value().unwrap();
        let ext4 = self.get_ext4();
        let mut cur = &value;
        let mut base_block = 0;
        while base_block < value.len() {
            let mut base_addr = 0;
            while base_addr < ext4.get_block_size() {
                let bs = value.slice(base_block..base_block + ext4.get_block_size());
                let entry = DirectoryEntry::parse_with_len_return(&bs.slice(base_addr..), 0);
                if let Err(e) = entry.0 {
                    break;
                }
                let dir = entry.0.unwrap();
                if String::from_utf8_lossy(&dir.name).eq(name) {
                    return Ok(dir.get_id());
                }
                base_addr += entry.1
            }
            base_block += ext4.get_block_size();
        }

        Err(MRError::new("file not found"))
    }

    pub fn get_size(&self) -> u64 {
        self.i_size_lo as u64 + ((self.i_size_high as u64) << 32)
    }

    pub fn get_flat_extents(&self) -> Result<Vec<Extent>, MRError> {
        let mut extents = vec![];
        let mut stack = vec![];
        let ext4 = self.get_ext4();
        let reader = ext4.get_reader();
        let first = ExtentTree::parse(&Bytes::from(self.i_block.clone()), self.base_addr + 0x28)?;
        stack.push(first);
        while let Some(f) = stack.pop() {
            if f.node_type.eq(&ExtentNodeType::ExtentType) {
                let mut index = f.base_addr + EXTENT_HEADER_SIZE as u64;
                for i in 0..f.header.eh_entries {
                    let v = reader.read_n(index as usize, EXTENT_SIZE).unwrap();
                    let t = Bytes::from(v);
                    let extent = Extent::parse(&t)?;
                    if extent.ee_start_lo == 0 && extent.ee_start_hi == 0 {
                        continue;
                    }
                    extents.push(extent);
                    index += 12;
                }
            } else {
                let mut index = f.base_addr + EXTENT_HEADER_SIZE as u64;
                if f.header.eh_magic != 0xF30A {
                    return Err(MRError::new("Not valid extent"));
                }
                for i in 0..f.header.eh_entries {
                    let v = reader.read_n(index as usize, EXTENT_IDX_SIZE).unwrap();
                    let t = Bytes::from(v);
                    let idx = ExtentIdx::parse(&t)?;
                    if idx.ei_leaf_lo == 0 && idx.ei_leaf_hi == 0 {
                        continue;
                    }

                    let leaf_offset = ((idx.ei_leaf_hi as u64) << 32) + idx.ei_leaf_lo as u64;
                    let leaf_offset = (leaf_offset as usize) * ext4.get_block_size();
                    let header_bs = reader.read_n(leaf_offset, 12).unwrap();
                    let mut child_node =
                        ExtentNode::parse_header(&Bytes::from(header_bs), leaf_offset as u64)?;
                    let mut idx_index = 0;
                    let idxs_size = EXTENT_IDX_SIZE * (child_node.header.eh_entries as usize);
                    let idxs_bs = reader
                        .read_n(leaf_offset + EXTENT_HEADER_SIZE, idxs_size)
                        .unwrap();
                    for j in 0..child_node.header.eh_entries {
                        let _vs = idxs_bs[idx_index..(idx_index + EXTENT_IDX_SIZE)].to_vec();
                        let child_idx = ExtentIdx::parse(&Bytes::from(_vs))?;
                        child_node.idx_items.push(child_idx);
                        idx_index += EXTENT_IDX_SIZE;
                    }
                    index += EXTENT_IDX_SIZE as u64;
                    stack.push(child_node);
                }
            }
        }
        Ok(extents)
    }

    pub fn is_dir(&self) -> bool {
        self.i_mode & 0x4000 == 0x4000
    }

    pub fn is_socket(&self) -> bool {
        self.i_mode & 0xc000 == 0xc000
    }

    pub fn get_extents_value(&self) -> Result<Bytes, MRError> {
        let extents = match self.get_flat_extents() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let ext4 = self.get_ext4();
        let reader = ext4.get_reader();
        let mut result = Vec::new();

        for extent in extents {
            let mut bs = match reader.read_n(
                extent.get_start() * ext4.get_block_size(),
                (extent.ee_len as usize) * ext4.get_block_size(),
            ) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };

            result.append(&mut bs);
        }
        Ok(Bytes::from(result))
    }

    pub fn is_symbolic_link(&self) -> bool {
        self.i_mode & 0xa000 == 0xa000
    }

    pub fn is_char_device(&self) -> bool {
        self.i_mode & 0x2000 == 0x2000
    }

    pub fn is_block_device(&self) -> bool {
        self.i_mode & 0x2000 == 0x2000
    }

    pub fn is_regular_file(&self) -> bool {
        self.i_mode & 0x8000 == 0x8000
    }

    pub fn parse(bs: &Bytes, ext4: &Ext4, offset: u64) -> Result<Inode, MRError> {
        let i_mode = (sub_bytes(bs, 0..2)?).get_u16_le();
        let i_uid = (sub_bytes(bs, 2..4)?).get_u16_le();
        let i_size_lo = (sub_bytes(bs, 0x4..8)?).get_u32_le();
        let i_atime = (sub_bytes(bs, 0x8..12)?).get_u32_le();
        let i_ctime = (sub_bytes(bs, 0xc..0xc + 4)?).get_u32_le();
        let i_mtime = (sub_bytes(bs, 0x10..0x14)?).get_u32_le();
        let i_dtime = (sub_bytes(bs, 0x14..0x18)?).get_u32_le();
        let i_gid = (sub_bytes(bs, 0x18..0x1a)?).get_u16_le();
        let i_link_count = (sub_bytes(bs, 0x1a..0x1c)?).get_u16_le();
        let i_blocks_lo = (sub_bytes(bs, 0x1c..0x20)?).get_u32_le();
        let i_flags = (sub_bytes(bs, 0x20..0x24)?).get_u32_le();
        let i_generation = (sub_bytes(bs, 0x64..0x68)?).get_u32_le();
        let i_file_acl_lo = (sub_bytes(bs, 0x68..0x6c)?).get_u32_le();
        let i_size_high = (sub_bytes(bs, 0x6c..0x70)?).get_u32_le();
        let i_obso_faddr = (sub_bytes(bs, 0x70..0x74)?).get_u32_le();
        let i_extra_isize = (sub_bytes(bs, 0x80..0x82)?).get_u16_le();
        let i_checksum_hi = (sub_bytes(bs, 0x82..0x84)?).get_u16_le();
        let i_ctime_extra = (sub_bytes(bs, 0x84..0x88)?).get_u32_le();
        let i_mtime_extra = (sub_bytes(bs, 0x88..0x8c)?).get_u32_le();
        let i_atime_extra = (sub_bytes(bs, 0x8c..0x90)?).get_u32_le();
        let i_ctrime = (sub_bytes(bs, 0x90..0x94)?).get_u32_le();
        let i_ctrime_extra = (sub_bytes(bs, 0x94..0x98)?).get_u32_le();
        let i_version_hi = (sub_bytes(bs, 0x98..0x9c)?).get_u32_le();
        let i_projid = (sub_bytes(bs, 0x9c..0x100)?).get_u32_le();
        let i_block = (sub_bytes(bs, 0x28..0x64)?);
        Ok(Inode {
            i_mode,
            i_uid,
            i_size_lo,
            i_atime,
            i_ctime,
            i_mtime,
            i_dtime,
            i_gid,
            i_links_count: i_link_count,
            i_blocks_lo,
            i_flags,
            i_generation,
            i_file_acl_lo,
            i_size_high,
            i_obso_faddr,
            i_extra_isize,
            i_checksum_hi,
            i_ctime_extra,
            i_mtime_extra,
            i_atime_extra,
            i_crtime: i_ctrime,
            i_crtime_extra: i_ctrime_extra,
            i_version_hi,
            i_projid,
            i_block: i_block.to_vec(),
            ext4: Some(ext4 as *const Ext4),
            base_addr: offset,
        })
    }
}

impl TryFrom<u8> for FileType {
    type Error = MRError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0x0 {
            return Ok(Self::Unknown);
        } else if value == 0x1 {
            return Ok(Self::RegularFile);
        } else if value == 0x2 {
            return Ok(Self::Directory);
        } else if value == 0x3 {
            return Ok(Self::CharacterDeviceFile);
        } else if value == 0x4 {
            return Ok(Self::BlockDeviceFile);
        } else if value == 0x5 {
            return Ok(Self::FIFO);
        } else if value == 0x6 {
            return Ok(Self::Socket);
        } else if value == 0x7 {
            return Ok(Self::SymbolicLink);
        }
        let msg = format!("Error for value {}", value);
        Err(MRError::new(&msg))
    }
}

impl DirectoryEntry {
    pub fn get_zero_end_name(&self) -> &String {
        &self.with_zero_end_string
    }

    pub fn get_f_type(&self) -> &FileType {
        &self.file_type
    }

    pub fn parse_with_len_return(
        bs: &Bytes,
        start_with: usize,
    ) -> (Result<DirectoryEntry, MRError>, usize) {
        if bs.len() < 7 {
            return (Err(MRError::new("Not a valid directory")), 0);
        }
        let id = (&bs[start_with..start_with + 4]).get_u32_le();
        if id == 0 {
            return (Err(MRError::new("Not a valid inode id")), 0);
        }
        let rec_len = (&bs[start_with + 4..start_with + 6]).get_u16_le();
        if rec_len > 4096 {
            return (Err(MRError::new("Not a valid rec_len")), 0);
        }
        let name_len = (&bs[start_with + 6..start_with + 7]).get_u8();
        let f_type = (&bs[start_with + 7..start_with + 8]).get_u8();
        let ft = match FileType::try_from(f_type) {
            Ok(o) => o,
            Err(e) => return (Err(e), 0),
        };

        let mut real_name_len = 0;
        for i in (start_with + 8)..bs.len() {
            if bs[i] == 0 {
                break;
            }
            real_name_len += 1;
        }
        if start_with + (8 + name_len as usize) > bs.len() {
            return (Err(MRError::new("Not a valid name")), 0);
        }
        let name = bs[start_with + 8..start_with + (8 + name_len as usize)].to_vec();
        if start_with + (8 + real_name_len as usize) > bs.len() {
            return (Err(MRError::new("Not a valid zero_end_name")), 0);
        }
        let zero_end_name = bs[start_with + 8..start_with + (8 + real_name_len as usize)].to_vec();
        let zero_end_name = String::from_utf8_lossy(&zero_end_name);

        (
            Ok(Self {
                inode: id,
                rec_len,
                name_len,
                file_type: ft,
                utf8_name: String::from_utf8_lossy(&name).to_string(),
                name,
                with_zero_end_string: zero_end_name.to_string(),
            }),
            rec_len as usize,
        )
    }

    pub fn get_name(&self) -> &String {
        &self.utf8_name
    }

    pub fn get_id(&self) -> u32 {
        self.inode
    }
}
