use bytes::{Bytes, Buf};
use crate::utils::{file::MRFile, MRError, funcs::i_to_m};

use super::elf64::{get_str_to_zero};
use super::elf_pub::{ElfN_Ehdr,ElfN_Phdr, ShdrType};

#[derive(Default, Debug)]
pub struct Elf32_Addr(pub u32);

#[derive(Default, Debug)]
pub struct Elf32_Off(pub u32);

#[derive(Default, Debug)]
pub struct Elf32_Ehdr {
    e_entry: Elf32_Addr,
    e_phoff: Elf32_Off,
    e_shoff: Elf32_Off,
    _ehdr: ElfN_Ehdr,
}

impl Elf32_Ehdr {
    fn new(bytes: &Bytes) -> Elf32_Ehdr {
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

        return "";
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

#[derive(Default, Debug)]
pub struct Elf32_Phdr {
    p_offset: Elf32_Off,
    p_vaddr: Elf32_Addr,
    p_paddr: Elf32_Addr,
    p_filesz: u32,
    p_memsz: u32,
    p_align: u32,
    _phdr: ElfN_Phdr,
    self_elf: Option<*const ELF32>,
}

impl Elf32_Phdr {
    fn new(bytes: &Bytes, elf: &ELF32) -> Elf32_Phdr {
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

    fn get_type(&self) -> &str {
        let p_type = self._phdr.p_type;
        if p_type == 0 {
            return "PT_NULL";
        } else if p_type == 1 {
            return "PT_LOAD";
        } else if p_type == 2 {
            return "PT_DYNAMIC";
        } else if p_type == 3 {
            return "PT_INTERP";
        } else if p_type == 4 {
            return "PT_NOTE";
        } else if p_type == 5 {
            return "PT_SHLIB";
        } else if p_type == 6 {
            return "PT_PHDR";
        } else if p_type == 7 {
            return "PT_TLS";
        } else if p_type == 8 {
            return "PT_NUM";
        }
        "NULL"
    }
}

#[derive(Default, Debug)]
pub struct Elf32_Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u32,
    sh_addr: Elf32_Addr,
    sh_offset: Elf32_Off,
    sh_size: u32,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u32,
    sh_entsize: u32,
    self_elf: Option<*const ELF32>,
    data    : Bytes,
    name    : String
}

impl Elf32_Shdr {
    fn new(bytes: &Bytes, elf: &ELF32) -> Elf32_Shdr {
        Elf32_Shdr {
            sh_name: (&bytes[0..4]).get_u32_le(),
            sh_type: (&bytes[4..8]).get_u32_le(),
            sh_flags: (&bytes[8..12]).get_u32_le(),
            sh_addr: Elf32_Addr((&bytes[12..16]).get_u32_le()),
            sh_offset: Elf32_Off((&bytes[16..20]).get_u32_le()),
            sh_size: (&bytes[20..24]).get_u32_le(),
            sh_link: (&bytes[24..28]).get_u32_le(),
            sh_info: (&bytes[28..32]).get_u32_le(),
            sh_addralign: (&bytes[32..36]).get_u32_le(),
            sh_entsize: (&bytes[36..40]).get_u32_le(),
            self_elf: Some(elf as *const ELF32),
            data    : Bytes::default(),
            name    : "".to_string()
        }
    }

    pub fn get_name(&self) -> &String {
        unsafe {
            if self.name.len() != 0 {
                return &self.name;
            }
            let data = &*(*self.self_elf.unwrap()).get_shtab();
            let mut index = self.sh_name as usize;
            let start = index;
            while index < data.len() {
                if data[index] != 0 {
                    index += 1;
                } else {
                    break;
                }
            }

            i_to_m(self).name = std::str::from_utf8(&data[start..index].to_vec())
                .unwrap_or(&"None".to_string()).to_string();
            &self.name
        }
    }

    fn get_elf(&self) -> &ELF32 {
        unsafe {
            &*self.self_elf.unwrap()
        }
    }

    pub fn get_data(&self) -> &Bytes {
        unsafe {
            if self.data.len() != 0 {
                return &self.data;
            }
            let elf = self.get_elf();
            let data = elf.get_elf();
            //i_to_m(self).data = data.slice(self.sh_offset.0 as usize..(self.sh_offset.0+self.sh_size) as usize);
            i_to_m(self).data = Bytes::from(elf.get_elf().read_n(self.sh_offset.0 as usize, self.sh_size as usize).unwrap());
            return &self.data;
        }
    }
}

pub struct Elf32_Sym {
    st_name     : u32,
    st_value    : Elf32_Addr,
    st_size     : u32,
    st_info     : u8,
    st_other    : u8,
    st_shndx    : u16,
    self_elf    : Option<*const ELF32>
}

impl Elf32_Sym {
    pub fn parse(bytes: &Bytes,elf: *const ELF32) -> Elf32_Sym {
        Elf32_Sym {
            st_name : (&bytes[0..4]).get_u32_le(),
            st_value : Elf32_Addr((&bytes[4..8]).get_u32_le()),
            st_size : (&bytes[8..12]).get_u32_le(),
            st_info : (&bytes[12..13]).get_u8(),
            st_other : (&bytes[13..14]).get_u8(),
            st_shndx  : (&bytes[14..16]).get_u16_le(),
            self_elf  : Some(elf as *const ELF32)
        }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let elf = &(*self.self_elf.unwrap());
            let str_data = elf.get_strtab_data();
            return get_str_to_zero(&str_data, self.st_name as usize).unwrap_or("".to_string());
        }
    }

    pub fn get_value(&self) -> &Elf32_Addr {
        &self.st_value
    }
}

#[derive(Default, Debug)]
pub struct ELF32 {
    ehdr: Elf32_Ehdr,
    phdrs: Vec<Elf32_Phdr>,
    shdrs: Vec<Elf32_Shdr>,
    shtab: Vec<u8>,
    symstrtab : Vec<u8>,
    elf_file    : MRFile
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

