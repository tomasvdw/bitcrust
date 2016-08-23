
use super::stack;
use super::ScriptError;

        
/// Context provides an execution environment for scripts
///
/// It is passed through 
pub struct Context {
    pub stack:     stack::Stack,
    pub alt_stack: stack::Stack,
    
    pub script1:   Vec<u8>,
    pub ip:        usize   
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




mod tests {
    #![cfg(test)]
    use ::script::context::Context;

    
    #[test]
    fn test_op_false()
    {
        let script = vec![0x00];
        let ctx = Context::create(script);
        
        
    }
}