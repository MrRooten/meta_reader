
use std::{collections::HashMap, fs, ptr::addr_of};

use bytes::{Buf, Bytes};

use crate::{
    file_struct::ntfs::{
        mft_impl::vec_u8_to_utf16string, FileReference, FileReference128, MFTValue, USNIdentifier,
    },
    utils::{MRErrKind, MRError},
};

use super::{DataDescriptor, FileTime, MFTEntry, Ntfs, USNChangeJournal, USNChangeJournalEntry};

static mut USN_REASON: Option<HashMap<u32,&str>> = None;
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}
impl USNChangeJournalEntry {
    pub fn filetime(&self) -> Option<&FileTime> {
        Some(&self.update_date)
    }

    pub fn get_time_string(&self) -> Option<String> {
        self.update_date.to_native_date().map(|s| s.to_string())
    }

    pub fn get_index(&self) -> u64 {
        self.reference.mft_index
    }

    pub fn filename(&self) -> &String {
        &self.name
    }

    pub fn get_timestamp(&self) -> u64 {
        self.update_date.get_timestamp()
    }

    pub fn get_update_reason(&self) -> String {
        if unsafe { USN_REASON.is_none() } {
            let mut map = HashMap::new();
            map.insert(0x00000001, "DATA_OVERWRITE");
            map.insert(0x00000002, "DATA_EXTEND");
            map.insert(0x00000004, "DATA_TRUNCATION");
            map.insert(0x00000010, "NAMED_DATA_OVERWRITE");
            map.insert(0x00000020, "NAMED_DATA_EXTEND");
            map.insert(0x00000040, "NAMED_DATA_TRUNCATION");
            map.insert(0x00000100, "FILE_CREATE");
            map.insert(0x00000200, "FILE_DELETE");
            map.insert(0x00000400, "EA_CHANGE");
            map.insert(0x00000800, "SECURITY_CHANGE");
            map.insert(0x00001000, "RENAME_OLD_NAME");
            map.insert(0x00002000, "RENAME_NEW_NAME");
            map.insert(0x00004000, "INDEXABLE_CHANGE");
            map.insert(0x00008000, "BASIC_INFO_CHANGE");
            map.insert(0x00010000, "HARD_LINK_CHANGE");
            map.insert(0x00020000, "COMPRESSION_CHANGE");
            map.insert(0x00040000, "ENCRYPTION_CHANGE");
            map.insert(0x00080000, "OBJECT_ID_CHANGE");
            map.insert(0x00100000, "REPARSE_POINT_CHANGE");
            map.insert(0x00200000, "STREAM_CHANGE");
            map.insert(0x00400000, "TRANSACTED_CHANGE");
            map.insert(0x80000000, "CLOSE");
            unsafe {
                USN_REASON = Some(map);
            }
        }

        let mut reason = vec![];
        if let Some(map) = unsafe { &*addr_of!(USN_REASON) } {
            for i in map {
                if self.update_reason_flags & i.0 != 0 {
                    reason.push(*i.1);
                }
            }
        }

        reason.join("|")
    }

