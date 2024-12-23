use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bytes::Bytes;

use crate::utils::file::MRFile;
use std::ops::Range;
pub mod ntfs_impl;
pub mod mft_impl;
pub mod journal_impl;
pub mod fs_impl;
pub mod bitmap_impl;

pub struct Ntfs {
    start_with                  : Vec<u8>,
    boot_entry_point            : Vec<u8>,
    sectors_per_cluster_block   : u8,
    pub bytes_per_sector            : u16,
    pub total_sectors               : u64,
    mft_block_number            : u64,
    mft_mirror_block_number     : u64,
    mft_entry_size              : usize,
    index_entry_size            : u8,
    is_bitlocker                : bool,
    version                     : Option<(u8,u8)>,
    reader                      : MRFile,
    datas_of_mft                : RefCell<Vec<DataDescriptor>>,
    cache_mfts                  : Option<Vec<(Range<usize>, Rc<MFTEntry>)>>
}

#[derive(Clone, Copy, Debug)]
pub struct CResident {
    data_size       : u32,
    data_offset     : u16,
    indexed_flag    : u8,
    padding         : u8
}

#[derive(Clone, Copy, Debug)]
pub struct CNonResident {
    first_vcn       : u64,
    last_vcn        : u64,
    data_run_offset : u16,
    compression_unit_size   : u16,
    allocated_data_size     : u64,
    data_size               : u64,
    valid_data_size         : u64,
    total_allocated_size    : Option<u64>
}


#[derive(Debug)]
pub enum CCommon {
    Resident(CResident),
    NonResident(CNonResident)
}

#[derive(Debug, Clone)]
pub struct FileTime {
    low     : u32,
    high    : u32
}


#[derive(Debug)]
pub struct Value10_StandardInfomation {
    file_create_time    : FileTime,
    file_change_time    : FileTime,
    mft_change_time     : FileTime,
    file_last_visited   : FileTime,
    file_attr_flags     : Option<u32>,
    owner_id            : Option<u32>,
    security_id         : Option<u32>,
    quota_charged       : Option<u64>,
    update_sequence_num : Option<u64>,
}

#[derive(Debug)]
pub struct V20Attr {
    attribute_type      : u32,
    size                : u16,
    name_size           : u8,
    name_offset         : u8,
    data_vcn            : u64,
    file_reference      : FileReference,
    attribute_identifier: u16,
    name                : String
}

#[derive(Debug)]
pub struct Value20_AttributeList {
    list        : Option<Vec<V20Attr>>
}

#[derive(Debug, Clone)]
pub struct Value30_FileName {
    parent_file_num     : u64,
    create_time         : FileTime,
    change_time         : FileTime,
    mft_change_time     : FileTime,
    last_visit_time     : FileTime,
    alloc_size          : u64,
    real_size           : u64,
    file_flag           : u32,
    ea_flag             : u32,
    name_length         : u8,
    name_space          : u8,
    name                : String
}

#[derive(Debug)]
pub struct Value40_ObjectId {
    droid_file_identify         : u128,
    birth_droid_vol_identify    : u128,
    birth_droid_file_identify   : u128,
    birth_droid_domain_identify : u128
}

#[derive(Debug)]
pub struct Value50_SecurityDescriptor {

}

#[derive(Debug)]
pub struct Value60_VolumeName {

}

#[derive(Debug)]
pub struct Value70_VolumeInfomation {
    majar_version   : u8,
    minor_version   : u8,
    volume_flags    : u16
}

#[derive(Debug, Clone)]
pub struct DataDescriptor {
    datasize    : u64,
    start_addr  : u64,
}

impl DataDescriptor {
    pub fn get_datasize(&self) -> u64 {
        self.datasize
    }

    pub fn get_start_addr(&self) -> u64 {
        self.start_addr
    }
}

#[derive(Debug)]
pub struct Value80_Data {
    datas       : Vec<DataDescriptor>
}

#[derive(Debug)]
pub struct IndexRootHeader {
    attr_type       : u32,
    collation_type       : u32,
    index_entry_size            : u32,
    index_entry_number_cluser   : u32
}

#[derive(Debug)]
pub struct IndexEntryHeader {
    fix_up_value_offset     : u16,
    number_of_fix_up_values : u16,
    journal_sequence        : u64,
    vcn_of_index_entry      : u64
}

#[derive(Debug)]
pub struct IndexNodeHeader {
    index_values_offset     : u32,
    index_node_size         : u32,
    allocated_index_node_size   : u32,
    index_node_flags            : u32
}

#[derive(Debug)]
pub struct IndexValue {
    file_reference      : FileReference,
    index_value_size    : u16,
    index_key_data_size : u16,
    index_value_flags   : u32,
    index_key_data      : Option<Value30_FileName>,
    index_value_data    : Option<Vec<u8>>,
    sub_node_vcn        : Option<u64>,
}

#[derive(Debug)]
pub struct FileItem {
    mft_index       : u64,
    name            : Value30_FileName
}

#[derive(Debug)]
pub struct FileReference {
    mft_index       : u64,
    sequence_num    : u16
}

#[derive(Debug)]
pub struct Value90_IndexRoot {
    root_header     : IndexRootHeader,
    node_header     : IndexNodeHeader,
    values          : Vec<IndexValue>
}

#[derive(Debug)]
pub struct FixupValue {

}

