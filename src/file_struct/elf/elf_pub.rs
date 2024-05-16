#[derive(Default, Debug)]
pub struct ElfN_Ehdr {
    pub e_ident: [u8; 16],
    pub e_machine: u16,
    pub e_type: u16,
    pub e_version: u32,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shnum: u16,
    pub e_shentsize: u16,
    pub e_shstrndx: u16,
}


#[derive(Default, Debug)]
pub struct ElfN_Phdr {
    pub p_type: u32,
    pub p_flags: u32,
}

#[allow(clippy::enum_clike_unportable_variant)]
pub enum ShdrType {
    SHT_NULL = 0,
    SHT_PROGBITS = 1,
    SHT_SYMTAB = 2,
    SHT_STRTAB = 3,
    SHT_RELA = 4,
    SHT_HASH = 5,
    SHT_DYNAMIC = 6,
    SHT_NOTE = 7,
    SHT_NOBITS = 8,
    SHT_REL = 9,
    SHT_SHLIB = 11,
    SHT_DYNSYM = 12,
    SHT_LOPROC = 0x70000000,
    SHT_HIPROC = 0x7fffffff,
    SHT_LOUSER = 0x80000000,
    SHT_HIUSER = 0xffffffff,
}