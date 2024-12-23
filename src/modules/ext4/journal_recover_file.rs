use std::{collections::HashMap, fs, io::Write};

use crate::utils::MRError;

use super::Ext4Module;

impl Ext4Module {
    pub fn journal_recover_file(&self, args: HashMap<String, String>) -> Result<(), MRError> {
        let inode_id = match args.get("inode") {
            Some(o) => o,
            None => {
                return Err(MRError::new(
                    "Add argument:inode=${inode_id},out_file=${out_file}",
                ));
            }
        };

        let inode_id = inode_id.parse::<u32>().unwrap();
        let out_file = match args.get("out_file") {
            Some(o) => o,
            None => {
                return Err(MRError::new(
                    "Add argument:inode=${inode_id},out_file=${out_file}",
                ));
            }
        };

        let jbd2 = self.ext4.get_jbd2().unwrap();
        let inodes = jbd2.find_inodes(inode_id)?;
        let mut count = 0;
        for i in inodes {
            if let Ok(o) = i.get_extents_value() {
                if o.is_empty() {
                    continue;
                }
                let name = format!("{}.{}", out_file, count);
                let mut f = fs::File::create(&name).unwrap();
                f.write_all(&o).unwrap();
                count += 1;
            }
        }

        Ok(())
    }
}
