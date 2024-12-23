use std::{error::Error, fmt::Display};

pub mod error;
pub mod funcs;
pub mod file;
pub mod log;


#[derive(Debug)]
pub enum MRErrKind {
    None,
    OutOfByteRange
}

impl Default for MRErrKind {
    fn default() -> Self {
        Self::None
    }
}

impl Display for MRErrKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => {
                write!(f, "None")
            },
            MRErrKind::OutOfByteRange => todo!(),
        }
        
    }
}


#[derive(Debug, Default)]
pub struct MRError {
    detail  : Option<String>,
    err     : Option<Box<dyn Error>>,
    kind    : MRErrKind
}
