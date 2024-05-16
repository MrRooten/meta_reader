use std::{error::Error, fmt};

use super::{MRError, MRErrKind};


impl MRError {
    pub fn new(msg: &str) -> MRError{
        MRError {
            detail: Some(msg.to_string()),
            ..Default::default()
        }
    }

    pub fn new_with_kind(msg: &str, kind: MRErrKind) -> MRError {
        MRError {
            detail: Some(msg.to_string()),
            kind,
            ..Default::default()
        }
    }

    pub fn from(err: Box<dyn Error>) -> MRError{
        let mut result = MRError {
            detail: Some("".to_string()),
            ..Default::default()
        };
        result.err = Some(err);
        result
    }
}


impl fmt::Display for MRError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(detail) = &self.detail {
            return write!(f, "{}: {}", self.kind, detail)
        }
        
        if let Some(err) = &self.err {
            return write!(f, "{}: {:?}", self.kind, err)
        }

        write!(f, "{}: Nothing", self.kind)
    }
}

impl Error for MRError {
    fn description(&self) -> &str {
        match &self.detail {
            Some(s) => s,
            None => {
                ""
            }
        }
    }


}
