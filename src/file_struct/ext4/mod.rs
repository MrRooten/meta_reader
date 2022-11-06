use crate::utils::file::MRFile;
pub mod ext4_impl;

#[derive(Debug,Default)]
pub struct Ext4 {
    reader      : MRFile,
    super_block : Option<SuperBlock>,
}

#[derive(Debug,Default)]
pub struct SuperBlock {
    pub s_inodes_count      : u32,          //0x0
    pub s_block_count       : u32,          //0x4  
    pub s_log_block_size    : u32,          //0x18
    pub s_log_cluster_size  : u32,          //0x1c
    s_blocks_per_group      : u32,          //0x20
    s_clusters_per_group    : u32,          //0x24
    s_inodes_per_group      : u32,          //0x28
    s_creator_os            : u32,          //0x48
    s_uuid                  : [u8;16],
    s_volume_name           : [char;16],
    s_encrypt_algos         : u8,
    s_checksum              : u32
}

pub struct GroupDescriptor {

}

pub struct ReservedGDTBlock {

}

pub struct DataBlockBitmap {

}

pub struct InodeBitmap {

}

pub struct InodeTable {

}

pub struct DataBlock {

}