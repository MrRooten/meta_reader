use std::{cell::RefCell, collections::HashMap, fs, io::Write, mem::align_of, panic};

use bytes::{Buf, Bytes};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

use crate::utils::{funcs::sub_bytes, MRError};

use super::{
    CCommon, CNonResident, CResident, DataDescriptor, FileItem, FileReference, FileTime,
    IndexEntryHeader, IndexNodeHeader, IndexRootHeader, IndexValue, MFTAttribute, MFTEntry,
    MFTStream, MFTValue, Ntfs, V20Attr, Value10_StandardInfomation, Value20_AttributeList,
    Value30_FileName, Value40_ObjectId, Value50_SecurityDescriptor, Value60_VolumeName,
    Value70_VolumeInfomation, Value80_Data, Value90_IndexRoot, ValueA0_IndexAlloction,
};

pub fn long_to_short(name: &str, names: Vec<&str>) -> String {
    unimplemented!()
}

fn align(n: usize, alignment: usize) -> usize {
    (alignment - n % alignment) + n
}

impl MFTEntry {
    pub fn filename(&self) -> Option<String> {
        let attr = match self.map_attr_chains.get(&0x30) {
            Some(a) => a,
            None => {
                return None;
            }
        };
        let attr = match attr.first() {
            Some(s) => s,
            None => {
                return None;
            }
        };
        if let MFTValue::FileName(s) = &attr.value {
            return Some(s.name.to_string());
        }
        None
    }

    pub fn get_stream(&self, name: &str) -> Option<&Value80_Data> {
        let attrs = match self.map_attr_chains.get(&0x80) {
            Some(o) => o,
            None => {
                return None;
            }
        };

        for attr in attrs {
            if attr.attr_name.eq(name) {
                if let MFTValue::Data(d) = &attr.value {
                    return Some(d);
                }
            }
        }
        None
    }

    pub fn get_streams_list(&self) -> Option<Vec<String>> {
        let attrs = match self.map_attr_chains.get(&0x80) {
            Some(o) => o,
            None => {
                return None;
            }
        };

        let mut result = vec![];
        for attr in attrs {
            result.push(attr.attr_name.to_string());
        }

        Some(result)
    }

    pub fn get_flags(&self) -> u16 {
        self.entry_flags
    }

    pub fn get_index(&self) -> u64 {
        self.index
    }

    pub fn get_parent_index(&self) -> i64 {
        if *self.parent_index.borrow() > 0 {
            return *self.parent_index.borrow();
        }

        let attr = self.map_attr_chains.get(&0x30).unwrap().first().unwrap();
        if let MFTValue::FileName(s) = &attr.value {
            self.parent_index.replace(s.parent_file_num as i64);
        }
        *self.parent_index.borrow()
    }

    pub fn contains_attr(&self, key: u32) -> bool {
        self.map_attr_chains.contains_key(&key)
    }

    pub fn fullpath(&self) -> Option<String> {
        let mut index = 0;
        if self.index <= 13 {
            return Some(self.filename().unwrap());
        }
        let mut mft: Option<MFTEntry> = None;
        let mut names = vec![];
        let filename = match self.filename() {
            Some(s) => s,
            None => {
                names.reverse();
                return Some(format!("<{}>:{}", self.get_index(), names.join("\\")));
            }
        };
        names.push(filename);
        let ntfs = self.get_ntfs();
        let parent_index = self.get_parent_index();
        if parent_index < 5 {
            names.reverse();
            return Some(names.join("\\"));
        }
        index = parent_index;

        while index > 13 {
            let mft = ntfs.get_mft_entry_by_index(index as u64);
            let mft = match mft {
                Some(s) => s,
                None => {
                    names.reverse();
                    return Some(format!("<{}>:{}", index, names.join("\\")));
                }
            };
            let filename = match mft.filename() {
                Some(s) => s,
                None => {
                    names.reverse();
                    return Some(format!("<{}>:{}", index, names.join("\\")));
                }
            };

            names.push(filename);
            index = mft.get_parent_index();
        }
        names.reverse();
        Some(names.join("\\"))
    }

