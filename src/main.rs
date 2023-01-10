#![allow(unused)]

use meta_reader::file_struct::{elf::{elf64::ELF64, elf32::ELF32}, ext4::Ext4};


fn main() {
    let ext4 = Ext4::open("/dev/sdb").unwrap();
    let descs = ext4.get_descs().unwrap();
    let inode = ext4.get_inode_by_fname("/").unwrap();
    let dirs = inode.get_sub_dirs().unwrap();
    for dir in dirs {
        let i = ext4.get_inode_by_id(dir.get_id());
        println!("{} {} {}", dir.get_id(), dir.get_name(), i.is_deleted());
    }
}
