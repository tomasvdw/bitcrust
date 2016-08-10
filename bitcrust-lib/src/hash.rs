

extern crate serde_json;

use std::fmt::{Debug,Formatter,Error};
use serde::{Serializer,Deserializer};

#[derive(Serialize, Deserialize)]
pub struct Hash256(
    [u8; 32]);





impl Debug for Hash256 {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let x = self.0
            .iter()
            .map(|n| format!("{:02x}", n))
            .collect::<Vec<_>>()
            .concat();
        try!(fmt.write_str(&x));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    
    use super::*;
    
    #[test]
    fn test_format_hash256() {
        
        println!("{:?}", Hash256([0;32]));
    }
}