    pub fn parse(bs: Bytes) -> Result<Self, MRError> {
        if bs.len() < 60 || bs.len() % 8 != 0{
            return Err(MRError::new("size not right"));
        }
        let size = (bs.get(0..4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
        let major_version = (bs.get(4..6).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16_le();
        let minor_version = (bs.get(6..8).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16_le();
        if major_version == 2 {
            let file_ref = FileReference128 {
                mft_index: (bs.get(8..12).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le() as u64,
                seq_number: (bs.get(12..16).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le() as u64,
            };
            let parent_ref = FileReference128 {
                mft_index: (bs.get(16..20).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le() as u64,
                seq_number: (bs.get(20..24).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le() as u64,
            };
            let update_usn = (bs.get(24..32).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64_le();
            let update_time = FileTime::parse_from_u64((bs.get(32..40).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64());
            let update_reason = (bs.get(40..44).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let update_flag = (bs.get(44..48).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let security_d_id = (bs.get(48..52).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let f_flag = (bs.get(52..56).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let name_size = (bs.get(56..58).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16_le();
            let name_offset = (bs.get(58..60).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16_le();
            let name = {
                if name_size == 0 || name_offset == 0 {
                    Bytes::from(vec![])
                } else {
                    if name_offset as usize > bs.len() || name_offset as usize + name_size as usize > bs.len() {
                        return Err(MRError::new("Error entry parse in name_offset or name_size"));
                    }
                    bs.slice(name_offset as usize..name_offset as usize + name_size as usize)
                }
            };

            let name = vec_u8_to_utf16string(&name);
            return Ok(USNChangeJournalEntry {
                entry_size: size,
                major_version,
                minor_version,
                reference: file_ref,
                parent_reference: parent_ref,
                usn: update_usn,
                update_date: update_time,
                update_reason_flags: update_reason,
                update_source_flags: update_flag,
                security_descriptor_id: security_d_id,
                file_attributes_flags: f_flag,
                name_size,
                name_offset,
                name,
            });
        } else if major_version == 3 {
            if bs.len() < 76 {
                return Err(MRError::new("size not right"));
            }
            let file_ref = FileReference128 {
                mft_index: (bs.get(8..16).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64_le(),
                seq_number: (bs.get(16..24).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64_le(),
            };
            let parent_ref = FileReference128 {
                mft_index: (bs.get(24..32).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le() as u64,
                seq_number: (bs.get(32..40).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le() as u64,
            };
            let update_usn = (bs.get(40..48).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64_le();
            let update_time = FileTime::parse_from_u64((bs.get(48..56).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u64());
            let update_reason = (bs.get(56..60).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let update_flag = (bs.get(60..64).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let security_d_id = (bs.get(64..68).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let f_flag = (bs.get(68..72).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let name_size = (bs.get(72..74).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16_le();
            let name_offset = (bs.get(74..76).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u16_le();
            let name = {
                if name_size == 0 || name_offset == 0 {
                    Bytes::from(vec![])
                } else {
                    if name_offset as usize > bs.len() || name_offset as usize + name_size as usize > bs.len() {
                        return Err(MRError::new("Error entry parse in name_offset or name_size"));
                    }
                    bs.slice(name_offset as usize..name_offset as usize + name_size as usize)
                }
            };
            let name = vec_u8_to_utf16string(&name);
            return Ok(USNChangeJournalEntry {
                entry_size: size,
                major_version,
                minor_version,
                reference: file_ref,
                parent_reference: parent_ref,
                usn: update_usn,
                update_date: update_time,
                update_reason_flags: update_reason,
                update_source_flags: update_flag,
                security_descriptor_id: security_d_id,
                file_attributes_flags: f_flag,
                name_size,
                name_offset,
                name,
            });
        }
        Err(MRError::new(
            "Not recognize the version of USNChangeJournal",
        ))
    }
}

impl USNChangeJournal {
    pub fn from_mft(mft: MFTEntry, ntfs: &Ntfs) -> Result<Self, MRError> {
        Ok(USNChangeJournal {
            mft,
            ntfs: Some(ntfs),
        })
    }

    pub fn process_entry<F>(&self, mut handle: F) -> Result<(), MRError>
    where
        F: FnMut(&USNChangeJournalEntry) -> bool,
    {
        let data = match self.read_all() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let bs = Bytes::from(data);
        let mut i = 0;
        let mut offset = 0;
        while offset < bs.len() {
            let entry_size = (bs.get(offset..offset + 4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            if entry_size == 0 || entry_size > 512 {
                offset += 0x8;
                continue;
            }

            if offset + entry_size as usize > bs.len() {
                break;
            }
            let v = USNChangeJournalEntry::parse(bs.slice(offset..offset + entry_size as usize));
            let v = match v {
                Ok(o) => o,
                Err(e) => {
                    //fs::write("./error_entry", bs.slice(offset..).to_vec());
                    offset += 0x8;
                    continue;
                }
            };
            if !handle(&v) {
                break;
            }
            offset += entry_size as usize;
            i += 1;
        }

        Ok(())
    }

    fn get_data_runs(&self) -> Result<Vec<DataDescriptor>, MRError> {
        let data_runs: Vec<DataDescriptor>;
        if self.mft.map_attr_chains.contains_key(&0x20) {
            let ntfs = self.get_ntfs();
            let mut _data_runs: Option<Vec<DataDescriptor>> = None;
            if let Some(attrs) = self.mft.map_attr_chains.get(&0x20) {
                let attr = attrs.first().unwrap();
                if let MFTValue::AttrList(attrlist) = &attr.value {
                    let list = match &attrlist.list {
                        Some(s) => s,
                        None => {
                            return Err(MRError::new("List is empty"));
                        }
                    };
                    for l in list {
                        if l.name.eq("$J") {
                            let mft = match ntfs.get_mft_entry_by_index(l.file_reference.mft_index)
                            {
                                Some(s) => s,
                                None => {
                                    return Err(MRError::new("Not found mft"));
                                }
                            };

                            let data = match mft.get_stream("$J") {
                                Some(o) => o,
                                None => {
                                    return Err(MRError::new("Not found $J Stream, AttributeList"));
                                }
                            };
                            let data = data.datas[1..].to_vec();
                            _data_runs = Some(data);
                        }
                    }
                }
            }
            if _data_runs.is_none() {
                return Err(MRError::new("Not found $J Attribute List"));
            }

            data_runs = _data_runs.unwrap();
        } else {
            let stream = match self.mft.get_stream("$J") {
                Some(s) => s,
                None => {
                    return Err(MRError::new("Not found $J Stream, File"));
                }
            };
            if stream.datas[0].start_addr == 0 {
                data_runs = stream.datas[1..].to_vec();
            } else {
                data_runs = stream.datas.to_vec();
            }
            
        }

        Ok(data_runs)
    }

    fn get_ntfs(&self) -> &Ntfs {
        unsafe { &* self.ntfs.unwrap() }
    }

    pub fn read_all(&self) -> Result<Vec<u8>, MRError> {
        let data_runs: Vec<DataDescriptor> = match self.get_data_runs() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let mut result = vec![];
        let ntfs = self.get_ntfs();

        for data in data_runs {
            if data.datasize > 20*1024*1024 {
                continue;
            }
            let tmp_data = ntfs
                .reader
                .read_n(data.start_addr as usize, data.datasize as usize)
                .unwrap();
            result.extend(tmp_data);
        }
        Ok(result)
    }

    fn read_data(&mut self, addr: usize, n: usize) -> Result<Vec<u8>, MRError> {
        let data_runs: Vec<DataDescriptor> = match self.get_data_runs() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let mut result = Vec::new();
        let real_n = n;
        let mut last_n = real_n as u64;
        let mut last_addr = addr as u64;
        let ntfs = self.get_ntfs();
        for data in &data_runs {
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
                let bs = buffer_data[(last_addr) as usize..(data.datasize) as usize].to_vec();
                result.extend(bs);
                last_n -= data.datasize - last_addr;
                last_addr = 0;

                continue;
            }

            if last_addr < buffer_data.len() as u64
                && last_n <= buffer_data.len() as u64 - last_addr
            {
                let bs = buffer_data[(last_addr) as usize..(last_addr + last_n) as usize].to_vec();
                result.extend(bs);
                break;
            }
        }

        Ok(result)
    }

    pub fn read_first(&mut self) -> Result<USNChangeJournalEntry, MRError> {
        let data = match self.read_data(0, 4096) {
            Ok(o) => o,
            Err(e) => {
                return Err(MRError::new("Not found data"));
            }
        };

        USNChangeJournalEntry::parse(Bytes::from(data))
    }

    pub fn read_last(&mut self) -> Result<Vec<USNChangeJournalEntry>, MRError> {
        let data_runs: Vec<DataDescriptor> = match self.get_data_runs() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        let ntfs = self.get_ntfs();
        let data = &data_runs[data_runs.len()-1];
        let tmp_data = ntfs
            .reader
            .read_n(data.start_addr as usize, data.datasize as usize)
            .unwrap();
        let mut offset = 0;
        let bs = Bytes::from(tmp_data);
        let mut result = vec![];
        while offset < bs.len() {
            let entry_size = (bs.get(offset..offset + 4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            if entry_size == 0 || entry_size > 512 {
                offset += 0x8;
                continue;
            }
            let v = USNChangeJournalEntry::parse(bs.slice(offset..offset + entry_size as usize));
            let v = match v {
                Ok(o) => o,
                Err(e) => {
                    offset += 0x8;
                    continue;
                }
            };
            result.push(v);
            offset += entry_size as usize;

        }
        Ok(result)
    }

    pub fn process_last<F>(&mut self,mut f: F) -> Result<(), MRError> 
    where F: FnMut(&USNChangeJournalEntry) -> bool {
        let data_runs: Vec<DataDescriptor> = match self.get_data_runs() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        let ntfs = self.get_ntfs();
        let data = data_runs.last().unwrap();
        let tmp_data = ntfs
            .reader
            .read_n(data.start_addr as usize, data.datasize as usize)
            .unwrap();
        let mut offset = 0;
        let bs = Bytes::from(tmp_data);
        while offset < bs.len() {
            let entry_size = (bs.get(offset..offset + 4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            if entry_size == 0 || entry_size > 512 {
                offset += 0x8;
                continue;
            }
            let v = USNChangeJournalEntry::parse(bs.slice(offset..offset + entry_size as usize));
            let v = match v {
                Ok(o) => o,
                Err(e) => {
                    offset += 0x8;
                    continue;
                }
            };
            if !f(&v) {
                break;
            }
            offset += entry_size as usize;

        }
        Ok(())
    }

    pub fn read_n_entry(&mut self, n: usize) -> Result<Vec<USNChangeJournalEntry>, MRError> {
        let mut result = vec![];
        let size = n * 0x200;
        let bs = match self.read_data(0, size) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let bs = Bytes::from(bs);
        let mut i = 0;
        let mut offset = 0;
        while i < n {
            let entry_size = (bs.get(offset..offset + 4).ok_or(MRError::new_with_kind("Out of range", MRErrKind::OutOfByteRange))?).get_u32_le();
            let v = USNChangeJournalEntry::parse(bs.slice(offset..offset + entry_size as usize));
            let v = match v {
                Ok(o) => o,
                Err(e) => {
                    break;
                }
            };
            result.push(v);
            offset += entry_size as usize;
            i += 1;
        }

        Ok(result)
    }
}
