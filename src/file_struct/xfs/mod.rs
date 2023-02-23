pub struct SuperBlock {
    block_size          : u32,
    block_number        : u64,
    device_block_number : u64,
    device_extent_number    : u64,
    file_system_id          : [u8;16],
    journal_block_number    : u64,
    root_dir_inode          : u64,
    rt_bitmap_extent_inode_number   : u64,
    rt_bitmap_summary_inode_number  : u64,
    rt_extent_size                  : u32,
    alloc_group_size                : u32,
    alloc_groups_number             : u32,
    rt_bitmap_size                  : u32,
    journal_size                    : u32,
    feature_flags                   : u16,
    sector_size                     : u16,
    inode_size                      : u16,
    inodes_per_block                : u16,
    //Only used in the first superblock
    number_of_inodes                : Option<u64>,
    number_of_free_inodes           : Option<u64>,
    number_of_free_data_blocks      : Option<u64>,
    number_of_rt_extents            : Option<u64>,
    //Only used if the XFS_SB_VERSION_QUOTABIT feature flag is set
    user_quota_inode_number         : Option<u64>,
    group_quota_inode_number        : Option<u64>,
    quota_flags                     : Option<u16>,
    //Only used if the XFS_SB_VERSION_ALIGNBIT feature flag is set
    inode_chunk_alignment_size      : Option<u32>,
}

pub struct XFS {
    
}