    pub fn filename_creation_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x30).unwrap().first().unwrap();
        if let MFTValue::FileName(s) = &attr.value {
            let time = &s.create_time;
            let time = match time.to_native_date() {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(time);
        }
        None
    }

    pub fn filename_change_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x30).unwrap().first().unwrap();
        if let MFTValue::FileName(s) = &attr.value {
            let time = &s.change_time;
            let time = match time.to_native_date() {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(time);
        }
        None
    }

    pub fn get_data_value(&self) -> Option<&Value80_Data> {
        let attr = match self.map_attr_chains.get(&0x80) {
            Some(o) => o.first().unwrap(),
            None => {
                return None;
            }
        };
        if let MFTValue::Data(s) = &attr.value {
            return Some(s);
        }
        None
    }

    pub fn filename_access_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x30).unwrap().first().unwrap();
        if let MFTValue::FileName(s) = &attr.value {
            let time = &s.last_visit_time;
            let time = match time.to_native_date() {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(time);
        }
        None
    }

    pub fn get_creation_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x10).unwrap().first().unwrap();
        if let MFTValue::StdInfo(s) = &attr.value {
            if s.file_create_time.low == 0 && s.file_create_time.high == 0 {
                return None;
            }
            let time = &s.file_create_time.to_native_date();
            let time = match time {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(*time);
        }
        None
    }

    pub fn get_change_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x10).unwrap().first().unwrap();
        if let MFTValue::StdInfo(s) = &attr.value {
            if s.file_change_time.low == 0 && s.file_change_time.high == 0 {
                return None;
            }
            let time = &s.file_change_time.to_native_date();
            let time = match time {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(*time);
        }
        None
    }

    pub fn get_access_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x10).unwrap().first().unwrap();
        if let MFTValue::StdInfo(s) = &attr.value {
            if s.file_last_visited.low == 0 && s.file_last_visited.high == 0 {
                return None;
            }
            let time = &s.file_last_visited.to_native_date();
            let time = match time {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(*time);
        }
        None
    }

    pub fn get_mft_change_time(&self) -> Option<DateTime<Local>> {
        let attr = self.map_attr_chains.get(&0x10).unwrap().first().unwrap();
        if let MFTValue::StdInfo(s) = &attr.value {
            if s.mft_change_time.low == 0 && s.mft_change_time.high == 0 {
                return None;
            }
            let time = &s.mft_change_time.to_native_date();
            let time = match time {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(*time);
        }
        None
    }

    pub fn get_filesize(&self) -> Option<usize> {
        let attr = self.map_attr_chains.get(&0x80).unwrap().first().unwrap();
        if let MFTValue::Data(info) = &attr.value {
            return Some(attr.common.get_data_size());
        }

        None
    }

    pub fn get_data(&self) -> Bytes {
        unimplemented!()
    }

    pub fn is_sparse(&self) -> bool {
        unimplemented!()
    }

    pub fn is_compress(&self) -> bool {
        false
    }

    fn min(self, n1: &usize, n2: &usize) -> usize {
        unimplemented!()
    }

    fn get_ntfs(&self) -> &Ntfs{
        unsafe { &*self.ntfs.unwrap() }
    }

    pub fn read_n_in_stream(
        &self,
        addr: usize,
        n: usize,
        stream: &str,
    ) -> Result<Vec<u8>, MRError> {
        let datas = match self.get_stream(stream) {
            Some(s) => s,
            None => {
                return Err(MRError::new("Not found stream"));
            }
        };
        let mut result = Vec::new();
        let real_n = n;
        let mut last_n = real_n as u64;
        let mut last_addr = addr as u64;
        let ntfs = self.get_ntfs();
        for data in &datas.datas {
            if last_addr > data.datasize {
                last_addr -= data.datasize;
                continue;
            }
            
            let read_size = {
                if n < data.datasize as usize {
                    n
                } else {
                    data.datasize as usize
                }
            };

            let __offset = data.start_addr % 512;
            let start_addr = data.start_addr - __offset;
            let tmp_data = ntfs
                .reader
                .read_n(start_addr as usize, __offset as usize + read_size)
                .unwrap();
            let buffer_data: Vec<u8> = tmp_data[__offset as usize..].to_vec();

            if last_addr < buffer_data.len() as u64 && last_n > buffer_data.len() as u64 - last_addr
            {
                // let bs = ntfs
                //     .reader
                //     .read_n(
                //         (data.start_addr + last_addr) as usize,
                //         (data.datasize - last_addr) as usize,
                //     )
                //     .unwrap();
                let bs = buffer_data[(last_addr) as usize..(data.datasize) as usize].to_vec();
                result.extend(bs);
                last_n -= data.datasize - last_addr;
                last_addr = 0;

                continue;
            }

            if last_addr < buffer_data.len() as u64
                && last_n <= buffer_data.len() as u64 - last_addr
            {
                // let bs = ntfs
                //     .reader
                //     .read_n((data.start_addr + last_addr) as usize, (last_n) as usize)
                //     .unwrap();
                let bs = buffer_data[(last_addr) as usize..(last_addr + last_n) as usize].to_vec();
                result.extend(bs);
                break;
            }
        }

        Ok(result)
    }

    pub fn read_n(&self, addr: usize, n: usize) -> Result<Vec<u8>, MRError> {
        let attrs = self.map_attr_chains.get(&0x80).unwrap();
        let mut result = Vec::new();
        let real_n = n;
        let mut last_n = real_n as u64;
        let mut last_addr = addr as u64;
        let ntfs = self.get_ntfs();

        if let Some(_t) = attrs.iter().next() {
            if let MFTValue::Data(datas) = &_t.value {
                for data in &datas.datas {
                    if last_addr > data.datasize {
                        last_addr -= data.datasize;
                        continue;
                    }

                    let read_size = {
                        if n < data.datasize as usize {
                            n
                        } else {
                            data.datasize as usize
                        }
                    };

                    let __offset = data.start_addr % 512;
                    let start_addr = data.start_addr - __offset;
                    let tmp_data = ntfs
                        .reader
                        .read_n(start_addr as usize, __offset as usize + read_size)
                        .unwrap();
                    let buffer_data = tmp_data[__offset as usize..].to_vec();

                    if last_addr < buffer_data.len() as u64
                        && last_n > buffer_data.len() as u64 - last_addr
                    {
                        // let bs = ntfs
                        //     .reader
                        //     .read_n(
                        //         (data.start_addr + last_addr) as usize,
                        //         (data.datasize - last_addr) as usize,
                        //     )
                        //     .unwrap();
                        let bs =
                            buffer_data[(last_addr) as usize..(data.datasize) as usize].to_vec();
                        result.extend(bs);
                        last_n -= data.datasize - last_addr;
                        last_addr = 0;

                        continue;
                    }

                    if last_addr < buffer_data.len() as u64
                        && last_n <= buffer_data.len() as u64 - last_addr
                    {
                        // let bs = ntfs
                        //     .reader
                        //     .read_n((data.start_addr + last_addr) as usize, (last_n) as usize)
                        //     .unwrap();
                        let bs = buffer_data[(last_addr) as usize..(last_addr + last_n) as usize]
                            .to_vec();
                        result.extend(bs);
                        break;
                    }
                }
            }
        }
        Ok(result)
    }

    pub fn parse(
        bs: Bytes,
        ntfs: &Ntfs,
        mft_base: u64,
        index: u64,
    ) -> Result<MFTEntry, MRError> {
        let sig = String::from_utf8_lossy(sub_bytes(&bs,0..4)?);
        if !sig.eq("BAAD") && !sig.eq("FILE") {
            return Err(MRError::new("Not a valid MFT entry"));
        }

        let fix_up_value_offset = (sub_bytes(&bs,4..6)?).get_u16_le();
        let number_fix_up_values = (sub_bytes(&bs,6..8)?).get_u16_le();
        let journal_sequence_number = (sub_bytes(&bs,8..16)?).get_u64_le();
        let sequence = (sub_bytes(&bs,16..18)?).get_u16_le();
        let reference_count = (sub_bytes(&bs,18..20)?).get_u16_le();
        let attributes_offset = (sub_bytes(&bs,20..22)?).get_u16_le();
        let entry_flags = (sub_bytes(&bs,22..24)?).get_u16_le();
        let used_size = (sub_bytes(&bs,24..28)?).get_u32_le();
        let total_size = (sub_bytes(&bs,28..32)?).get_u32_le();

        let mut map_attr_chains: HashMap<u32, Vec<MFTAttribute>> = HashMap::new();
        let mut base_addr = 56;
        let len = bs.len();

        while base_addr < len - 1 {
            let _attr_len = (sub_bytes(&bs,base_addr + 4..base_addr + 8)?).get_u16_le();
            let attr = match MFTAttribute::parse(
                &bs,
                ntfs,
                base_addr as u64,
                index,
                base_addr as u64 + mft_base,
            ) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };
            base_addr += attr.length as usize;
            if map_attr_chains.contains_key(&attr.mft_type) {
                let chains = match map_attr_chains.get_mut(&attr.mft_type) {
                    Some(s) => s,
                    None => {
                        continue;
                    }
                };
                chains.push(attr);
            } else {
                let mut chains = Vec::new();
                let mft_type = attr.mft_type;
                chains.push(attr);
                map_attr_chains.insert(mft_type, chains);
            }

            if base_addr >= len - 1 {
                break;
            }
            if bs[base_addr] == 0xff && bs[base_addr + 1] == 0xff {
                break;
            }
        }
        Ok(Self {
            fix_up_value_offset,
            number_fix_up_values,
            journal_sequence_number,
            sequence,
            reference_count,
            attributes_offset,
            entry_flags,
            used_size,
            total_size,
            map_attr_chains,
            ntfs: Some(ntfs),
            index,
            parent_index: RefCell::new(-1),
        })
    }

    pub fn is_dir(&self) -> bool {
        self.map_attr_chains.contains_key(&0x90)
    }

    pub fn get_sub_files(&self) -> Result<Vec<FileItem>, MRError> {
        let mut result = Vec::new();

        if let Some(indexs_a0) = self.map_attr_chains.get(&0xa0) {
            for index in indexs_a0 {
                if let MFTValue::IndexAlloc(is) = &index.value {
                    is.init_value();
                    let v = is.values.borrow();
                    let vs = match &*v {
                        Some(s) => s,
                        None => {
                            continue;
                        }
                    };

                    for v in vs {
                        let name = match &v.index_key_data {
                            Some(s) => s,
                            None => {
                                continue;
                            }
                        };
                        result.push(FileItem {
                            mft_index: v.file_reference.mft_index,
                            name: name.clone(),
                        });
                    }
                }
            }
        }

        if let Some(indexs_90) = self.map_attr_chains.get(&0x90) {
            for index in indexs_90 {
                if let MFTValue::IndexRoot(ir) = &index.value {
                    let vs = &ir.values;
                    for v in vs {
                        let name = match &v.index_key_data {
                            Some(s) => s,
                            None => {
                                continue;
                            }
                        };
                        result.push(FileItem {
                            mft_index: v.file_reference.mft_index,
                            name: name.clone(),
                        });
                    }
                }
            }
        }
        Ok(result)
    }
}

