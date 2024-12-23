use std::{collections::HashMap, io::Write};

use crate::utils::MRError;

use super::Ext4Module;

impl Ext4Module {
    pub fn read_file(&self, args: HashMap<String,String>) -> Result<(),MRError> {
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
        let value = inode.get_extents_value().unwrap();
        let size = inode.get_size();
        let mut out = std::io::stdout();
        if let Err(e) = out.write(&value[..size as usize]) {
            return Err(MRError::from(Box::new(e)));
        }
        Ok(())
    }
}