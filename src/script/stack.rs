///
/// Currently unused
///
///
use super::ScriptError;

pub struct Stack(Vec<Box<[u8]>>);
        

impl Stack {
    
    pub fn new() -> Stack {
        Stack(Vec::new())
    }
    
    /// Pops the top byte-vector from the stack
    ///
    /// Returns a StackUnderflow if no items are available
    pub fn pop(&mut self) -> Result<Box<[u8]>, ScriptError> {
        self.0.pop()
            .ok_or(ScriptError::StackUnderflow)         
    }
    
    /// Pushes the given byte-array on the stack
    ///
    /// Does not return an error; stack-overflows should be handled by the caller
    pub fn push(&mut self, data: Box<[u8]>) -> Result<(), ScriptError> {
        self.0.push(data);
        Ok(())
    }
    
    /// Pops a value of the stack, interprets its as a scriptnum
    /// and returns it as a i64
    ///
    /// The stack contains byte-arrays.
    /// Scriptnums are stored as little-endian, sign-and-magnitude
    /// which means that the highest bit of the last byte of the stack-item
    /// determines the sign.
    /// Numbers larger then 4 bytes can be pushed but not popped
    /// This gives a range of -0x7fffffff to +0x7fffffff
    ///
    /// Can return a stack-underflow if no items are on the stack
    ///
    pub fn pop_scriptnum(&mut self) -> Result<i64, ScriptError> {
        let bytes = self.pop()?;
        
        if bytes.len() == 0 {
            return Ok(0);
        }
        if bytes.len() > 4 {
            return Err(ScriptError::NumericOverflow);
        }
        
        // grap each bytes with default 0 
        let b1 = *bytes.get(0).unwrap_or(&0) as i64;
        let b2 = *bytes.get(1).unwrap_or(&0) as i64;
        let b3 = *bytes.get(2).unwrap_or(&0) as i64;
        let b4 = *bytes.get(3).unwrap_or(&0) as i64;

        let signed_magnitude  = b1 | (b2 << 8) | (b3 << 16) | (b4 << 24);
        
        let sign_bit_mask = 0x80_i64 << ((bytes.len()-1)*8);
        
        Ok(
            // if the signed-magnitude is negative,  
            // convert to "normal" two-complement
            if (signed_magnitude & sign_bit_mask) > 0 { 
                -( signed_magnitude ^ sign_bit_mask)
            }
            else {
                signed_magnitude
            }    
        )   
    }
    
    /// Stores a scriptnum on the stack
    /// in signed-magnitude format (see pop_scriptnum)
    /// 
    /// If the value is more then 4-bytes,
    /// it will not be possible to read it back as a scriptnum 
    pub fn push_scriptnum(&mut self, n: i64) -> Result<(), ScriptError> {
        
        if n == 0 {
            self.0.push(Box::new([]));
            return Ok(());
        }

        // two-complement -> sign & magnitude
        let sign_byte: u8 = ((n >> 56) & 0x80) as u8;
        let mut magnitude = n.abs();

        // push all but last
        let mut result : Vec<u8> = Vec::with_capacity(8);
        while magnitude > 0x7F
        {
            result.push((magnitude & 0xFF) as u8);
            magnitude >>= 8;
        }

        // push last with sign-byte
        result.push(((magnitude & 0xFF) as u8) | sign_byte);
        
        self.0.push(result.into_boxed_slice());
        Ok(())
    }
    
}

#[cfg(test)]
mod test {
    
    use super::*;
    use script::ScriptError;
    
    #[test]
    fn test_push() {
        let mut stack = Stack::new();
        stack.push(Vec::new().into_boxed_slice()).unwrap();
        
        assert_eq!(1, stack.0.len());
        assert!(stack.0[0].eq(&Vec::new().into_boxed_slice()));
    }

    #[test]
    fn test_push_scriptnum() {

        
        let mut stack = Stack::new();
        stack.push_scriptnum(0).unwrap();
        assert_eq!(0, stack.0[0].len());

        stack.push_scriptnum(1).unwrap();
        assert_eq!(1, stack.0.last().unwrap().len());
        assert_eq!(0x01_u8, stack.0.last().unwrap()[0]);

        stack.push_scriptnum(-1).unwrap();
        assert_eq!(1, stack.0.last().unwrap().len());
        assert_eq!(0x81_u8, stack.0.last().unwrap()[0]);

        assert!(stack.0.last().unwrap().eq(&vec![0x81u8].into_boxed_slice()));
        
    
    }
    
    #[test]
    fn test_pop_scriptnum() {
        let mut stack = Stack::new();
        stack.push(vec![].into_boxed_slice()).unwrap();
        assert_eq!(0,stack.pop_scriptnum().unwrap());

        stack.push(vec![0x81].into_boxed_slice()).unwrap();
        assert_eq!(-1,stack.pop_scriptnum().unwrap());
        
        stack.push(vec![0x03,0x81].into_boxed_slice()).unwrap();
        assert_eq!(-259,stack.pop_scriptnum().unwrap());

        stack.push(vec![0x03,0x81,0x04,0x01,0xde].into_boxed_slice()).unwrap();
        assert_eq!(Err(ScriptError::NumericOverflow), stack.pop_scriptnum());
    }

    #[test]
    fn test_push_pop_scriptnum() {


        fn push_pop_scriptnum(stack: &mut Stack, n: i64) {
            
            stack.push_scriptnum(n).unwrap();
            let m = stack.pop_scriptnum().unwrap();
            assert_eq!(n,m);
        }
        
        let mut stack = Stack::new();
        for n in -1000i64..1000 {
            push_pop_scriptnum(&mut stack, n);
        
        }

        // edges
        push_pop_scriptnum(&mut stack, 0x7FFFFFFF_i64);
        push_pop_scriptnum(&mut stack, -0x7fffffff_i64);
        
        // push allowed pop not
        stack.push_scriptnum(0x80000000_i64).unwrap();
        assert_eq!(stack.pop_scriptnum().unwrap_err(), ScriptError::NumericOverflow);

        // push allowed pop not
        stack.push_scriptnum(-0x80000000_i64).unwrap();
        assert_eq!(stack.pop_scriptnum().unwrap_err(), ScriptError::NumericOverflow);
        
    }
    
}
