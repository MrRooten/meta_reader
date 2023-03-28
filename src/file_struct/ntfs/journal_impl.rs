use std::fs;

use bytes::{Buf, Bytes};

use crate::{
    file_struct::ntfs::{
        mft_impl::vec_u8_to_utf16string, FileReference, FileReference128, MFTValue,
    },
    utils::{funcs::i_to_m, MRError},
};

use super::{DataDescriptor, FileTime, MFTEntry, Ntfs, USNChangeJournal, USNChangeJournalEntry};

impl USNChangeJournalEntry {
    pub fn parse(bs: Bytes) -> Result<Self, MRError> {
        let size = (&bs[0..4]).get_u32_le();
        let major_version = (&bs[4..6]).get_u16_le();
        let minor_version = (&bs[6..8]).get_u16_le();
        if major_version == 2 {
            let file_ref = FileReference128 {
                mft_index: (&bs[8..12]).get_u32_le() as u64,
                seq_number: (&bs[12..16]).get_u32_le() as u64,
            };
            let parent_ref = FileReference128 {
                mft_index: (&bs[16..20]).get_u32_le() as u64,
                seq_number: (&bs[20..24]).get_u32_le() as u64,
            };
            let update_usn = (&bs[24..32]).get_u64_le();
            let update_time = (&bs[32..40]).get_u64_le();
            let update_reason = (&bs[40..44]).get_u32_le();
            let update_flag = (&bs[44..48]).get_u32_le();
            let security_d_id = (&bs[48..52]).get_u32_le();
            let f_flag = (&bs[52..56]).get_u32_le();
            let name_size = (&bs[56..58]).get_u16_le();
            let name_offset = (&bs[58..60]).get_u16_le();
            let name = {
                if name_size == 0 || name_offset == 0 {
                    Bytes::from(vec![])
                } else {
                    bs.slice(name_offset as usize..name_offset as usize + name_size as usize)
                }
                
            };
            
            let name = vec_u8_to_utf16string(&name.to_vec());
            return Ok(USNChangeJournalEntry {
                entry_size: size,
                major_version,
                minor_version,
                reference: file_ref,
                parent_reference: parent_ref,
                usn: update_usn,
                update_date: FileTime::parse_from_u64(update_time),
                update_reason_flags: update_reason,
                update_source_flags: update_flag,
                security_descriptor_id: security_d_id,
                file_attributes_flags: f_flag,
                name_size,
                name_offset,
                name,
            });
        } else if major_version == 3 {
            let file_ref = FileReference128 {
                mft_index: (&bs[8..16]).get_u64_le(),
                seq_number: (&bs[16..24]).get_u64_le(),
            };
            let parent_ref = FileReference128 {
                mft_index: (&bs[24..32]).get_u32_le() as u64,
                seq_number: (&bs[32..40]).get_u32_le() as u64,
            };
            let update_usn = (&bs[40..48]).get_u64_le();
            let update_time = (&bs[48..56]).get_u64_le();
            let update_reason = (&bs[56..60]).get_u32_le();
            let update_flag = (&bs[60..64]).get_u32_le();
            let security_d_id = (&bs[64..68]).get_u32_le();
            let f_flag = (&bs[68..72]).get_u32_le();
            let name_size = (&bs[72..74]).get_u16_le();
            let name_offset = (&bs[74..76]).get_u16_le();
            let name = bs.slice(name_offset as usize..name_offset as usize + name_size as usize);
            let name = vec_u8_to_utf16string(&name.to_vec());
            return Ok(USNChangeJournalEntry {
                entry_size: size,
                major_version,
                minor_version,
                reference: file_ref,
                parent_reference: parent_ref,
                usn: update_usn,
                update_date: FileTime::parse_from_u64(update_time),
                update_reason_flags: update_reason,
                update_source_flags: update_flag,
                security_descriptor_id: security_d_id,
                file_attributes_flags: f_flag,
                name_size,
                name_offset,
                name,
            });
        }
        return Err(MRError::new(
            "Not recognize the version of USNChangeJournal",
        ));
    }
}

impl USNChangeJournal {
    pub fn from_mft(mft: MFTEntry, ntfs: &Ntfs) -> Result<Self, MRError> {
        Ok(USNChangeJournal {
            mft: mft,
            ntfs: Some(ntfs),
        })
    }

    fn read_data(&mut self, addr: usize, n: usize) -> Result<Vec<u8>, MRError> {
        if self.mft.map_attr_chains.get(&0x20).is_none() {
            return self.mft.read_n_in_stream(addr, n, "$J");
        }
        let ntfs = unsafe { &*self.ntfs.unwrap() };
        let ntfs = i_to_m(ntfs);
        let mut data_runs: Option<Vec<DataDescriptor>> = None;
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
                        let mft = match ntfs.get_mft_entry_by_index(l.file_reference.mft_index) {
                            Some(s) => s,
                            None => {
                                return Err(MRError::new("Not found mft"));
                            }
                        };

                        let data = mft.get_data_value().unwrap();
                        let data = data.datas[1..].to_vec();
                        data_runs = Some(data);
                    }
                }
            }
        }
        if data_runs.is_none() {
            return Err(MRError::new("Not found $J Attribute List"));
        }

        let data_runs = data_runs.unwrap();
        let mut result = Vec::new();
        let real_n = n;
        let mut last_n = real_n as u64;
        let mut last_addr = addr as u64;
        let ntfs = unsafe { &*self.ntfs.unwrap() };
        for data in &data_runs {
            if last_addr > data.datasize {
                last_addr -= data.datasize;
                continue;
            }
            let buffer_data: Vec<u8>;
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
                .read_n(start_addr as usize, __offset as usize + read_size as usize)
                .unwrap();
            buffer_data = tmp_data[__offset as usize..].to_vec();

            if last_addr < buffer_data.len() as u64
                && last_n > buffer_data.len() as u64 - last_addr
            {

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
                let bs = buffer_data[(last_addr) as usize..(last_addr + last_n) as usize]
                    .to_vec();
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

        return USNChangeJournalEntry::parse(Bytes::from(data));
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
            let entry_size = (&bs[offset..offset + 4]).get_u32_le();
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

        return Ok(result);
    }
}
