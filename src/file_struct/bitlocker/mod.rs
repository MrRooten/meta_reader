#[allow(unused)]
pub struct BDEHeaderVista {
    entry_point         : [u8;3],
    signature           : [u8;8],
    bytes_per_sector    : u16,
    sectors_per_clushter_block  : u8,
    root_directory_entries      : u16,
    number_sectors      : u16,
}