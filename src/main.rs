#![allow(unused)]

use meta_reader::file_struct::elf::{elf64::ELF64, elf32::ELF32};


fn main() {
    let elf = ELF32::new("./target/test32".to_string());
    let syms = elf.get_syms().unwrap();
    for sym in syms {
        println!("{} {:0x}",sym.get_name(),sym.get_value().0);
    }
    println!()
}
