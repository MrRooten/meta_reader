use std::{borrow::Borrow, cell::RefCell, fs, io::BufRead, ops::Range};

use bytes::{Buf, Bytes, BytesMut};

use super::elf_pub::{ElfN_Ehdr, ElfN_Phdr, ShdrType};
use crate::utils::{file::MRFile, MRError};

#[derive(Default, Debug)]
pub struct Elf64_Addr(pub u64);

#[derive(Default, Debug)]
pub struct Elf64_Off(pub u64);

#[derive(Default, Debug)]
pub struct Elf64_Ehdr {
    e_entry: Elf64_Addr,
    e_phoff: Elf64_Off,
    e_shoff: Elf64_Off,
    _ehdr: ElfN_Ehdr,
}

impl Elf64_Ehdr {
    fn new(bytes: &Bytes) -> Elf64_Ehdr {
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

    fn get_type(&self) -> &str {
        let t = self._ehdr.e_type;
        if t == 0 {
            return "ET_None";
        } else if t == 1 {
            return "ET_REL";
        } else if t == 2 {
            return "ET_EXEC";
        } else if t == 3 {
            return "ET_DYN";
        } else if t == 4 {
            return "ET_CORE";
        } else if t == 5 {
            return "ET_NUM";
        }

        ""
    }

    fn get_machine(&self) -> &str {
        ""
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

#[derive(Default, Debug)]
pub struct Elf64_Phdr {
    p_offset: Elf64_Off,
    p_vaddr: Elf64_Addr,
    p_paddr: Elf64_Addr,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
    _phdr: ElfN_Phdr,
    self_elf: Option<*const ELF64>,
}

impl Elf64_Phdr {
    fn new(bytes: &Bytes, elf: &ELF64) -> Elf64_Phdr {
        let mut phdr = Elf64_Phdr::default();
        phdr._phdr.p_type = (&bytes[0..4]).get_u32_le();
        phdr._phdr.p_flags = (&bytes[4..8]).get_u32_le();
        phdr.p_offset = Elf64_Off((&bytes[8..16]).get_u64_le());
        phdr.p_vaddr = Elf64_Addr((&bytes[16..24]).get_u64_le());
        phdr.p_paddr = Elf64_Addr((&bytes[24..32]).get_u64_le());
        phdr.p_filesz = (&bytes[32..40]).get_u64_le();
        phdr.p_memsz = (&bytes[40..48]).get_u64_le();
        phdr.p_align = (&bytes[48..56]).get_u64_le();
        phdr.self_elf = Some(elf as *const ELF64);
        phdr
    }
}

#[derive(Default, Debug)]
pub struct Elf64_Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: Elf64_Addr,
    sh_offset: Elf64_Off,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
    self_elf: Option<*const ELF64>,
    data: RefCell<Bytes>,
    name: RefCell<String>,
}

impl Elf64_Shdr {
    fn new(bytes: &Bytes, elf: &ELF64) -> Elf64_Shdr {
        Elf64_Shdr {
            sh_name: (&bytes[0..4]).get_u32_le(),
            sh_type: (&bytes[4..8]).get_u32_le(),
            sh_flags: (&bytes[8..16]).get_u64_le(),
            sh_addr: Elf64_Addr((&bytes[16..24]).get_u64_le()),
            sh_offset: Elf64_Off((&bytes[24..32]).get_u64_le()),
            sh_size: (&bytes[32..40]).get_u64_le(),
            sh_link: (&bytes[40..44]).get_u32_le(),
            sh_info: (&bytes[44..48]).get_u32_le(),
            sh_addralign: (&bytes[48..56]).get_u64_le(),
            sh_entsize: (&bytes[56..64]).get_u64_le(),
            self_elf: Some(elf as *const ELF64),
            data: RefCell::new(Bytes::default()),
            name: RefCell::new("".to_string()),
        }
    }

    pub fn get_name(&self) -> &RefCell<String> {
        if !self.name.borrow().is_empty() {
            return &self.name;
        }
        let data = self.get_elf().get_shtab();
        let mut index = self.sh_name as usize;
        let start = index;
        while index < data.len() {
            if data[index] != 0 {
                index += 1;
            } else {
                break;
            }
        }

        self.name.replace(
            std::str::from_utf8(&data[start..index])
                .unwrap_or("None")
                .to_string(),
        );
        &self.name
    }

    fn get_elf(&self) -> &ELF64 {
        unsafe { &*self.self_elf.unwrap() }
    }

    pub fn get_data(&self) -> &RefCell<Bytes> {
        if !self.data.borrow().is_empty() {
            return &self.data;
        }
        let elf = self.get_elf();
        let data = elf.get_elf();
        self.data.replace(Bytes::from(
            elf.get_elf()
                .read_n(self.sh_offset.0 as usize, self.sh_size as usize)
                .unwrap(),
        ));
        &self.data
    }
}

pub struct Elf64_Sym {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: Elf64_Addr,
    st_size: u64,
    self_elf: Option<*const ELF64>,
}

pub fn get_str_to_zero(v: &[u8], i: usize) -> Result<String, MRError> {
    let mut index = i;
    let start = index;

    while index < v.len() {
        if v[index] != 0 {
            index += 1;
        } else {
            break;
        }
    }
    let bind = v[start..index].to_vec();

    let res = String::from_utf8(bind).expect("msg");

    Ok(res)
}

impl Elf64_Sym {
    pub fn parse(bytes: &Bytes, elf: &ELF64) -> Elf64_Sym {
        Elf64_Sym {
            st_name: (&bytes[0..4]).get_u32_le(),
            st_info: (&bytes[4..5]).get_u8(),
            st_other: (&bytes[5..6]).get_u8(),
            st_shndx: (&bytes[6..8]).get_u16_le(),
            st_value: Elf64_Addr((&bytes[8..16]).get_u64_le()),
            st_size: (&bytes[16..24]).get_u64_le(),
            self_elf: Some(elf as *const ELF64),
        }
    }

    pub fn get_name(&self) -> String {
        let elf = unsafe { &(*self.self_elf.unwrap()) };
        let str_data = elf.get_strtab_data();
        get_str_to_zero(&str_data.borrow(), self.st_name as usize).unwrap_or("".to_string())
    }

    pub fn get_value(&self) -> &Elf64_Addr {
        &self.st_value
    }
}

#[derive(Default, Debug)]
pub struct ELF64 {
    ehdr: Elf64_Ehdr,
    phdrs: Vec<Elf64_Phdr>,
    shdrs: Vec<Elf64_Shdr>,
    shtab: Vec<u8>,
    symstrtab: RefCell<Vec<u8>>,
    elf_file: Option<MRFile>,
}

impl ELF64 {
    fn get_phnum(&self) -> usize {
        self.ehdr._ehdr.e_phnum as usize
    }

    fn get_phsize(&self) -> usize {
        self.ehdr._ehdr.e_phentsize as usize
    }

    fn get_shnum(&self) -> usize {
        self.ehdr._ehdr.e_shnum as usize
    }

    fn get_shsize(&self) -> usize {
        self.ehdr._ehdr.e_shentsize as usize
    }

    pub fn get_shtab(&self) -> &Vec<u8> {
        &self.shtab
    }

    pub fn get_elf(&self) -> &MRFile {
        self.elf_file.as_ref().unwrap()
    }

    pub fn get_shdrs(&self) -> &Vec<Elf64_Shdr> {
        &self.shdrs
    }

    pub fn get_symtab(&self) -> Result<&Elf64_Shdr, MRError> {
        for shdr in &self.shdrs {
            if shdr.sh_type == ShdrType::SHT_SYMTAB as u32 {
                return Ok(shdr);
            }
        }
        Err(MRError::new("Not found symtab"))
    }

    pub fn get_syms(&self) -> Result<Vec<Elf64_Sym>, MRError> {
        let mut result: Vec<Elf64_Sym> = Vec::default();
        let symtab = self.get_symtab();
        let symtab = match symtab {
            Ok(tab) => tab,
            Err(err) => {
                return Err(err);
            }
        };

        let data = symtab.get_data();
        let mut index = 0;
        while index <= data.borrow().len() - 24 {
            let sym = Elf64_Sym::parse(&data.borrow().slice(index..index + 24), self);
            result.push(sym);
            index += 24;
        }
        Ok(result)
    }

    pub fn get_strtab_data(&self) -> &RefCell<Vec<u8>> {
        if !self.symstrtab.borrow().is_empty() {
            return &self.symstrtab;
        }
        let mut result: Vec<u8> = Vec::default();
        self.symstrtab.replace(
            self.shdrs[self.get_shnum() - 1]
                .get_data()
                .borrow()
                .to_vec(),
        );
        &self.symstrtab
    }

    pub fn get_comment(&self) -> Result<String, MRError> {
        let shdrs = self.get_shdrs();
        let mut commend_section = None::<&Elf64_Shdr>;
        for shdr in shdrs {
            if shdr.sh_type == ShdrType::SHT_PROGBITS as u32 {
                commend_section = Some(shdr);
            }
        }

        if commend_section.is_none() {
            return Err(MRError::new("No commend section of this elf"));
        }

        let commend_section = commend_section.unwrap();
        let s = std::str::from_utf8(&commend_section.get_data().borrow())
            .unwrap()
            .to_string();
        Ok(s)
    }
}

impl ELF64 {
    pub fn new(file: String) -> ELF64 {
        let mut elf = ELF64::default();
        let file2 = file.clone();

        let mr_f = MRFile::new(file2.as_str()).unwrap();
        elf.ehdr = Elf64_Ehdr::new(&Bytes::from(mr_f.read_n(0, 64).unwrap()));
        let phdr_offset = elf.ehdr.get_phdr_offset().0 as usize;
        let mut start = 0_usize;
        for _ in 0..elf.get_phnum() {
            let phdr_bytes = mr_f.read_n(phdr_offset + start, elf.get_phsize()).unwrap();
            elf.phdrs
                .push(Elf64_Phdr::new(&Bytes::from(phdr_bytes), &elf));
            start += elf.get_phsize();
        }
        let shdr_offset = elf.ehdr.get_shdr_offset().0 as usize;
        let shdr_bytes = mr_f.read_n(shdr_offset, 64).unwrap();
        elf.shdrs
            .push(Elf64_Shdr::new(&Bytes::from(shdr_bytes), &elf));
        start = 0;
        for _ in 0..elf.get_shnum() {
            let shdr_bytes = mr_f.read_n(shdr_offset + start, 64).unwrap();
            elf.shdrs
                .push(Elf64_Shdr::new(&Bytes::from(shdr_bytes), &elf));
            start += elf.get_shsize();
        }
        let section = &elf.shdrs[elf.get_shnum()];

        elf.shtab = mr_f
            .read_n(section.sh_offset.0 as usize, section.sh_size as usize)
            .unwrap();
        elf.elf_file = Some(mr_f);
        elf
    }
}
