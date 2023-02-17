use std::collections::HashMap;

use crate::utils::MRError;

use super::NtfsModule;

impl NtfsModule {
    pub fn stat(&mut self, args: HashMap<String,String>) -> Result<(),MRError> {
        let path = match args.get("path") {
            Some(s) => s,
            None => {
                return Err(MRError::new("path=${target_path}"));
            }
        };
        let mft = match self.ntfs.get_mft_by_path(path) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        println!("creation: {:?}", mft.get_creation_time());
        println!("access: {:?}", mft.get_access_time());
        println!("modify: {:?}", mft.get_change_time());
        println!("creation2: {:?}", mft.filename_creation_time());
        println!("access2: {:?}", mft.filename_access_time());
        println!("modify2: {:?}", mft.filename_change_time());
        Ok(())
    }
}