impl FileItem {
    pub fn get_name(&self) -> &String {
        &self.name.name
    }

    pub fn get_index(&self) -> u64 {
        self.mft_index
    }
}

pub fn vec_u8_to_utf16string(bytes: &[u8]) -> String {
    let title: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|a| u16::from_ne_bytes([a[0], a[1]]))
        .collect();
    let title = title.as_slice();
    
    String::from_utf16_lossy(title)
}

impl TryInto<Value10_StandardInfomation> for MFTValue {
    type Error = MRError;

    fn try_into(self) -> Result<Value10_StandardInfomation, Self::Error> {
        if let MFTValue::StdInfo(s) = self {
            Ok(s)
        } else {
            Err(MRError::new("Not StandardInformation"))
        }
    }
}

impl MFTValue {
    pub fn parse(
        attr_type: u32,
        bs: Bytes,
        ntfs: &Ntfs,
        index: u64,
        is_nonresident: bool,
        base: u64,
        common: &CCommon,
    ) -> Result<MFTValue, MRError> {
        if attr_type == 0x10 {
            let info = match Value10_StandardInfomation::parse(bs, ntfs, index) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };

            Ok(MFTValue::StdInfo(info))
        } else if attr_type == 0x20 {
            let attrlist = match Value20_AttributeList::parse(bs, ntfs, is_nonresident) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };

