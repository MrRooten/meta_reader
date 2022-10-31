#![allow(unused)]
#![allow(non_camel_case_types)]
use std::{fs, ops::Range};

use bytes::{Bytes, Buf};

#[derive(Default,Debug)]
struct Elf32_Addr(u32);

#[derive(Default,Debug)]
struct Elf32_Off(u32);

#[derive(Default,Debug)]
#[warn(non_camel_case_types)]
struct Elf64_Addr(u64);

#[derive(Default,Debug)]
struct Elf64_Off(u64);

pub enum ShdrType {
    SHT_NULL=0,
    SHT_PROGBITS=1,
    SHT_SYMTAB=2,
    SHT_STRTAB=3,
    SHT_RELA=4,
    SHT_HASH=5,
    SHT_DYNAMIC=6,
    SHT_NOTE=7,
    SHT_NOBITS=8,
    SHT_REL=9,
    SHT_SHLIB=11,
    SHT_DYNSYM=12,
    SHT_LOPROC=0x70000000,
    SHT_HIPROC=0x7fffffff,
    SHT_LOUSER=0x80000000,
    SHT_HIUSER=0xffffffff
}
#[derive(Default,Debug)]
struct  ElfN_Ehdr {
    e_ident     : [u8;16],
    e_machine   : u16,
    e_type      : u16,
    e_version   : u32,
    e_flags     : u32,
    e_ehsize    : u16,
    e_phentsize : u16,
    e_phnum     : u16,
    e_shnum     : u16,
    e_shentsize : u16,
    e_shstrndx  : u16
}

impl ElfN_Ehdr {
    
}
#[derive(Default,Debug)]
pub struct  Elf32_Ehdr {
    e_entry     : Elf32_Addr,
    e_phoff     : Elf32_Off,
    e_shoff     : Elf32_Off,
    _ehdr       : ElfN_Ehdr
}

#[derive(Default,Debug)]
pub struct Elf64_Ehdr {
    e_entry     : Elf64_Addr,
    e_phoff     : Elf64_Off,
    e_shoff     : Elf64_Off,
    _ehdr       : ElfN_Ehdr
}

impl Elf32_Ehdr {
    fn new(bytes : &Bytes) -> Elf32_Ehdr {
        let mut ehdr = Elf32_Ehdr::default();
        ehdr._ehdr.e_ident.copy_from_slice(&bytes[0..16]);
        ehdr._ehdr.e_type = (&bytes[16..18]).get_u16_le();
        ehdr._ehdr.e_machine = (&bytes[18..20]).get_u16_le();
        ehdr._ehdr.e_version = (&bytes[20..24]).get_u32_le();
        ehdr.e_entry = Elf32_Addr((&bytes[24..28]).get_u32_le());
        ehdr.e_phoff = Elf32_Off((&bytes[28..32]).get_u32_le());
        ehdr.e_shoff = Elf32_Off((&bytes[32..36]).get_u32_le());
        ehdr._ehdr.e_flags = (&bytes[36..40]).get_u32_le();
        ehdr._ehdr.e_ehsize = (&bytes[40..42]).get_u16_le();
        ehdr._ehdr.e_phentsize = (&bytes[42..44]).get_u16_le();
        ehdr._ehdr.e_phnum = (&bytes[44..46]).get_u16_le();
        ehdr._ehdr.e_shentsize = (&bytes[46..48]).get_u16_le();
        ehdr._ehdr.e_shnum = (&bytes[48..50]).get_u16_le();
        ehdr._ehdr.e_shstrndx = (&bytes[50..52]).get_u16_le();
        ehdr
    }

    fn get_type(&self) -> String {
        let t = self._ehdr.e_type;
        if t == 0 {
            return "ET_None".to_string();
        }
        else if t == 1 {
            return "ET_REL".to_string();
        }
        else if t == 2 {
            return "ET_EXEC".to_string();
        }
        else if t == 3 {
            return "ET_DYN".to_string();
        }
        else if t == 4 {
            return "ET_CORE".to_string();
        }
        else if t == 5 {
            return "ET_NUM".to_string();
        }

        return "".to_string();
    }

    fn get_machine(&self) -> String {
        "".to_string()
    }

    fn get_entry(&self) -> &Elf32_Addr {
        &self.e_entry
    }

    fn get_phdr_offset(&self) -> &Elf32_Off {
        &self.e_phoff
    }

