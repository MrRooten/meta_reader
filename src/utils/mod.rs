use std::error::Error;

pub mod error;
pub mod funcs;
pub mod file;
#[derive(Debug,Default)]
pub struct MRError {
    detail  : String,
    err     : Option<Box<dyn Error>>
}