            return Ok(MFTValue::AttrList(attrlist));
        } else if attr_type == 0x30 {
            let name = match Value30_FileName::parse(bs) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };
            return Ok(MFTValue::FileName(name));
        } else if attr_type == 0x80 {
            let data = match Value80_Data::parse(index, bs, ntfs, is_nonresident, base, common) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };

            return Ok(MFTValue::Data(data));
        } else if attr_type == 0x40 {
            return Ok(MFTValue::ObjectId(Value40_ObjectId::parse(bs, ntfs)));
        } else if attr_type == 0x70 {
            return Ok(MFTValue::VolumeInfo(Value70_VolumeInfomation::parse(
                bs, ntfs,
            )));
        } else if attr_type == 0x90 {
            let vs = bs.to_vec();
            let value = match Value90_IndexRoot::parse(bs, ntfs) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };
            return Ok(MFTValue::IndexRoot(value));
        } else if attr_type == 0xa0 {
            return Ok(MFTValue::IndexAlloc(
                ValueA0_IndexAlloction::new(bs, ntfs, is_nonresident).unwrap(),
            ));
        } else {
            return Ok(MFTValue::None);
        }
    }
}

impl FileTime {
    pub fn parse_from_u64(s: u64) -> Self {
        let low = ((s >> 32) as u32).swap_bytes();
        let high = ((s % (2 << 32)) as u32).swap_bytes();
        Self {
            low,
            high,
        }
    }
    fn to_seconds(&self, t: u64) -> Option<u64> {
        let s = t / 10000000;
        if s < 11644473600 {
            return None;
        }
        Some(s - 11644473600)
    }

    pub fn get_timestamp(&self) -> u64 {
        let t = (self.high as u64) * num::pow(2_u64, 32) as u64 + self.low as u64;
        //NaiveDateTime::from_timestamp_opt(self.to_seconds(t) as i64, 0).unwrap();
        
        self.to_seconds(t).unwrap()
    }
    pub fn to_native_date(&self) -> Option<DateTime<Local>> {
        let t = (self.high as u64) * num::pow(2_u64, 32) as u64 + self.low as u64;
        //NaiveDateTime::from_timestamp_opt(self.to_seconds(t) as i64, 0).unwrap();
        let timestamp = match self.to_seconds(t) {
            Some(s) => s,
            None => {
                return None;
            }
        };
        let datetime_utc = match Utc.timestamp_opt(timestamp as i64, 0) {
            chrono::LocalResult::None => return None,
            chrono::LocalResult::Single(s) => s,
            chrono::LocalResult::Ambiguous(_, _) => return None,
        };
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let datetime_local = match std::panic::catch_unwind(|| datetime_utc.with_timezone(&chrono::Local)) {
            Ok(o) => o,
            Err(_) =>  {
                return None;
            },
        };
        panic::set_hook(prev_hook);
        Some(datetime_local)
    }
}

impl Value10_StandardInfomation {
    pub fn parse(bs: Bytes, ntfs: &Ntfs, index: u64) -> Result<Self, MRError> {
        let file_create_time = FileTime::parse_from_u64((sub_bytes(&bs,0..8)?).get_u64());
        let file_change_time = FileTime::parse_from_u64((sub_bytes(&bs,8..16)?).get_u64());
        let mft_change_time = FileTime::parse_from_u64((sub_bytes(&bs,16..24)?).get_u64());
        let file_last_visited = FileTime::parse_from_u64((sub_bytes(&bs,24..32)?).get_u64());
        let file_attr = (sub_bytes(&bs,32..36)?).get_u32_le();
        // if index != 3 {
        //     //Not $Volume file
        //     let binding = ntfs.get_version();
        //     let version = match &binding {
        //         Some(s) => s,
        //         None => {
        //             return Err(MRError::new("error in get version of ntfs"));
        //         }
        //     };
        //     if version.0 >= 3 && bs.len() > 48 {
        //         let owner_id = Some((sub_bytes(&bs,48..52)?).get_u32_le());
        //         let security_id = Some((sub_bytes(&bs,52..56)?).get_u32_le());
        //         let quota_charged = Some((sub_bytes(&bs,56..64)?).get_u64_le());
        //         let update_sequence_num = Some((sub_bytes(&bs,64..72)?).get_u64_le());
        //         return Ok(Self {
        //             file_create_time,
        //             file_change_time,
        //             mft_change_time,
        //             file_last_visited,
        //             file_attr_flags: Some(file_attr),
        //             owner_id,
        //             security_id,
        //             quota_charged,
        //             update_sequence_num,
        //         });
        //     }
        // }
        Ok(Self {
            file_create_time,
            file_change_time,
            mft_change_time,
            file_last_visited,
            file_attr_flags: None,
            owner_id: None,
            security_id: None,
            quota_charged: None,
            update_sequence_num: None,
        })
    }
}

impl Value20_AttributeList {
    pub fn init(&mut self) -> bool {
        true
    }