    fn get_shdr_offset(&self) -> &Elf32_Off {
        &self.e_shoff
    }
}

impl Elf64_Ehdr {
    fn new(bytes : &Bytes) -> Elf64_Ehdr {
        let mut ehdr = Elf64_Ehdr::default();
        ehdr._ehdr.e_ident.copy_from_slice(&bytes[0..16]);
        ehdr._ehdr.e_type = (&bytes[16..18]).get_u16_le();
        ehdr._ehdr.e_machine = (&bytes[18..20]).get_u16_le();
        ehdr._ehdr.e_version = (&bytes[20..24]).get_u32_le();
        ehdr.e_entry = Elf64_Addr((&bytes[24..32]).get_u64_le());
        ehdr.e_phoff = Elf64_Off((&bytes[32..40]).get_u64_le());
        ehdr.e_shoff = Elf64_Off((&bytes[40..48]).get_u64_le());
        ehdr._ehdr.e_flags = (&bytes[48..52]).get_u32_le();
        ehdr._ehdr.e_ehsize = (&bytes[52..54]).get_u16_le();
        ehdr._ehdr.e_phentsize = (&bytes[54..56]).get_u16_le();
        ehdr._ehdr.e_phnum = (&bytes[56..58]).get_u16_le();
        ehdr._ehdr.e_shentsize = (&bytes[58..60]).get_u16_le();
        ehdr._ehdr.e_shnum = (&bytes[60..62]).get_u16_le();
        ehdr._ehdr.e_shstrndx = (&bytes[62..64]).get_u16_le();
        ehdr
    }

    fn get_type(&self) -> String {
        let t = self._ehdr.e_type;
        if t == 0 {
            return "ET_None".to_string();
        }
        else if t == 1 {
            return "ET_REL".to_string();
        }
        else if t == 2 {
            return "ET_EXEC".to_string();
        }
        else if t == 3 {
            return "ET_DYN".to_string();
        }
        else if t == 4 {
            return "ET_CORE".to_string();
        }
        else if t == 5 {
            return "ET_NUM".to_string();
        }

        return "".to_string();
    }

    fn get_machine(&self) -> String {
        "".to_string()
    }

    fn get_entry(&self) -> &Elf64_Addr {
        &self.e_entry
    }

    fn get_phdr_offset(&self) -> &Elf64_Off {
        &self.e_phoff
    }

    fn get_shdr_offset(&self) -> &Elf64_Off {
        &self.e_shoff
    }
}

#[derive(Default,Debug)]
pub struct ElfN_Phdr {
    p_type : u32,
    p_flags : u32,

}

#[derive(Default,Debug)]
pub struct Elf32_Phdr {
    p_offset : Elf32_Off,
    p_vaddr  : Elf32_Addr,
    p_paddr  : Elf32_Addr,
    p_filesz : u32,
    p_memsz  : u32,
    p_align  : u32,
    _phdr    : ElfN_Phdr,
    self_elf : Option<*const ELF32>
}

#[derive(Default,Debug)]
pub struct Elf64_Phdr {
    p_offset : Elf64_Off,
    p_vaddr  : Elf64_Addr,
    p_paddr  : Elf64_Addr,
    p_filesz : u64,
    p_memsz  : u64,
    p_align  : u64,
    _phdr    : ElfN_Phdr,
    self_elf : Option<*const ELF64>
}

impl Elf32_Phdr {
    fn new(bytes : &Bytes, elf : &ELF32) -> Elf32_Phdr{
        let mut phdr = Elf32_Phdr::default();
        phdr._phdr.p_type = (&bytes[0..4]).get_u32_le();
        phdr.p_offset = Elf32_Off((&bytes[4..8]).get_u32_le());
        phdr.p_vaddr = Elf32_Addr((&bytes[4..8]).get_u32_le());
        phdr.p_paddr = Elf32_Addr((&bytes[8..12]).get_u32_le());
        phdr.p_filesz = (&bytes[12..16]).get_u32_le();
        phdr.p_memsz = (&bytes[16..20]).get_u32_le();
        phdr._phdr.p_flags = (&bytes[20..24]).get_u32_le();
        phdr.p_align = (&bytes[24..28]).get_u32_le();
        phdr.self_elf = Some(elf as *const ELF32);
        phdr
    }

