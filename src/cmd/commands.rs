use std::{process, fs};

use crate::utils::MRError;

use super::handler::Argument;

pub trait Command {
    fn short_name(&self) -> &str;

    fn full_name(&self) -> &str;

    fn run(&self, args: &Vec<Argument>) -> Result<(), MRError>;

    fn help(&self) -> String;

    fn info(&self) -> String;
}

pub type Commands = Vec<Box<dyn Command>>;
pub type BoxCommand = Box<dyn Command>;
pub struct CmdMgr {
    cmds    : Commands
}

impl CmdMgr {
    pub fn new() -> CmdMgr {
        let mut result = Vec::<Box<dyn Command>>::new();
        result.push(Box::new(Exit{}));
        CmdMgr { cmds: result }
    }

    pub fn get_procs(&self) -> &Commands{
        &self.cmds
    }

    pub fn get_proc(&self, name: &str) -> Option<&BoxCommand> {
        for cmd in &self.cmds {
            if cmd.full_name().eq(name) || cmd.short_name().eq(name) {
                return Some(cmd);
            }
        }

        return None;
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

    fn run(&self, args: &Vec<Argument>) -> Result<(), MRError> {
        process::exit(0);
    }

    fn help(&self) -> String {
        "exit the process, Usage: exit".to_string()
    }

    fn info(&self) -> String {
        todo!()
    }
}

pub struct Help {

}

impl Command for Help {
    fn short_name(&self) -> &str {
        "h"
    }

    fn full_name(&self) -> &str {
        "help"
    }

    fn run(&self, args: &Vec<Argument>) -> Result<(), MRError> {
        todo!()
    }

    fn help(&self) -> String {
        "".to_string()
    }

    fn info(&self) -> String {
        todo!()
    }
}

pub struct CreateWork {

}

impl Command for CreateWork {
    fn short_name(&self) -> &str {
        "cw"
    }

    fn full_name(&self) -> &str {
        "create_work"
    }

    fn run(&self, args: &Vec<Argument>) -> Result<(), MRError> {
        if args.len() < 1 {
            return Err(MRError::new("Must set the workplace name"));
        }
        let name = args[0].get_name();
        if std::path::Path::new(name).exists() {
            let output = format!("'{}' already existed", name);
            return Err(MRError::new(&output));
        }
        let path = format!("{}",name);
        match fs::create_dir(path) {
            Ok(_) => {

            }, 
            Err(e) => {
                return Err(MRError::from(Box::new(e)));
            }
        };
        return Ok(())
    }

    fn help(&self) -> String {
        return "create_work ${name}".to_string()
    }

    fn info(&self) -> String {
        todo!()
    }
}



