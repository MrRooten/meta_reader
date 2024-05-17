pub enum VMDKFlag {

}

pub enum CompressMethod {

}

pub struct VMDKHeader {
    signature       : [u8;4],
    version         : u32,
    flags           : VMDKFlag,
    maximum_data_of_sectors     : u64,
    number_of_sectors           : u64,
    descriptor_sector_number    : u64,
    descriptor_number_of_sector : u64,
    number_of_grain_table       : u64,
    secondary_grain_sector_number   : u64,
    grain_director_sector_number    : u64,
    metadat_number_of_sector        : u64,
    is_dirty        : u8,
    single_end_of_line  : char, // '\n'
    non_end_of_line     : char, // ''
    double_end_of_line  : char, // '\r'
    second_double_end_of_line   : char, // '\n'
    compression_method  : CompressMethod,
}

pub(crate) struct GrainData {
    sector_number       : u64,
    compress_size       : u32,  //Compressed data
                                //Contains ZLIB compressed data (DEFLATE + ZLIB header)
}