    pub fn parse(bs: Bytes, ntfs: &Ntfs, is_nonresident: bool) -> Result<Self, MRError> {
        if !is_nonresident {
            let mut i = 0;
            let mut list = vec![];
            while i < bs.len() {
                let attribute_type = (sub_bytes(&bs,i..i + 4)?).get_u32_le();
                let size = (sub_bytes(&bs,i + 4..i + 6)?).get_u16_le();
                let name_size = (sub_bytes(&bs,i + 6..i + 7)?).get_u8();
                let name_offset = (sub_bytes(&bs,i + 7..i + 8)?).get_u8();
                let data_vcn = (sub_bytes(&bs,i + 8..i + 16)?).get_u64_le();
                let file_reference = FileReference::parse(bs.slice(i + 16..i + 24));
                let attribute_identifier = (sub_bytes(&bs,i + 24..i + 26)?).get_u16_le();
                if i + name_offset as usize + 2 * name_size as usize > bs.len() {
                    i += size as usize;
                    let v20 = V20Attr {
                        attribute_type,
                        size,
                        name_size,
                        name_offset,
                        data_vcn,
                        file_reference,
                        attribute_identifier,
                        name: String::new(),
                    };
                    list.push(v20);
                    continue;
                }
                let name = vec_u8_to_utf16string(
                    &bs.slice(
                        i + name_offset as usize..i + name_offset as usize + 2 * name_size as usize,
                    ),
                );
                i += size as usize;
                let v20 = V20Attr {
                    attribute_type,
                    size,
                    name_size,
                    name_offset,
                    data_vcn,
                    file_reference,
                    attribute_identifier,
                    name,
                };
                list.push(v20);
            }

            return Ok(Self { list: Some(list) });
        }

        Ok(Self { list: None })
    }
}

impl Value30_FileName {
    pub fn parse(bs: Bytes) -> Result<Self, MRError> {
        if bs.len() < 66 {
            return Err(MRError::new("error format filename length"));
        }
        //let parent_file_num = (sub_bytes(&bs,0..8)?).get_u64_le();
        let parent_file_num = get_le_u64(bs.slice(0..6)).unwrap();
        let creation_date = FileTime::parse_from_u64((sub_bytes(&bs,8..16)?).get_u64());
        let last_modify_time = FileTime::parse_from_u64((sub_bytes(&bs,16..24)?).get_u64());
        let mft_change_time = FileTime::parse_from_u64((sub_bytes(&bs,24..32)?).get_u64());
        let last_visit_time = FileTime::parse_from_u64((sub_bytes(&bs,32..40)?).get_u64());
        let alloc_size = (sub_bytes(&bs,40..48)?).get_u64_le();
        let file_size = (sub_bytes(&bs,48..56)?).get_u64_le();
        let file_attr_flags = (sub_bytes(&bs,56..60)?).get_u32_le();
        let extended_flags = (sub_bytes(&bs,60..64)?).get_u32_le();
        let name_length = (sub_bytes(&bs,64..65)?).get_u8();
        let name_space = (sub_bytes(&bs,65..66)?).get_u8();
        if bs.len() < 66 + (name_length as usize) * 2 {
            return Err(MRError::new("error format"));
        }
        let name_vec = bs.slice(66..66 + (name_length as usize) * 2).to_vec();
        let name = vec_u8_to_utf16string(&name_vec);
        Ok(Self {
            parent_file_num,
            create_time: creation_date,
            change_time: last_modify_time,
            mft_change_time,
            last_visit_time,
            alloc_size,
            real_size: file_size,
            file_flag: file_attr_flags,
            ea_flag: file_attr_flags,
            name_length,
            name_space,
            name,
        })
    }
}

pub fn get_le_u64(bs: Bytes) -> Option<u64> {
    if bs.len() > 8 {
        return None;
    }
    let mut result = 0;
    let mut count = 0;
    for b in bs {
        let k = b as u64;
        result += (k << count);
        count += 8;
    }
    Some(result)
}

impl Value40_ObjectId {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Self {
        Self {
            droid_file_identify: 0,
            birth_droid_vol_identify: 0,
            birth_droid_file_identify: 0,
            birth_droid_domain_identify: 0,
        }
    }
}

impl Value50_SecurityDescriptor {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Self {
        unimplemented!()
    }
}

impl Value60_VolumeName {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Self {
        unimplemented!()
    }
}

impl Value70_VolumeInfomation {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Self {
        let major = (sub_bytes(&bs,8..9).unwrap()).get_u8();
        let minor = (sub_bytes(&bs,9..10).unwrap()).get_u8();
        let flags = (sub_bytes(&bs,10..12).unwrap()).get_u16_le();
        Self {
            majar_version: major,
            minor_version: minor,
            volume_flags: flags,
        }
    }
}

