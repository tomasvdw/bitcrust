
use super::stack;
use super::ScriptError;

        

pub struct Context {
    pub stack: stack::Stack,
    pub alt_stack: stack::Stack,
    
    pub script1: Vec<u8>,
    pub ip: usize    
}

impl Context {
    pub fn create(script: Vec<u8>) -> Context
    {
        
        Context {
            stack:     stack::Stack::new(),
            alt_stack: stack::Stack::new(),
            script1:   script,
            ip:        0
        }
    }
    
    pub fn run(&mut self) -> Result<(), ScriptError> {
        Ok(())
    }
}


