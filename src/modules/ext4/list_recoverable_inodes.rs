use std::collections::HashMap;

use crate::{utils::MRError, file_struct::ext4::Inode};

use super::Ext4Module;

impl Ext4Module {
    pub fn list_recoverable_inodes(&self, _args: HashMap<String,String>) -> Result<Vec<(u32,Inode)>,MRError> {
        let jbd2 = self.ext4.get_jbd2().unwrap();
        let mut result = vec![];
        jbd2.iter_files(|id, inode| {
            if inode.is_deleted() {
                result.push((id, inode.clone()))
            }
        });
        Ok(result)
    }
}