impl Value80_Data {
    pub fn get_datas(&self) -> &Vec<DataDescriptor> {
        &self.datas
    }
    pub fn parse(
        index: u64,
        bs: Bytes,
        ntfs: &Ntfs,
        is_nonresident: bool,
        base: u64,
        common: &CCommon,
    ) -> Result<Self, MRError> {
        if !is_nonresident {
            let offset = common.get_data_offset() as u64;
            let filesize = common.get_data_size() as u64;

            return Ok(Self {
                datas: vec![DataDescriptor {
                    datasize: filesize,
                    start_addr: base + offset,
                }],
            });
        }

        let mut index = 0;
        let mut result = vec![];
        let mut cluster_number = 0;
        while index < bs.len() {
            let len = (sub_bytes(&bs,index..1 + index)?).get_u8();
            let filesize_len = len % 16;
            let start_addr_len = len / 16;
            if filesize_len == 0 && start_addr_len == 0{
                break;
            }

            if filesize_len > 6 || start_addr_len > 6 {
                break;
            }
            if index + 1 + filesize_len as usize > bs.len() {
                break;
            }
            let filesize = match get_le_u64(bs.slice(index + 1..index + 1 + filesize_len as usize))
            {
                Some(s) => s,
                None => {
                    //break;
                    fs::write("./error_data80", &bs);
                    return Err(MRError::new(
                        "too many bytes in Value80_Data::new get filesize",
                    ));
                }
            };

            if start_addr_len == 0 {
                index = index + 1 + filesize as usize;
                continue;
            }

            let _s = (index + 1 + filesize_len as usize);
            if _s + start_addr_len as usize > bs.len() {
                break;
            }
            let mut offset = match get_le_u64(bs.slice(_s.._s + start_addr_len as usize)) {
                Some(o) => o,
                None => {
                    break
                }
            };

            if offset == 0 {
                //println!("fuck is zero: {:?}", bs.slice(_s.._s + start_addr_len as usize).to_vec());
            }

            index += filesize_len as usize + start_addr_len as usize + 1;

            if ((start_addr_len != 0) && (offset & (1 << (start_addr_len * 8 - 1))) != 0) {
                let mut i = start_addr_len;
                while i < 8 {
                    offset |= 0xff << (i * 8);
                    i += 1;
                }
                let _t: u128 = cluster_number as u128 + offset as u128;
                cluster_number = (0xffffffffffffffff & _t) as u64;

                
            } else {
                
                cluster_number += offset;

                
            }
            if filesize.checked_mul(ntfs.get_cluster_size()).is_none() {
                break
            }
            let data = DataDescriptor {
                datasize: filesize * ntfs.get_cluster_size(),
                start_addr: offset,
            };
            

            if cluster_number
                .checked_mul(ntfs.get_cluster_size())
                .is_none()
            {
                break;
            }
            let data = DataDescriptor {
                datasize: data.datasize,
                start_addr: cluster_number * ntfs.get_cluster_size(),
            };
            // let _bs = ntfs.reader.read_n(data.start_addr as usize, 0x400).unwrap();
            //println!("{:?}", _bs);
            result.push(data);
        }

        Ok(Self { datas: result })
    }
}

impl IndexRootHeader {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Result<Self, MRError> {
        let attr_type = (sub_bytes(&bs,0..4)?).get_u32_le();
        let collation_type = (sub_bytes(&bs,4..8)?).get_u32_le();
        let entry_size = (sub_bytes(&bs,8..12)?).get_u32_le();
        let entry_num = (sub_bytes(&bs,12..16)?).get_u32_le();
        Ok(Self {
            attr_type,
            collation_type,
            index_entry_size: entry_size,
            index_entry_number_cluser: entry_num,
        })
    }
}

impl IndexEntryHeader {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Result<Self, MRError> {
        let fix_up_value_offset = (sub_bytes(&bs,4..6)?).get_u16_le();
        let number_of_fix_up_values = (sub_bytes(&bs,6..8)?).get_u16_le();
        let journal_sequence = (sub_bytes(&bs,8..16)?).get_u64_le();
        let vcn_of_index_entry = (sub_bytes(&bs,16..24)?).get_u64_le();
        Ok(Self {
            fix_up_value_offset,
            number_of_fix_up_values,
            journal_sequence,
            vcn_of_index_entry,
        })
    }
}

impl IndexNodeHeader {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Result<Self, MRError> {
        let index_values_offset = (sub_bytes(&bs,0..4)?).get_u32_le();
        let index_node_size = (sub_bytes(&bs,4..8)?).get_u32_le();
        let allocated_index_node_size = (sub_bytes(&bs,8..12)?).get_u32_le();
        let index_node_flags = (sub_bytes(&bs,12..16)?).get_u32_le();
        Ok(Self {
            index_values_offset,
            index_node_size,
            allocated_index_node_size,
            index_node_flags,
        })
    }
}

impl Value90_IndexRoot {
    pub fn parse(bs: Bytes, ntfs: &Ntfs) -> Result<Self, MRError> {
        let root_header = IndexRootHeader::parse(bs.slice(0..16), ntfs).unwrap();
        let node_header = IndexNodeHeader::parse(bs.slice(16..32), ntfs).unwrap();
        let value_offset = node_header.index_values_offset + 16;
        let mut index = 0;
        let mut values = Vec::new();

        while index < node_header.index_node_size as usize {
            //Not handle
            let size_offset = index + value_offset as usize + 8;
            let size = (sub_bytes(&bs,size_offset..size_offset + 4)?).get_u16_le();
            if size == 0 {
                break;
            }

            if node_header.index_node_size - (index as u32) <= 16 {
                break;
            }
            let offset = value_offset as usize + index;
            if offset + size as usize > bs.len() {
                break;
            }

            let value = match IndexValue::parse(bs.slice(offset..offset + size as usize)) {
                Ok(o) => o,
                Err(e) => {
                    return Err(e);
                }
            };
            let flags = value.index_value_flags;
            values.push(value);
            if flags & 0x2 == 0x2 {
                break;
            }

            index += size as usize;
        }
        Ok(Self {
            root_header,
            node_header,
            values,
        })
    }
}

impl FileReference {
    pub fn parse(bs: Bytes) -> Self {
        let index = get_le_u64(bs.slice(0..6)).unwrap();
        let sequence = get_le_u64(bs.slice(6..8)).unwrap();
        Self {
            mft_index: index,
            sequence_num: sequence as u16,
        }
    }
}

impl IndexValue {
    pub fn get_name(&self) -> Option<&String> {
        if let Some(s) = &self.index_key_data {
            return Some(&s.name);
        }

        None
    }

