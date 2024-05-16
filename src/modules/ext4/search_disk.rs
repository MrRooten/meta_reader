use std::{collections::HashMap, fmt::Write, ops::Range};

use indicatif::{ProgressState, ProgressStyle};

use crate::{
    file_struct::ext4::{Ext4, Inode},
    modules::ntfs::MatchType,
    utils::MRError,
};

use super::Ext4Module;

fn cache_inodes<F>(ext4: &Ext4, mut filter: F) -> Vec<(Range<usize>, u32)>
where
    F: FnMut(u64, &Inode) -> bool,
{
    let mut result = Vec::new();
    let mut cache = Vec::new();
    ext4.iter_inodes(|id, entry, all| {
        // unimplemented!();
        // let data_value = entry.get_flat_extents();
        // if let Ok(extents) = data_value {
        //     for extent in extents {
        //         let start = extent.get_start() * ext4.get_block_size();
        //         let end = extent.get_start() * ext4.get_block_size() + extent.get_len() * ext4.get_block_size();
        //         let v = (
        //             Range {
        //                 start: start,
        //                 end: end
        //             },
        //             id
        //         );
        //         cache.push(v);
        //     }
        // }
        if id % 888 == 0 {
            println!("{}",id);
        }
    });
    //cache.sort_by(|k, v| k.0.start.cmp(&v.0.start));
    result.extend(cache);
    result
}

pub fn sec_to_s(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

impl Ext4Module {
    pub fn search_disk(&self, args: HashMap<String, String>) -> Result<(), MRError> {
        let match_type: MatchType;
        let default_to_file = "true".to_string();
        let to_file = match args.get("ref_file") {
            Some(s) => s,
            None => &default_to_file,
        };
        let bool_to_file: bool = true;
        let mut _inodes: Vec<(Range<usize>, u32)> = vec![];
        println!("Loading inodes....");
        _inodes = cache_inodes(&self.ext4, |offset, entry| {
            true
        });

        Ok(())
    }
}
