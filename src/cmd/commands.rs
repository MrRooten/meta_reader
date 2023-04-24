use crate::utils::MRError;

use super::handler::Argument;

pub trait Command {
    fn short_name(&self) -> &str;

    fn full_name(&self) -> &str;

    fn run(&self) -> Result<(), MRError>;

    fn help(&self, args: &Vec<Argument>) -> String;

    fn info(&self) -> String;
}

pub struct Exit {

}

impl Command for Exit {
    fn short_name(&self) -> &str {
        "exit"
    }

    fn full_name(&self) -> &str {
        "exit"
    }

    fn run(&self) -> Result<(), MRError> {
        todo!()
    }

    fn help(&self, args: &Vec<Argument>) -> String {
        todo!()
    }

    fn info(&self) -> String {
        todo!()
    }
}

pub struct Help {

}
pub struct ChangeWorkspace {

}



