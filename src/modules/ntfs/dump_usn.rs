use std::{collections::HashMap, fs, io::Write};

use crate::utils::MRError;

use super::NtfsModule;

impl NtfsModule {
    pub fn dump_usn(&mut self, args: HashMap<String,String>) -> Result<(),MRError> {
        //let b = ntfs.get_mft_entry_by_index(516778);
        let _s = "./usn_log.txt".to_string();
        let out = match args.get("out") {
            Some(s) => s,
            None => {
                &_s
            }
        };
        let mut journal = self.ntfs.get_usn_journal().unwrap();
        let mut out_file = fs::File::create(out).unwrap();
        journal.process_entry(|entry| -> bool {
            let line = format!("{}:{} {:?} {}\n", entry.get_index(), entry.filename(), entry.get_time_string(), entry.get_update_reason());
            out_file.write_all(line.as_bytes());
            true
        });
        Ok(())
    }
}