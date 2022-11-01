#![allow(unused)]

use meta_reader::file_struct::elf::{ELF32, ELF64};

fn main() {
    let elf = ELF64::new("./test64".to_string());
    let syms = elf.get_syms().unwrap();
    for sym in syms {
        println!("{} {:0x}",sym.get_name(),sym.get_value().0);
    }
    println!()
}
