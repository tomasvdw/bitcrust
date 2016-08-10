
use super::ScriptError;

pub struct Stack(Vec<Vec<u8>>);
        

impl Stack {
    
    pub fn new() -> Stack {
        Stack(Vec::new())
    }
    
    pub fn pop(&mut self) -> Result<Vec<u8>, ScriptError> {
        self.0.pop()
            .ok_or(ScriptError::Pop_Empty_Stack)
         
    }
    
    pub fn push(&mut self, data: Vec<u8>) -> Result<(), ScriptError> {
        self.0.push(data);
        Ok(())
    }
    
    pub fn pop_scriptnum(&mut self) -> Result<i64, ScriptError> {
        let bytes = try!(self.pop());
        
        if bytes.len() > 4 {
            return Err(ScriptError::Numeric_Overflow);
        }
            
        // parse bytes as little-endian
        let signed_mag: i64 = bytes.iter().enumerate().fold(0, 
            |sum, (n, byte)| sum + ((*byte as i64) << (n * 8))
        );
            
        // signed-magnitude -> two complement
        if signed_mag < 0 {
            Ok(- signed_mag - 1) 
        }
        else {
            Ok(signed_mag)
        }                
    }
    
    pub fn push_scriptnum(&mut self, n: i64) -> Result<(), ScriptError> {
        
        //two-complement -> signed-magnitude
        let mut signed_mag = if n < 0 { - n - 1 } else { n };
   
        // create byte-array
        let mut result : Vec<u8> = Vec::with_capacity(8);
        while signed_mag != 0 && signed_mag != -1
        {
            result.push((signed_mag & 0xFF) as u8);
            signed_mag >>= 8;
        }
        self.0.push(result);
        Ok(())
    }
    
}

#[cfg(test)]
mod test {
    
    use super::*;
    
    #[test]
    fn test_push() {
        let mut stack = Stack::new();
        stack.push(Vec::new());
        
        assert_eq!(1, stack.0.len());
        assert_eq!(0, stack.0[0].len());
    }
}
