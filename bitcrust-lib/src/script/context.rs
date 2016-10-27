
use super::stack;
use super::ScriptError;
use std::fmt;
use std::io;
use std::io::Write;

use script::opcode::OPCODES;

/// Context provides an execution environment for scripts
///
/// It is passed through 
pub struct Context<'a> {
    pub stack:     stack::Stack,
    pub alt_stack: stack::Stack,
    
    pub script1:   &'a[u8],
    pub ip:        usize   
}


impl<'a> Context<'a> {

    pub fn new(script:  &'a[u8]) -> Context<'a>
    {
        Context {
            stack:     stack::Stack::new(),
            alt_stack: stack::Stack::new(),
            script1:   script,
            ip:        0
        }
    }
    
    pub fn run(&mut self) -> Result<(), ScriptError> {
        unimplemented!();
    }


    /// Returns the bytes of the script pointed to by the current
    /// ip (instruction pointer), and increases the ip to the last
    /// byte returned
    ///
    /// Can return a UnexpectedEndOfScript if not enough bytes are available  
    pub fn next_bytes(&mut self, count: u64) -> Result<&[u8], ScriptError> { 
        if self.script1.len() < self.ip + count  as usize + 1 {
            return Err(ScriptError::UnexpectedEndOfScript);
        }

        let old_ip = self.ip;
        self.ip += count as usize;
        Ok(&self.script1[old_ip + 1 .. self.ip + 1])
        
    }

    pub fn next_uint(&mut self, count: u64) -> Result<u64, ScriptError> { 
        let bytes = try!(self.next_bytes(count));

        // parse as little endian
        Ok(bytes.iter().enumerate().fold(0, 
            |sum, (n, byte)| sum + ((*byte as u64) << (n * 8))
        ))
    }

}


impl<'a> fmt::Debug for Context<'a> {
    
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        
        // create mutable copy
        let mut copied_context = Context::new(self.script1);

        // target to write the script to
        let buf: Vec<u8> = Vec::new();
        let mut cursor = io::Cursor::new(buf);
        

        while copied_context.ip < copied_context.script1.len() {
            let opcode = copied_context.script1[copied_context.ip] as usize;

            // OpCodes display function
            (OPCODES[opcode].display)(&mut copied_context, &mut cursor).unwrap();

            write!(&mut cursor, " ").unwrap();
            copied_context.ip += 1;
        }

        // write to output
        // we know we're not writing invalid utf so we can unwrap
        write!(fmt, "{}", &String::from_utf8(cursor.into_inner()).unwrap())    
        
    }
}



mod tests {
    #![cfg(test)]
    use ::script::context::Context;

    
    #[test]
    fn test_op_false()
    {
        let script = vec![0x00];
        let ctx = Context::new(&script);
        
        
        
        
    }
}