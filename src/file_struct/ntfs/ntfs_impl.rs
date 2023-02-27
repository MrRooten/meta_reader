use std::{collections::HashMap, fs, io::Write, ops::Range, rc::Rc};

use bytes::{Buf, Bytes};

use crate::utils::{
    file::MRFile,
    funcs::{i_to_m, m_to_i},
    MRError,
};

use super::{DataDescriptor, FileItem, MFTEntry, MFTValue, Ntfs};

impl Ntfs {
    pub fn open(img: &str) -> Result<Ntfs, MRError> {
        let mr_file = MRFile::new(img);
        let mr_file = match mr_file {
            Ok(file) => file,
            Err(e) => {
                return Err(MRError::from(Box::new(e)));
            }
        };
        let header = mr_file.read_n(0, 512).unwrap();
        let bep = header[0..3].to_vec();
        let signature = header[3..11].to_vec();
        let signature = String::from_utf8_lossy(&signature).to_string();
        let is_bitlocker: bool;
        if signature.eq("-FVE-FS-") == true {
            is_bitlocker = true;
        } else {
            is_bitlocker = false;
        }
        if signature.starts_with("NTFS") == false && is_bitlocker {
            return Err(MRError::new("Not a valid NTFS image"));
        }

        let header = Bytes::from(header);
        let bytes_per_sector = (&header[11..13]).get_u16_le();
        let sectors_per_cluster_block = (&header[13..14]).get_u8();

        let total_sectors = (&header[40..48]).get_u64_le();
        let mft_cluster_block_number = (&header[48..56]).get_u64_le();
        let mft_mirror_cluster_block_number = (&header[56..64]).get_u64_le();
        let mft_entry_size = (&header[64..65]).get_u8();
        let index_entry_size = (&header[68..69]).get_u8();

        let start_with = (&header[0..12]).to_vec();
        let cluster_size = bytes_per_sector as u64 * sectors_per_cluster_block as u64;
        let offset = mft_cluster_block_number * cluster_size;
        let s: usize;
        if mft_entry_size <= 127 {
            s = mft_entry_size as usize;
        } else {
            s = 256 - mft_entry_size as usize;
        }

        let mft_entry_size = num::pow(2, s);

        Ok(Self {
            reader: mr_file,
            start_with: start_with,
            boot_entry_point: bep,
            sectors_per_cluster_block,
            bytes_per_sector,
            total_sectors,
            mft_block_number: mft_cluster_block_number,
            mft_mirror_block_number: mft_mirror_cluster_block_number,
            mft_entry_size,
            index_entry_size: index_entry_size,
            is_bitlocker,
            version: None,
            datas_of_mft: vec![],
            cache_mfts: None,
        })
    }

    pub fn get_datas_of_mft(&mut self) -> &Vec<DataDescriptor> {
        if self.datas_of_mft.len() != 0 {
            return &self.datas_of_mft;
        }

        let offset = self.get_mft_offset() as usize;
        let bs = self.reader.read_n(offset, self.get_mft_size()).unwrap();
        // let mut f = fs::File::create("./target/test.bin").unwrap();
        // f.write_all(&bs);
        let mft = MFTEntry::parse(Bytes::from(bs), self, offset as u64, 0).unwrap();
        let data_values = mft.map_attr_chains.get(&0x80).unwrap().first().unwrap();
        let _t = &data_values.value;
        if let MFTValue::Data(data) = _t {
            self.datas_of_mft = data.datas.clone();
        }
        &self.datas_of_mft
    }

    

    

    pub fn get_mft_by_path(&mut self, path: &str) -> Result<MFTEntry, MRError> {
        let ps = path.split("\\").collect::<Vec<&str>>()[1..].to_vec();
        let root_mft = match self.get_root_mft() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        let mut next = root_mft;
        for p in ps {
            if p.eq("") {
                continue;
            }
            let subs = next.get_sub_files().unwrap();
            let mut find = false;
            for sub in subs {
                if sub.get_name().eq_ignore_ascii_case(p) {
                    let mft = match self.get_mft_entry_by_index(sub.get_index()) {
                        Some(s) => s,
                        None => {
                            return Err(MRError::new("No such a file in mft"));
                        }
                    };
                    next = mft;
                    find = true;
                    break;
                }
            }

            if find == false {
                return Err(MRError::new("No such a file in directory"));
            }
        }

        return Ok(next);
    }

    pub fn get_cluster_size(&self) -> u64 {
        self.bytes_per_sector as u64 * self.sectors_per_cluster_block as u64
    }

    pub fn get_reader(&self) -> &MRFile {
        &self.reader
    }

    pub fn get_sector_num(&self) -> u64 {
        self.total_sectors
    }

    pub fn get_sector_bytes_num(&self) -> u64 {
        self.bytes_per_sector as u64
    }

