use std::collections::HashMap;

use crate::{
    file_struct::ext4::{Inode, Ext4},
    utils::MRError,
};

use super::Ext4Module;

impl Ext4Module {
    pub fn search_recoverable_files<F>(
        &mut self,
        args: HashMap<String, String>,
        mut f: F
    ) -> Result<Vec<String>, MRError> 
    where F: FnMut(u32, Inode, &String, &String,&mut Ext4){
        let path = match args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_dir}"));
            }
        };
        let base_inode = match self.ext4.get_inode_by_fname(path) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };
        let cur_inode = base_inode;
        let result = vec![];
        let mut stack = vec![];
        stack.push((path.to_string(),cur_inode));

        while let Some(s) = stack.pop() {
            
            if !s.1.is_dir() {
                continue;
            }

            let dirs = s.1.get_sub_dirs_raw().unwrap();
            let dirs2 = s.1.get_sub_dirs().unwrap();
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

                    
                    let mut name = s.0.clone();
                    if !name.ends_with('/') {
                        name.push('/');
                    }
                    name.push_str(i.get_name());
                    let mut name2 = s.0.clone();
                    if !name2.ends_with('/') {
                        name2.push('/');
                    }
                    name2.push_str(i.get_zero_end_name());
                    let jbd2 = self.ext4.get_jbd2().unwrap();
                    let jbd2_inodes = jbd2.find_inodes(i.get_id()).unwrap();
                    let ext4 = &mut self.ext4;
                    if !jbd2_inodes.is_empty() {
                        for jbd2_inode in jbd2_inodes {
                            f(i.get_id(), jbd2_inode, &name, &name2, ext4);
                        }
                    }
                    continue;
                }

                if !inode.is_dir() {
                    continue;
                }
                for x in &dirs2 {
                    if x.get_id() != i.get_id() {
                        continue;
                    }
                    if x.get_name().eq(".") || x.get_name().eq("..") {
                        continue;
                    }
                    let mut name = s.0.clone();
                    if !name.ends_with('/') {
                        name.push('/');
                    }
                    name.push_str(x.get_name());
                    stack.push((name, self.ext4.get_inode_by_id(x.get_id()).unwrap()));
                }
                
            }
        }
        Ok(result)
    }
}