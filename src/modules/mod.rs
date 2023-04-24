use std::collections::HashMap;

use crate::utils::MRError;

pub mod ext4;
pub mod ntfs;
pub trait Hanlder {
    fn run(&self, args: HashMap<String, String>) -> Result<(), MRError>;

    fn name(&self) -> &str;

    fn help(&self) -> &str;
}