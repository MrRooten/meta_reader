use std::io::BufReader;

use crate::utils::file::MRFile;
pub mod reg_impl;

pub struct RegValueData {

}

pub struct RegValuesLIst {

}

pub enum RegDataType {
    REG_NONE=0x00000000,
    REG_SZ,
    REG_EXPAND_SZ,
    REG_BINARY,
    REG_DWORD,
    REG_DWORD_BIG_ENDIAN,
    REG_LINK,
    REG_MULTI_SZ,
    REG_RESOURCE_LIST,
    REG_FULL_RESOURCE_DESCRIPTOR,
    REG_RESOURCE_REQUIREMENT_LIST,
    REG_QWORD
}

pub struct RegValueKey {
    sign        : [u8;2],
    name_size   : u16,
    data_size   : u32,
    data_offset : u32,
    data_type   : u32,
    flags       : u16,
    v_name      : String
}

struct RI_SubKey {

}

struct LI_SubKey {

}

struct LH_SubKey {

}

struct LF_SubKey {

}

pub struct RegSubKeyList {
    sign            : [u8;2],
    num_of_elems    : u16 
}

pub struct RegNtSecurityDescriptorr {

}

pub struct RegSecurityKey {
    sign        : [u8;2],
    prev_sec_key_offset     : u32,
    next_sec_key_offset     : u32,
    reference_count         : u32,
    nt_sec_desc_size        : u32,

}

pub struct RegNamedKey {
    sign        : [u8;2],
    flags       : u16,
    last_written_time   : u64,
    parent_key_offset   : u32,
    subkeys_num         : u32,
    volatile_subkeys_num        : u32,
    subkeys_list_offset         : u32,
    volatile_subkeys_list_offset: u32,
    number_of_values            : u32,
    values_list_offset          : u32,
    security_key_offset         : u32,
    class_name_offset           : u32,
    largest_sub_key_name_size   : u32,
    largest_sub_key_class_name_size : u32,
    largest_value_size          : u32,
    key_name_size               : u32,
    class_name_size             : u32,
    key_name_string             : String
}

pub struct HiveBinCell {
    size                : u32,
    offset_from_start   : u32
}

pub struct HiveBin {
    sign                : Vec<u8>,
    offset              : u32,
    size                : u32,
    offset_of_file      : u32 
}

pub struct RegFileHeader {
    sign                : Vec<u8>,
    primary_seq_num     : u32,
    second_seq_num      : u32,
    last_modify         : u64,
    major_version       : u32,
    minor_version       : u32,
    root_key_offset     : u32,
    hive_bins_data_size : u32,
}

pub struct RegFile {
    header  : RegFileHeader,
    file    : MRFile
}