    pub fn parse(bs: Bytes) -> Result<IndexValue, MRError> {
        let file_reference = FileReference::parse(bs.slice(0..8));
        let index_value_size = (sub_bytes(&bs,8..10)?).get_u16_le();
        let index_key_data_size = (sub_bytes(&bs,10..12)?).get_u16_le();
        let index_value_flags = (sub_bytes(&bs,12..16)?).get_u32_le();
        let mut index_key_data: Option<Value30_FileName> = None;
        let mut index_value_data: Option<Vec<u8>> = None;
        let mut sub_node_vcn: Option<u64> = None;
        if index_key_data_size > 0 {
            if 16 + index_key_data_size as usize > bs.len() {
                return Err(MRError::new("Structure error"));
            }

            let filename =
                match Value30_FileName::parse(bs.slice(16..16 + (index_key_data_size as usize))) {
                    Ok(o) => o,
                    Err(e) => {
                        return Err(e);
                    }
                };
            index_key_data = Some(filename);
            let value_offset = 16 + index_key_data_size as usize;
            if index_value_flags & 0x1 == 0x1 {
                index_value_data = Some(bs.slice(value_offset..bs.len() - 8).to_vec());
            } else {
                index_value_data = Some(bs.slice(value_offset..).to_vec());
            }
        }
        Ok(Self {
            file_reference,
            index_value_size,
            index_key_data_size,
            index_value_flags,
            index_key_data,
            index_value_data,
            sub_node_vcn,
        })
    }
}

impl ValueA0_IndexAlloction {
    pub fn new(bs: Bytes, ntfs: &Ntfs, is_nonresident: bool) -> Result<Self, MRError> {
        if !is_nonresident {
            let entry_header = IndexEntryHeader::parse(bs.slice(0..24), ntfs).unwrap();
            let node_header = IndexNodeHeader::parse(bs.slice(24..40), ntfs).unwrap();
            let value_offset = node_header.index_values_offset + 24;
            let mut index = 0;
            let mut values = Vec::new();
            loop {
                if index < node_header.index_node_size as usize {
                    break;
                }
                let size_offset = index + value_offset as usize + 8;
                let size = (sub_bytes(&bs,size_offset..size_offset + 4)?).get_u32_le();
                if size == 0 {
                    break;
                }

                let offset = value_offset as usize + index;
                let value = IndexValue::parse(bs.slice(offset..offset + size as usize)).unwrap();
                values.push(value);
            }

            Ok(Self {
                offset: 0,
                size: 0,
                node_header: RefCell::new(Some(vec![node_header])),
                values: RefCell::new(Some(values)),
                ntfs: Some(ntfs),
                entry_header: RefCell::new(Some(vec![entry_header])),
            })
        } else {
            let _tmp = (sub_bytes(&bs,0..1)?).get_u8();
            let offset_size = (_tmp / 16) as usize;
            let size_size = (_tmp % 16) as usize;
            let size = match get_le_u64(bs.slice(1..1 + size_size)) {
                Some(s) => s,
                None => {
                    return Err(MRError::new(
                        "too many bytes in ValueA0_IndexAlloction::new get size",
                    ));
                }
            };
            let offset = match get_le_u64(bs.slice(1 + size_size..1 + size_size + offset_size)) {
                Some(s) => s,
                None => {
                    return Err(MRError::new(
                        "too many bytes in ValueA0_IndexAlloction::new get offset",
                    ));
                }
            };
            Ok(Self {
                offset,
                size,
                node_header: RefCell::new(None),
                values: RefCell::new(None),
                ntfs: Some(ntfs),
                entry_header: RefCell::new(None),
            })
        }
    }

    pub fn is_init(&self) -> bool {
        self.values.borrow().is_some()
    }

    fn get_ntfs(&self) -> &Ntfs {
        unsafe { &*self.ntfs.unwrap() }
    }

    pub fn init_value(&self) {
        if self.values.borrow().is_some() {
            return;
        }
        let ntfs = self.get_ntfs();
        let bs = Bytes::from(
            ntfs.reader
                .read_n(
                    (self.offset * ntfs.get_cluster_size()) as usize,
                    self.size as usize * ntfs.get_cluster_size() as usize,
                )
                .unwrap(),
        );
        let mut node_headers = Vec::new();
        let mut entry_headers = Vec::new();
        let mut base = 24;
        let mut origin_base = base;
        let mut values = Vec::new();
        // let mut f = fs::File::create("./test.bin").unwrap();
        // f.write_all(&bs);
        while base < bs.len() {
            let entry_header = IndexEntryHeader::parse(bs.slice(base - 24..base), ntfs).unwrap();
            if base + 16 > bs.len() {
                break;
            }
            let node_header = IndexNodeHeader::parse(bs.slice(base..base + 16), ntfs).unwrap();
            let c = node_header.allocated_index_node_size;
            let node_bs = bs.slice(base..node_header.allocated_index_node_size as usize + base);
            let value_offset = node_header.index_values_offset;
            base += node_header.index_node_size as usize;
            let mut index = node_header.index_values_offset as usize;

            while index < node_header.index_node_size as usize {
                let size_offset = index + 8;
                let size = (&node_bs[size_offset..size_offset + 4]).get_u16_le();
                if size == 0 {
                    break;
                }
                let value = IndexValue::parse(node_bs.slice(index..index + size as usize)).unwrap();
                if value.index_value_flags & 0x2 == 0x2 {
                    break;
                }
                values.push(value);
                index += size as usize;
            }
            node_headers.push(node_header);
            entry_headers.push(entry_header);
            base = origin_base + c as usize + 24;
            origin_base = base;
        }

        self.node_header.replace(Some(node_headers));
        self.entry_header.replace(Some(entry_headers));
        self.values.replace(Some(values));
    }
}

