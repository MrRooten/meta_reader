use std::ops::Range;

use crate::utils::MRError;

use super::{Bitmap, DataDescriptor, MFTEntry, MFTValue, Ntfs};

impl Bitmap {
    pub fn from_mft(mft: MFTEntry, ntfs: &Ntfs) -> Result<Self, MRError> {
        Ok(Self {
            mft,
            ntfs: Some(ntfs),
        })
    }

    pub fn generate_unalloc(&self) -> Result<(Vec<Range<usize>>, usize), MRError> {
        let mut res = vec![];
        let data = self.read_all()?;
        let mut start = 0;
        let mut start = 0;
        let mut end = 0;
        for (index, b) in data.iter().enumerate() {
            for i in 0..8u8 {
                let bit = (b >> i) & 1;
                if bit == 0 {
                    end += 1;
                } else {
                    if start == end {
                        start += 1;
                        end += 1;
                        continue;
                    }
                    res.push(Range {
                        start,
                        end
                    });

                    end += 1;
                    start = end;
                }
            }

            
        }

        if end != start {
            res.push(Range {
                start,
                end
            });
        }
        let mut count = 0;
        for i in &res {
            let x = i.end - i.start;
            count += x;
        }
        Ok((res, count))
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
            let stream = match self.mft.get_data_value() {
                Some(s) => s,
                None => {
                    return Err(MRError::new("Not found data stream"));
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
        unsafe { &*self.ntfs.unwrap() }
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
            if data.datasize > 20 * 1024 * 1024 {
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
}
