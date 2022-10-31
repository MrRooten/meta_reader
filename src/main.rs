#![allow(unused)]

use meta_reader::file_struct::elf::{ELF32, ELF64};

fn main() {
    let elf = ELF64::new("./test64".to_string());
}