    fn align(n: usize, alignment: usize) -> usize {
        (alignment - n % alignment) + n
    }
    pub fn iter_diy_block<F>(&self, size: usize, redundancy: usize, cores: u32, mut f: F)
    where
        F: FnMut(u64, u64, Vec<u8>) -> bool,
    {
        let redundancy = Ntfs::align(redundancy, 512);
        let c_sectors = self.total_sectors;
        let c_bytes = c_sectors * self.bytes_per_sector as u64;
        let mut offset = 0x1000;
        let mut _i = 0;
        while offset < c_bytes {
            let bs = match self.reader.read_n(offset as usize, size + redundancy) {
                Ok(o) => o,
                Err(e) => {
                    break;
                }
            };
            let diy_block_id = offset / size as u64;
            let is_break = f(diy_block_id, offset, bs);
            if is_break {
                break;
            }
            offset += size as u64;
            _i += 1;
        }
    }

    pub fn iter_mft<F>(&self, mut f: F)
    where
        F: FnMut(u64, Result<MFTEntry, MRError>, bool, &mut Ntfs),
    {
        let mut index = 0;
        let mft_size = self.get_mft_size();
        let datas = i_to_m(self).get_datas_of_mft();
        let mut is_deleted = false;
        let reader = self.get_reader();
        for data in datas {
            let d = data.datasize as usize;

            let block = self.get_mft_size() * 0x100;
            let start = data.start_addr as usize;
            let mut i = 0;
            while i < d {
                let mfts_bs = match self.get_reader().read_n(start + i, block) {
                    Ok(o) => o,
                    Err(e) => {
                        break;
                    }
                };
                if mfts_bs[0] == 0 {
                    break;
                }
                let mfts_bs = Bytes::from(mfts_bs);
                let mut offset = 0;
                while offset < block {
                    let mft_bs = mfts_bs.slice(offset..offset + 0x400);
                    if mft_bs[0] == 0 {
                        index += 1;
                        offset += self.get_mft_size();
                        continue;
                    }
                    if mft_bs[22] == 0 {
                        is_deleted = true;
                    } else {
                        is_deleted = false;
                    }
                    let entry = MFTEntry::parse(
                        Bytes::from(mft_bs),
                        i_to_m(self),
                        (start + i + offset) as u64,
                        index,
                    );
                    f(index, entry, is_deleted, i_to_m(self));
                    index += 1;
                    offset += self.get_mft_size();
                }
                i += block;
            }
        }
    }

    pub fn get_mft_offset(&self) -> u64 {
        self.mft_block_number * self.get_cluster_size()
    }

    pub fn get_mft_size(&self) -> usize {
        self.mft_entry_size
    }

    pub fn get_mft_entry_by_index(&mut self, index: u64) -> Option<MFTEntry> {
        let mut _index = index;
        let mft_size = self.get_mft_size();
        let datas = self.get_datas_of_mft();
        for data in datas {
            let mft_cap = data.datasize / mft_size as u64;
            if _index > mft_cap {
                _index -= mft_cap;
                continue;
            }

            let offset = data.start_addr as usize + _index as usize * self.get_mft_size();

            let mft_bs = match self.reader.read_n(offset, self.get_mft_size()) {
                Ok(o) => o,
                Err(e) => {
                    //eprintln!("Index {},Offset {}: {:?}", index, offset ,e);
                    return None;
                }
            };
            let entry = MFTEntry::parse(Bytes::from(mft_bs), self, offset as u64, index);
            match entry {
                Ok(o) => {
                    return Some(o);
                }
                Err(e) => return None,
            }
        }
        return None;
    }

    pub fn get_root_mft(&mut self) -> Result<MFTEntry, MRError> {
        let root_mft = self.get_mft_entry_by_index(5).unwrap();
        Ok(root_mft)
    }

    pub fn read_raw(&mut self, range: Range<usize>) -> Result<Vec<u8>, MRError> {
        self.reader.read_range(range)
    }

    pub fn get_version(&mut self) -> Option<(u8, u8)> {
        if self.version.is_some() {
            return self.version;
        }

        if self.datas_of_mft.len() == 0 {
            return None;
        }
        let volume_index = 3;
        let volume = self.get_mft_entry_by_index(volume_index).unwrap();
        let chains = volume.map_attr_chains.get(&0x70).unwrap();
        let volume_info = &chains.first().unwrap().value;
        if let MFTValue::VolumeInfo(s) = volume_info {
            self.version = Some((s.majar_version, s.minor_version));
            self.version
        } else {
            None
        }
    }

    pub fn get_sub_nodes(&self, mft: &MFTEntry) -> Result<Vec<FileItem>, MRError> {
        mft.get_sub_files()
    }

    pub fn is_deleted_by_index(&self, index: u64) -> bool {
        let mut _index = index;
        let mft_size = self.get_mft_size();
        let datas = i_to_m(self).get_datas_of_mft();
        for data in datas {
            let mft_cap = data.datasize / mft_size as u64;
            if _index > mft_cap {
                _index -= mft_cap;
                continue;
            }

            let offset = data.start_addr as usize + _index as usize * self.get_mft_size();

            let mft_bs = self.reader.read_n(offset, 40).unwrap();
            if mft_bs[22] == 0 && mft_bs[23] == 0 {
                return true;
            }
        }
        return false;
    }
}
