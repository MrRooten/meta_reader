use colored::{ColoredString, Colorize};

use crate::{file_struct::{FileSystem, ext4::Ext4, ntfs::Ntfs}, utils::MRError};

pub struct CliEnv {
    count       : u32,
    filesystem  : Option<Box<dyn FileSystem>>,
    path        : String
}

impl Default for CliEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl CliEnv {
    pub fn new() -> CliEnv {
        Self { 
            count : 0,
            filesystem : None,
            path: "".to_string(),
        }
    }
    pub fn get_prompt(&self) -> ColoredString {
        format!("reader {}> ", self.count).bright_blue()
    }

    pub fn add_count(&mut self) {
        self.count += 1
    }

    pub fn set_filesystem(&mut self, fs: &str, path: &str) -> Result<(), MRError> {
        if fs.eq("ext4") {
            let ext4 = match Ext4::open(path) {
                Ok(o) => o, 
                Err(e) => {
                    return Err(e);
                }
            };
            self.filesystem = Some(Box::new(ext4));
        } else if fs.eq("ntfs") {
            let ntfs = match Ntfs::open(path) {
                Ok(o) => o, 
                Err(e) => {
                    return Err(e);
                }
            };
            self.filesystem = Some(Box::new(ntfs));
        }
        Ok(())
    }

    pub fn get_filesystem(&self) -> &Option<Box<dyn FileSystem>> {
        &self.filesystem
    }

    pub fn get_cur_path(&self) -> &String {
        &self.path
    }
}


pub fn process_line(cmdline: &str, env: &mut CliEnv) {
    cmdline.split(' ');
    env.add_count()
    
}