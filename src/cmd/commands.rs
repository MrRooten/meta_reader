use crate::utils::MRError;

use super::handler::Argument;

pub trait Command {
    fn short_name(&self) -> &str;

    fn full_name(&self) -> &str;

    fn run(&self) -> Result<(), MRError>;

    fn help(&self, args: &Vec<Argument>) -> String;

    fn info(&self) -> String;
}

pub type Commands = Vec<Box<dyn Command>>;
pub struct CmdMgr {
    cmds    : Commands
}

impl CmdMgr {
    pub fn new() -> CmdMgr {
        let mut result = Vec::<Box<dyn Command>>::new();
        CmdMgr { cmds: result }
    }

    pub fn get_procs(&self) -> &Commands{
        &self.cmds
    }

    
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