    fn get_type(&self) -> String {
        let p_type = self._phdr.p_type;
        if p_type == 0 {
            return "PT_NULL".to_string();
        } 
        else if p_type == 1 {
            return "PT_LOAD".to_string();
        }
        else if p_type == 2 {
            return "PT_DYNAMIC".to_string();
        }
        else if p_type == 3 {
            return "PT_INTERP".to_string();
        }
        else if p_type == 4 {
            return "PT_NOTE".to_string();
        }
        else if p_type == 5 {
            return "PT_SHLIB".to_string();
        }
        else if p_type == 6 {
            return "PT_PHDR".to_string();
        }
        else if p_type == 7 {
            return "PT_TLS".to_string();
        }
        else if p_type == 8 {
            return "PT_NUM".to_string();
        }
        "NULL".to_string()
    }
}

impl Elf64_Phdr {
    fn new(bytes : &Bytes, elf : &ELF64) -> Elf64_Phdr{
        let mut phdr = Elf64_Phdr::default();
        phdr._phdr.p_type = (&bytes[0..4]).get_u32_le();
        phdr.p_offset = Elf64_Off((&bytes[4..12]).get_u64_le());
        phdr.p_vaddr = Elf64_Addr((&bytes[12..20]).get_u64_le());
        phdr.p_paddr = Elf64_Addr((&bytes[20..28]).get_u64_le());
        phdr.p_filesz = (&bytes[28..36]).get_u64_le();
        phdr.p_memsz = (&bytes[36..44]).get_u64_le();
        phdr._phdr.p_flags = (&bytes[44..48]).get_u32_le();
        phdr.p_align = (&bytes[48..56]).get_u64_le();
        phdr.self_elf = Some(elf as *const ELF64);
        phdr
    }
}
#[derive(Default,Debug)]
pub struct Elf32_Shdr {
    sh_name     : u32,
    sh_type     : u32,
    sh_flags    : u32,
    sh_addr     : Elf32_Addr,
    sh_offset   : Elf32_Off,
    sh_size     : u32,
    sh_link     : u32,
    sh_info     : u32,
    sh_addralign: u32,
    sh_entsize  : u32,
    self_elf    : Option<*const ELF32>
}

#[derive(Default,Debug)]
pub struct Elf64_Shdr {
    sh_name     : u32,
    sh_type     : u32,
    sh_flags    : u64,
    sh_addr     : Elf64_Addr,
    sh_offset   : Elf64_Off,
    sh_size     : u64,
    sh_link     : u32,
    sh_info     : u32,
    sh_addralign: u64,
    sh_entsize  : u64,
    self_elf    : Option<*const ELF64>
}

impl Elf32_Shdr {
    fn new(bytes : &Bytes,elf : &ELF32) -> Elf32_Shdr {
        Elf32_Shdr { 
            sh_name: (&bytes[0..4]).get_u32(), 
            sh_type: (&bytes[4..8]).get_u32(), 
            sh_flags: (&bytes[8..12]).get_u32(), 
            sh_addr: Elf32_Addr((&bytes[12..16]).get_u32()), 
            sh_offset: Elf32_Off((&bytes[16..20]).get_u32()), 
            sh_size: (&bytes[20..24]).get_u32(), 
            sh_link: (&bytes[24..28]).get_u32(), 
            sh_info: (&bytes[28..32]).get_u32(), 
            sh_addralign: (&bytes[32..36]).get_u32(), 
            sh_entsize: (&bytes[36..40]).get_u32(),
            self_elf: Some(elf as *const ELF32)
        }
    }

    fn get_shname(&self) -> String {
        String::default()
    }
}

impl Elf64_Shdr {
    fn new(bytes : &Bytes,elf : &ELF64) -> Elf64_Shdr {
        Elf64_Shdr { 
            sh_name: (&bytes[0..4]).get_u32(), 
            sh_type: (&bytes[4..8]).get_u32(), 
            sh_flags: (&bytes[8..16]).get_u64(), 
            sh_addr: Elf64_Addr((&bytes[16..24]).get_u64()), 
            sh_offset: Elf64_Off((&bytes[24..32]).get_u64()), 
            sh_size: (&bytes[32..40]).get_u64(), 
            sh_link: (&bytes[40..44]).get_u32(), 
            sh_info: (&bytes[44..48]).get_u32(), 
            sh_addralign: (&bytes[48..56]).get_u64(), 
            sh_entsize: (&bytes[56..64]).get_u64(),
            self_elf: Some(elf as *const ELF64)
        }
    }
}
#[derive(Default,Debug)]
pub struct ELF32 {
    ehdr  : Elf32_Ehdr,
    phdrs : Vec<Elf32_Phdr>,
    shdrs : Vec<Elf32_Shdr>,
    data  : Bytes
}

