use crate::utils::MRError;

use super::commands::CmdMgr;

pub struct Workplace {
    _name    : String 

}

impl Workplace {
    pub fn new(name: &str) -> Workplace {
        Workplace { _name: name.to_string() }
    }
}

pub struct Environment {
    _workplace   : Option<Workplace>
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Environment {
        Environment { _workplace: None }
    }
}

pub struct Argument {
    name    : String,
    value   : String
}

impl Argument {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_value(&self) -> &String {
        &self.value
    }
}

pub struct CMDHandler {
    env     : Environment,
    cmds    : CmdMgr
}

fn process_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut state = false;
    let mut open_char = '-';
    let mut s = String::new();
    let mut is_escape = false;
    for c in line.chars() {
        if c.is_whitespace() && !state {
            if !s.is_empty() {
                result.push(s.clone());
                s = String::new();
            }
            continue;
        }
        if is_escape {
            s.push(c);
            is_escape = false;
            continue;
        }

        if c.eq(&'\\') {
            is_escape = true;
            continue;
        }

        if c.eq(&'\'') && !state {
            open_char = '\'';
            state = true;
            continue;
        }

        if c.eq(&'"') && !state {
            open_char = '"';
            state = true;
            continue;
        }

        if c.eq(&'\'') && open_char.eq(&'\'') && state {
            if !s.is_empty() {
                result.push(s.clone());
                s = String::new();
            }
            state = false;
            open_char = '-';
            continue;
        }

        if c.eq(&'"') && open_char.eq(&'"') && state {
            if !s.is_empty() {
                result.push(s.clone());
                s = String::new();
            }
            state = false;
            open_char = '-';
            continue;
        }

        s.push(c);
    }

    if !s.is_empty() {
        result.push(s);
    }
    result
}

pub fn process_arg(arg: &str) -> Vec<String> {
    let mut result = Vec::new();
    let index = match arg.find('=') {
        Some(s) => s,
        None => {
            result.push(arg.to_string());
            return result;
        }
    };
    let key = arg[0..index].to_string();
    let value = arg[index+1..].to_string();
    result.push(key);
    result.push(value);
    result
}

impl CMDHandler {
    pub fn get_handler() -> CMDHandler {
        CMDHandler {  
            env : Environment::new(),
            cmds : CmdMgr::new()
        }
    }

    pub fn get_env(&self) -> &Environment {
        &self.env
    }

    pub fn process(&self, line: &str) -> Result<(), MRError> {
        let argv = process_line(line);
        let command = &argv[0];
        let command = match self.cmds.get_proc(command) {
            Some(s) => s,
            None => {
                return Err(MRError::new("Not found command"));
            }
        };

        let argv = argv[1..].to_vec();
        let mut args: Vec<Argument> = Vec::new();
        for arg in argv {
            let kv = process_arg(&arg);
            let a = if kv.len() != 2 {
                Argument {
                    name: kv[0].to_string(),
                    value: "".to_string(),
                }
            } else {
                Argument {
                    name: kv[0].to_string(),
                    value: kv[1].to_string(),
                }
            };
            args.push(a);
        }

        command.run(&args)
    }


}