impl CNonResident {
    pub fn parse(bs: Bytes) -> Result<Self, MRError> {
        let first_vcn = (sub_bytes(&bs,0..8)?).get_u64_le();
        let last_vcn = (sub_bytes(&bs,8..16)?).get_u64_le();
        let data_run_offset = (sub_bytes(&bs,16..18)?).get_u16_le();
        let compression_unit_size = (sub_bytes(&bs,18..20)?).get_u16_le();
        let allocated_data_size = (sub_bytes(&bs,24..32)?).get_u64_le();
        let data_size = (sub_bytes(&bs,32..40)?).get_u64_le();
        let valid_data_size = (sub_bytes(&bs,40..48)?).get_u64_le();
        if compression_unit_size > 0 {
            let total_allocated_size = (sub_bytes(&bs,48..56)?).get_u64_le();
            return Ok(Self {
                first_vcn,
                last_vcn,
                data_run_offset,
                compression_unit_size,
                allocated_data_size,
                data_size,
                valid_data_size,
                total_allocated_size: Some(total_allocated_size),
            });
        }

        Ok(Self {
            first_vcn,
            last_vcn,
            data_run_offset,
            compression_unit_size,
            allocated_data_size,
            data_size,
            valid_data_size,
            total_allocated_size: None,
        })
    }
}

impl CResident {
    pub fn parse(bs: Bytes) -> Result<Self, MRError> {
        let data_size = (sub_bytes(&bs,0..4)?).get_u32_le();
        let data_offset = (sub_bytes(&bs,4..6)?).get_u16_le();
        let indexed_flag = (sub_bytes(&bs,6..7)?).get_u8();
        Ok(Self {
            data_size,
            data_offset,
            indexed_flag,
            padding: 0,
        })
    }
}

impl CResident {
    fn get_data_size(&self) -> usize {
        self.data_size as usize
    }

    fn get_data_offset(&self) -> usize {
        self.data_offset as usize
    }
}

impl CNonResident {
    fn get_data_size(&self) -> usize {
        self.data_size as usize
    }

    fn get_data_offset(&self) -> usize {
        self.data_run_offset as usize
    }
}

impl CCommon {
    pub fn get_data_size(&self) -> usize {
        match self {
            Self::NonResident(c) => c.data_size as usize,

            Self::Resident(c) => c.data_size as usize,
        }
    }

    pub fn get_data_offset(&self) -> usize {
        match self {
            Self::NonResident(c) => c.data_run_offset as usize,

            Self::Resident(c) => c.data_offset as usize,
        }
    }

    pub fn is_compress(&self) -> bool {
        if let Self::NonResident(c) = self {
            return c.compression_unit_size > 0;
        }

        false
    }

    pub fn get_compress_unit_size(&self) -> usize {
        if let Self::NonResident(c) = self {
            return c.compression_unit_size as usize;
        }

        0
    }
}

impl MFTAttribute {
    pub fn parse(
        bs: &Bytes,
        ntfs: &Ntfs,
        base_of_mft: u64,
        index: u64,
        base_addr: u64,
    ) -> Result<Self, MRError> {
        let offset = base_of_mft as usize;
        let attr_type = (sub_bytes(bs,offset..offset + 4)?).get_u32_le();
        let size = (sub_bytes(bs,offset + 4..offset + 6)?).get_u16_le();
        let non_resident_flag = (sub_bytes(bs,offset + 8..offset + 9)?).get_u8();
        let name_length = (sub_bytes(bs,offset + 9..offset + 10)?).get_u8();
        let name_offset = (sub_bytes(bs,offset + 10..offset + 12)?).get_u16_le();
        let data_flags = (sub_bytes(bs,offset + 12..offset + 14)?).get_u16_le();
        let attr_id = (sub_bytes(bs,offset + 14..offset + 16)?).get_u16_le();
        let attr_name = bs.slice(
            (offset + name_offset as usize)
                ..(offset + name_offset as usize + 2 * name_length as usize),
        );
        let attr_name = vec_u8_to_utf16string(&attr_name);

        let common: CCommon;
        let base: usize;
        if non_resident_flag == 1 {
            let c = CNonResident::parse(bs.slice(offset + 16..offset + 16 + 56))?;
            base = c.data_run_offset as usize;
            common = CCommon::NonResident(c);
        } else {
            let c = CResident::parse(bs.slice(offset + 16..offset + 24))?;
            base = c.data_offset as usize;
            common = CCommon::Resident(c);
        }
        let data_len = common.get_data_size();

        let value = if data_len == 0 {
            MFTValue::None
        } else {
            
            let is_nonresident: bool = non_resident_flag == 1;

            match MFTValue::parse(
                attr_type,
                bs.slice(offset + base..offset + size as usize),
                ntfs,
                index,
                is_nonresident,
                base_addr,
                &common,
            ) {
                Ok(o) => o,
                Err(e) => {
                    fs::write("./error_value", bs);
                    return Err(e);
                }
            }
            
        };

        Ok(Self {
            mft_type: attr_type,
            length: size,
            non_resident_flag,
            name_length,
            name_offset,
            attribute_flags: data_flags,
            identity: attr_id,
            common,
            value,
            attr_name,
        })
    }

    pub fn is_sparse(&self) -> bool {
        self.attribute_flags & 0x8000 == 0x8000
    }

    pub fn is_encrypt(&self) -> bool {
        self.attribute_flags & 0x4000 == 0x4000
    }
}
