#![allow(unused)]

use meta_reader::file_struct::{elf::{elf64::ELF64, elf32::ELF32}, ext4::Ext4};


fn main() {
    let ext4 = Ext4::open("./target/test.img").unwrap();
    let descs = ext4.get_descs().unwrap();
    for desc in descs {
        println!("{:?}",desc);
    }
}
