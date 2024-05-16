use std::collections::HashMap;

use crate::{utils::MRError, file_struct::ext4::{Inode, Ext4}};

use super::Ext4Module;

impl Ext4Module {
    pub fn list_recoverable_inodes<F>(&self, _args: HashMap<String,String>, mut f: F)
    -> Result<Vec<String>, MRError> 
    where F: FnMut(u32, Inode, &String ,&Ext4) {
        let mut result = vec![];
        let path = match _args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_dir}"));
            }
        };
        let s = match self.ext4.get_inode_by_fname(path) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        let dirs = s.get_sub_dirs_raw().unwrap();
        let dirs2 = s.get_sub_dirs().unwrap();
        for i in dirs {
            let inode = match self.ext4.get_inode_by_id(i.get_id()) {
                Ok(o) => o,
                Err(_) => {
                    continue;
                }
            };
            if dirs2.iter().all(|dir| {
                !dir.get_name().eq(i.get_name())
            }) {
                if i.get_name().is_empty() {
                    continue;
                }

                

                let jbd2 = self.ext4.get_jbd2().unwrap();
                let jbd2_inodes = jbd2.find_inodes(i.get_id()).unwrap();
                let ext4 = &self.ext4;
                if !jbd2_inodes.is_empty() {
                    for jbd2_inode in jbd2_inodes {
                        f(i.get_id(), jbd2_inode, i.get_name(), ext4);
                    }
                }
                continue;
            }
            
        }
        Ok(result)
    }
}