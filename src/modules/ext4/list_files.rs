use std::collections::HashMap;

use crate::{file_struct::ext4::DirectoryEntry, utils::MRError};

use super::Ext4Module;

impl Ext4Module {
    pub fn list_files(&self, args: HashMap<String,String>) -> Result<Vec<DirectoryEntry>,MRError> {
        let path = match args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_path}"));
            }
        };
        let inode = match self.ext4.get_inode_by_fname(path) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        let dirs = match inode.get_sub_dirs() {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        Ok(dirs)
    }
}