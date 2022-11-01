use std::{error::Error, fmt};

use super::MRError;


impl MRError {
    pub fn new(msg: &str) -> MRError{
        MRError {
            detail: msg.to_string(),
            ..Default::default()
        }
    }

    pub fn from(err: Box<dyn Error>) -> MRError{
        let mut result = MRError {
            detail: "".to_string(),
            ..Default::default()
        };
        result.err = Some(err);
        result
    }
}

impl fmt::Display for MRError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.detail)
    }
}

impl Error for MRError {
    fn description(&self) -> &str {
        &self.detail
    }


}