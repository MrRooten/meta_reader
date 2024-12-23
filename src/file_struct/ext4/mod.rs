use std::{cell::{Cell, RefCell, UnsafeCell}, ops::Range};

use bytes::Bytes;

use crate::utils::file::MRFile;
pub mod ext4_impl;
pub mod group_descriptor_impl;
pub mod inode_table_impl;
pub mod journal_impl;
pub mod fs_impl;
#[derive(Debug,Default)]
pub struct Ext4 {
    reader                  : Option<MRFile>,
    super_block             : RefCell<Option<SuperBlock>>,
    group_descriptors       : RefCell<Option<Vec<GroupDescriptor>>>,
    block_size              : Cell<usize>
}

#[derive(Debug,Default, Clone, Copy)]
pub struct SuperBlock {
    pub s_inodes_count      : u32,          //0x0
    pub s_block_count       : u32,          //0x4  
    pub s_log_block_size    : u32,          //0x18
    pub s_log_cluster_size  : u32,          //0x1c
    s_blocks_per_group      : u32,          //0x20
    s_clusters_per_group    : u32,          //0x24
    s_inodes_per_group      : u32,          //0x28
    s_creator_os            : u32,          //0x48
    s_inode_size            : u16,
    s_uuid                  : [u8;16],
    s_volume_name           : [char;16],
    s_encrypt_algos         : u8,
    s_checksum              : u32,
    s_desc_size             : u16,
    s_reserved_gdt_blocks   : u16,
    s_log_groups_per_flex   : u8,
    is_64bit                : bool
}

#[derive(Debug,Default)]
pub struct GroupDescriptor {
    bg_block_bitmap_lo      : u32,
    bg_block_bitmap_hi      : u32,
    bg_inode_bitmap_lo      : u32,
    bg_inode_bitmap_hi      : u32,
    bg_inode_table_lo       : u32,
    bg_inode_table_hi       : u32,
    bg_free_blocks_count_lo : u16,
    bg_free_blocks_count_hi : u16,
    bg_free_inodes_count_lo : u16,
    bg_free_inodes_count_hi : u16,
    bg_used_dirs_count_lo   : u16,
    bg_used_dirs_count_hi   : u16,

    ext4_to_self            : Option<*const Ext4>,
    is_64bit                : bool
}

pub struct ReservedGDTBlock {

}

pub struct Block {

}

pub struct Bitmap {
    bitmap                  : Bytes
}

#[derive(Debug,PartialEq)]
pub struct ExtentHeader {
    eh_magic        : u16,
    eh_entries      : u16,
    eh_max          : u16,
    eh_depth        : u16,
    eh_generation   : u32
}

#[derive(Debug,PartialEq)]
pub struct ExtentIdx {
    ei_block        : u32,
    ei_leaf_lo      : u32,
    ei_leaf_hi      : u16,
    ei_unused       : u16
}

#[derive(Debug,PartialEq)]
pub struct Extent {
    ee_block        : u32,
    ee_len          : u16,
    ee_start_hi     : u16,
    ee_start_lo     : u32
}

#[derive(Debug,PartialEq)]
pub enum ExtentNodeType {
    ExtentType,
    IdxType
}

#[derive(Debug,PartialEq)]
pub struct ExtentNode {
    header          : ExtentHeader,
    idx_items       : Vec<ExtentIdx>,
    extents         : Vec<Extent>,
    node_type       : ExtentNodeType,
    base_addr       : u64
}


#[derive(Debug,Clone,PartialEq)]
pub enum FileType {
    Unknown=0x0,
    RegularFile=0x1,
    Directory=0x2,
    CharacterDeviceFile=0x3,
    BlockDeviceFile=0x4,
    FIFO=0x5,
    Socket=0x6,
    SymbolicLink=0x7
}



#[derive(Debug,Clone,PartialEq)]
pub struct DirectoryEntry {
    inode           : u32,
    rec_len         : u16,
    name_len        : u8,
    file_type       : FileType,
    name            : Vec<u8>,
    utf8_name       : String,
    with_zero_end_string    : String
}

type ExtentTree = ExtentNode;


pub enum FileMode {
    S_IXOTH=0x1,
    S_IWOTH=0x2,
    S_IROTH=0x3,
    S_IFDIR=0x4000
}

#[derive(Debug, Clone)]
pub struct Inode {
    i_mode              : u16,
    i_uid               : u16,
    i_size_lo           : u32,
    i_atime             : u32,
    i_ctime             : u32,
    i_mtime             : u32,
    i_dtime             : u32,
    i_gid               : u16,
    i_links_count       : u16,
    i_blocks_lo         : u32,
    i_flags             : u32,
    i_generation        : u32,
    i_file_acl_lo       : u32,
    i_size_high         : u32,
    i_obso_faddr        : u32,
    i_block             : Vec<u8>,
    i_extra_isize       : u16,
    i_checksum_hi       : u16,
    i_ctime_extra       : u32,
    i_mtime_extra       : u32,
    i_atime_extra       : u32,
    i_crtime            : u32,
    i_crtime_extra      : u32,
    i_version_hi        : u32,
    i_projid            : u32,

    ext4                : Option<*const Ext4>,
    base_addr           : u64
}

pub struct InodeTableIter {

}

pub enum HASH_VERSION {
    Legacy,
    HalfMD4,
    Tea,
    LeacyUnsigned,
    HalfMD4Unsigned,
    TeaUnsigned
}

#[derive(Debug)]
pub struct JournalHeader {
    h_magic         : u32,
    h_blocktype     : u32,
    h_sequence      : u32,
}

#[derive(Debug)]
pub struct JournalSuperBlock {
    header          : JournalHeader,
    s_blocksize     : u32,
    s_maxlen        : u32,
    s_first         : u32,
    s_errno         : u32,
    s_max_transaction   : u32,
    s_max_trans_data    : u32,
    s_feature_compat    : u32,
    s_feature_incompat  : u32,
    s_feature_ro_compat	: u32,

}

#[derive(Debug)]
pub struct JournalBlockTag {
    blocknr         : u32,
    flag            : u32,
    blocknr_high    : u32,
    size            : usize,
    uuid            : Vec<u8>,
}

#[derive(Debug)]
pub struct RevocationBlock {
    header      : JournalHeader,
    r_count     : u32,
    blocks      : u64
}

#[derive(Debug)]
pub struct CommitBlock {
    header              : JournalHeader,
    commit_sec          : u64,
    commit_nsec         : u32,
}

#[derive(Debug)]
pub struct JournalDescriptorBlock {
    header              : JournalHeader,
    open_coded_array    : Vec<JournalBlockTag>
}

#[derive(Debug, Clone)]
pub struct JournalDataBlock {
    range       : Range<usize>,
    block_id    : u64
}



#[derive(Debug)]
pub struct JournalTransaction {
    desc_block          : JournalDescriptorBlock,
    data_blocks         : Vec<JournalDataBlock>,
    revocation_blocks   : Vec<RevocationBlock>,
    commit_block        : CommitBlock
}

#[derive(Debug)]
pub struct JournalTransactionIteration {
    transaction         : JournalTransaction
}

pub struct Journal {
    super_block     : JournalSuperBlock,
    ext4            : Option<*const Ext4>,
    offset          : usize
}