use std::collections::HashMap;

use crate::{file_struct::ext4::DirectoryEntry, utils::MRError};

use super::Ext4Module;

impl Ext4Module {
    pub fn list_journal_recoverable(&self, args: HashMap<String,String>) -> Result<Vec<DirectoryEntry>,MRError> {
        let mut vs = vec![];
        let mut result = vec![];
        let path = match args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_path}"));
            }
        };
        let s = self.ext4.get_inode_by_fname(path).unwrap();
        let dirs = s.get_sub_dirs_raw().unwrap();
        let dirs2 = s.get_sub_dirs().unwrap();
        for dir in &dirs {
            if dir.get_name().is_empty() {
                continue;
            }
            let mut flag = false;
            for dir2 in &dirs2 {
                if dir.get_name().eq(dir2.get_name()) {
                    flag = true;
                }
            }
            if flag {
                continue;
            }
            vs.push(dir.clone())
        }

        for sub_dir in vs {
            let jbd2 = self.ext4.get_jbd2().unwrap();
            let inodes = jbd2.find_inodes(sub_dir.get_id()).unwrap();
            for i in inodes {
                if !i.is_empty()? {
                    if result.contains(&sub_dir) {
                        continue;
                    }
                    result.push(sub_dir.clone());
                }
            }
        }
        Ok(result)
    }
}