    pub fn get_shtab(&self) -> &Vec<u8> {
        &self.shtab
    }

    pub fn get_strtab(&self) -> &Vec<u8> {
        &self.shtab
    }

    pub fn get_elf(&self) -> &MRFile {
        return &self.elf_file;
    }

    pub fn get_shdrs(&self) -> &Vec<Elf32_Shdr> {
        return &self.shdrs;
    }

    pub fn get_symtab(&self) -> Result<&Elf32_Shdr,MRError> {
        for shdr in &self.shdrs {
            if shdr.sh_type == ShdrType::SHT_SYMTAB as u32 {
                return Ok(shdr)
            }
        }
        Err(MRError::new("Not found symtab"))
    }

    pub fn get_syms(&self) -> Result<Vec<Elf32_Sym>,MRError> {
        let mut result: Vec<Elf32_Sym> = Vec::default();
        let symtab = self.get_symtab();
        let symtab = match symtab {
            Ok(tab) => tab,
            Err(err) => {
                return Err(err);
            }
        };

        let data = symtab.get_data();
        let mut index = 0;
        while index <= data.len()-16 {
            let sym = Elf32_Sym::parse(&data.slice(index..index+16),self);
            result.push(sym);
            index += 16;
        }
        Ok(result)
    }

    pub fn get_strtab_data(&self) -> &Vec<u8> {
        if self.symstrtab.len() != 0 {
            return &self.symstrtab;
        }
        let mut result: Vec<u8> = Vec::default();
        i_to_m(self).symstrtab = self.shdrs[self.get_shnum()-1].get_data().to_vec();
        &self.symstrtab
    }
}

impl ELF32 {
    pub fn new(file: String) -> ELF32 {
        let mut elf = ELF32::default();
        let file2 = file.clone();
        
        let mr_f = MRFile::new(file2.as_str()).unwrap();
        elf.ehdr = Elf32_Ehdr::new(&Bytes::from(mr_f.read_n(0, 52).unwrap()));
        let phdr_offset = elf.ehdr.get_phdr_offset().0 as usize;
        let mut start = 0 as usize;
        for _ in 0..elf.get_phnum() {
            let phdr_bytes = mr_f.read_n(phdr_offset + start, elf.get_phsize()).unwrap();
            elf.phdrs.push(Elf32_Phdr::new(&Bytes::from(phdr_bytes), &elf));
            start += elf.get_phsize();
        }
        let shdr_offset = elf.ehdr.get_shdr_offset().0 as usize;
        let shdr_bytes = mr_f.read_n(shdr_offset, 40).unwrap();
        elf.shdrs.push(Elf32_Shdr::new(&Bytes::from(shdr_bytes), &elf));
        start = 0;
        for _ in 0..elf.get_shnum() {
            let shdr_bytes = mr_f.read_n(shdr_offset + start, 40).unwrap();
            elf.shdrs.push(Elf32_Shdr::new(&Bytes::from(shdr_bytes), &elf));
            start += elf.get_shsize();
        }
        let section = &elf.shdrs[elf.get_shnum()];

        elf.shtab = mr_f.read_n(section.sh_offset.0 as usize, section.sh_size as usize).unwrap();
        elf.elf_file = mr_f;
        elf
    }

    pub fn get_ehdr(&self) -> &Elf32_Ehdr {
        &self.ehdr
    }

    
}