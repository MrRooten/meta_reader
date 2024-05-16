use std::collections::HashMap;

use crate::{file_struct::ext4::{DirectoryEntry, Inode, Ext4}, utils::MRError};

use super::Ext4Module;

impl Ext4Module {
    pub fn list_deleted_files<F>(&mut self, args: HashMap<String,String>, mut f: F) -> Result<Vec<DirectoryEntry>,MRError>
    where F: FnMut(u32,Option<Inode>,&String,&String,&mut Ext4) {
        let path = match args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_dir}"));
            }
        };
        let mut vs = vec![];
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
            vs.push(dir.clone());
            let inode = self.ext4.get_inode_by_id(dir.get_id());
            if let Ok(o) = &inode {
                f(dir.get_id(),Some(o.clone()),dir.get_name(),dir.get_zero_end_name(),&mut self.ext4);
            }

            if let Err(_e) = &inode {
                f(dir.get_id(), None, dir.get_name(), dir.get_zero_end_name(),&mut self.ext4);
            }
        }
        Ok(vs)
    }
}
