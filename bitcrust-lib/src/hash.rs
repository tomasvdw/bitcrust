

extern crate serde_json;
use decode;
use std::fmt::{Debug,Formatter,Error};


#[derive(PartialEq)]
pub struct Hash32<'a>(pub &'a[u8]);

impl<'a> decode::Parse<'a> for Hash32<'a> {
    fn parse(buffer: &mut decode::Buffer<'a>) -> Result<Hash32<'a>, decode::EndOfBufferError> {
        Ok(
            Hash32(try!(buffer.parse_bytes(32)))
        )
    }
}


impl<'a> Debug for Hash32<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let x = self.0
            .iter()
            .rev()
            .map(|n| format!("{:02x}", n))
            .collect::<Vec<_>>()
            .concat();
            
        fmt.write_str(&x)
    }
}

#[cfg(test)]
mod test {
    
    use super::*;
    
    #[test]
    fn test_format_hash32() {
        
        println!("{:?}", Hash32(&[0;32]));
    }
}