impl ELF32 {
    fn get_phnum(&self) -> usize {
        return self.ehdr._ehdr.e_phnum as usize;
    }

    fn get_phsize(&self) -> usize {
        return self.ehdr._ehdr.e_phentsize as usize;
    }

    fn get_shnum(&self) -> usize {
        return self.ehdr._ehdr.e_shnum as usize;
    }

    fn get_shsize(&self) -> usize {
        return self.ehdr._ehdr.e_shentsize as usize;
    }
}

#[derive(Default,Debug)]
pub struct ELF64 {
    ehdr    : Elf64_Ehdr,
    phdrs   : Vec<Elf64_Phdr>,
    shdrs   : Vec<Elf64_Shdr>,
    data    : Bytes
}

impl ELF64 {
    fn get_phnum(&self) -> usize {
        return self.ehdr._ehdr.e_phnum as usize;
    }

    fn get_phsize(&self) -> usize {
        return self.ehdr._ehdr.e_phentsize as usize;
    }

    fn get_shnum(&self) -> usize {
        return self.ehdr._ehdr.e_shnum as usize;
    }

    fn get_shsize(&self) -> usize {
        return self.ehdr._ehdr.e_shentsize as usize;
    }


    fn get_strtab_section(&self) -> Range<usize> {
        for i in 0..self.get_shnum() {
            if self.shdrs[i].sh_type == ShdrType::SHT_STRTAB as u32 {

            }
        }
        unimplemented!()
    }
}

impl ELF32 {
    pub fn new(file : String) -> ELF32 {
        let mut elf = ELF32::default();
        let f = fs::read(file).expect("no such a file");
        let bytes = Bytes::from(f);
        elf.ehdr = Elf32_Ehdr::new(&bytes);
        let phdr_offset = elf.ehdr.get_phdr_offset().0 as usize;
        let mut start = 0 as usize;
        for _ in 0..elf.get_phnum() {
            let phdr_bytes = bytes.slice(phdr_offset+start..bytes.len());
            elf.phdrs.push(Elf32_Phdr::new(&phdr_bytes,&elf));
            start += elf.get_phsize();
        }
        let shdr_offset = elf.ehdr.get_shdr_offset().0 as usize;
        let shdr_bytes = bytes.slice(shdr_offset..bytes.len());
        elf.shdrs.push(Elf32_Shdr::new(&shdr_bytes,&elf));
        start = 0;
        for _ in 0..elf.get_shnum() {
            let shdr_bytes = bytes.slice(shdr_offset+start..bytes.len());
            elf.shdrs.push(Elf32_Shdr::new(&shdr_bytes,&elf));
            start += elf.get_shsize();
        }
        elf.data = bytes;
        elf
    }

    pub fn get_ehdr (&self) -> &Elf32_Ehdr {
        &self.ehdr
    }
}


impl ELF64 {
    pub fn new(file : String) -> ELF64 {
        let mut elf = ELF64::default();
        let f = fs::read(file).expect("no such a file");
        let bytes = Bytes::from(f);
        elf.ehdr = Elf64_Ehdr::new(&bytes);
        let phdr_offset = elf.ehdr.get_phdr_offset().0 as usize;
        let mut start = 0 as usize;
        for _ in 0..elf.get_phnum() {
            let phdr_bytes = bytes.slice(phdr_offset+start..bytes.len());
            elf.phdrs.push(Elf64_Phdr::new(&phdr_bytes,&elf));
            start += elf.get_phsize();
        }
        let shdr_offset = elf.ehdr.get_shdr_offset().0 as usize;
        let shdr_bytes = bytes.slice(shdr_offset..bytes.len());
        elf.shdrs.push(Elf64_Shdr::new(&shdr_bytes, &elf));
        start = 0;
        for _ in 0..elf.get_shnum() {
            let shdr_bytes = bytes.slice(shdr_offset+start..bytes.len());
            elf.shdrs.push(Elf64_Shdr::new(&shdr_bytes, &elf));
            start += elf.get_shsize();
        }
        elf.data = bytes;
        elf
    }
}