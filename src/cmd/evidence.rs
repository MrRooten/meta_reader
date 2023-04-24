pub enum EvidenceType {
    Ntfs,
    Ext4,
    Elf64
}

pub struct Evidence {
    path        : String,
    e_type      : String
}