#[derive(Debug)]
pub struct ValueA0_IndexAlloction {
    offset      : u64,
    size        : u64,
    entry_header: RefCell<Option<Vec<IndexEntryHeader>>>,
    node_header : RefCell<Option<Vec<IndexNodeHeader>>>,
    values      : RefCell<Option<Vec<IndexValue>>>,
    ntfs        : Option<*const Ntfs>
}

#[derive(Debug)]
pub struct ValueB0_Bitmap {

}

#[derive(Debug)]
pub struct ValueC0_SymbolicLink {

}

#[derive(Debug)]
pub struct Value100_LoggedUtilityStream {

}


#[derive(Debug)]
pub enum MFTValue {
    StdInfo(Value10_StandardInfomation),
    AttrList(Value20_AttributeList),
    FileName(Value30_FileName),
    ObjectId(Value40_ObjectId),
    SecurityDescriptor(Value50_SecurityDescriptor),
    VolumeName(Value60_VolumeName),
    VolumeInfo(Value70_VolumeInfomation),
    Data(Value80_Data),
    IndexRoot(Value90_IndexRoot),
    IndexAlloc(ValueA0_IndexAlloction),
    Bitmap(ValueB0_Bitmap),
    SymbolicLink(ValueC0_SymbolicLink),
    LoggedUtilityStream(Value100_LoggedUtilityStream),
    None
}


#[derive(Debug)]
pub struct MFTAttribute {
    mft_type            : u32,
    length              : u16,
    non_resident_flag   : u8,
    name_length         : u8,
    name_offset         : u16,
    attribute_flags     : u16,
    identity            : u16,
    common              : CCommon,
    value               : MFTValue,

    attr_name           : String
}

#[derive(Debug)]
pub struct MFTStream {
    name        : String,
    data        : Value80_Data
}

#[derive(Debug)]
pub struct MFTEntry {
    parent_index                : RefCell<i64>,
    index                       : u64,
    fix_up_value_offset         : u16,
    number_fix_up_values        : u16,
    journal_sequence_number     : u64,
    sequence                    : u16,
    reference_count             : u16,
    attributes_offset           : u16,
    entry_flags                 : u16,
    used_size                   : u32,
    total_size                  : u32,
    map_attr_chains             : HashMap<u32,Vec<MFTAttribute>>,
    ntfs                        : Option<*const Ntfs>
}

pub struct LFSRestartPageHeader {
    signature               : String,
    fix_up_values_offset    : u16,
    fix_up_values_number    : u16,
    checkdisk_last_lsn      : u64,
    system_page_size        : u32,
    log_page_size           : u32,
    restart_offset          : u16,
    minor_format_version    : u16,
    major_format_version    : u16
}

enum LFSRecordType {

}

pub struct ClientId {
    seq_number      : u16,
    client_index    : u16
}

pub struct LFSRecordHeader {
    meta_trans_journal_seq_number       : u64,
    pre_meta_trans_journal_seq_number   : u64,
    undo_meta_trans_journal_seq_number  : u64,

    client_data_length                  : u64,
    client_id                           : u64,
    record_type                         : LFSRecordType,
    flags                               : u16,
}

pub struct USNChangeJournalMetadata {
    maximum_data        : u64,
    allocation_data     : u64,
    usn_identifier      : FileTime
}

pub enum USNIdentifier {
    USN_REASON_DATA_OVERWRITE,
    USN_REASON_DATA_EXTEND,
    USN_REASON_DATA_TRUNCATION,
    USN_REASON_NAMED_DATA_OVERWRITE,
    USN_REASON_NAMED_DATA_EXTEND,
    USN_REASON_NAMED_DATA_TRUNCATION,
    USN_REASON_FILE_CREATE,
    USN_REASON_FILE_DELETE,
    USN_REASON_EA_CHANGE,
    USN_REASON_SECURITY_CHANGE,
    USN_REASON_RENAME_OLD_NAME,
    USN_REASON_RENAME_NEW_NAME,
    USN_REASON_INDEXABLE_CHANGE,
    USN_REASON_BASIC_INFO_CHANGE,
    USN_REASON_HARD_LINK_CHANGE,
    USN_REASON_COMPRESSION_CHANGE,
    USN_REASON_ENCRYPTION_CHANGE,	
    USN_REASON_OBJECT_ID_CHANGE,
    USN_REASON_REPARSE_POINT_CHANGE,
    USN_REASON_STREAM_CHANGE,
    USN_REASON_TRANSACTED_CHANGE,
    USN_REASON_CLOSE
}

#[derive(Debug)]
pub struct FileReference128 {
    mft_index       : u64,
    seq_number      : u64
}

#[derive(Debug)]
pub struct USNChangeJournalEntry {
    entry_size          : u32,
    major_version       : u16,
    minor_version       : u16,
    reference           : FileReference128,
    parent_reference    : FileReference128,
    usn                 : u64,
    update_date         : FileTime,
    update_reason_flags : u32,
    update_source_flags : u32,
    security_descriptor_id  : u32,
    file_attributes_flags   : u32,
    name_size               : u16,
    name_offset             : u16,
    name                : String
}

pub struct USNChangeJournal  {
    mft     : MFTEntry,
    ntfs    : Option<*const Ntfs>
}

pub struct Bitmap {
    mft     : MFTEntry,
    ntfs    : Option<*const Ntfs>
}