use crate::utils::MRError;

pub struct Workplace {
    name    : String 

}

pub struct Environment {
    workplace   : Workplace
}

impl Environment {
    pub fn new() -> Environment {
        unimplemented!()
    }
}

pub struct Argument {
    name    : String,
    value   : String
}

pub struct CMDHandler {
    env     : Environment
}

impl CMDHandler {
    pub fn get_handler() -> CMDHandler {
        CMDHandler {  
            env : Environment::new()
        }
    }

    pub fn process(&self, line: &str) -> Result<(), MRError> {
        unimplemented!()
    }


}