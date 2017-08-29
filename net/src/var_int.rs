use std::io;

use Encode;

#[cfg(test)]
mod tests {
    use super::{Encode, VarInt};

    #[test]
    fn it_encodes_a_u8_varint() {
        let input = VarInt::new(12);
        let mut var_int = vec![];
        let _ = input.encode(&mut var_int);
        assert_eq!(var_int, &[12 as u8]);
    }

    #[test]
    fn it_encodes_a_u16_varint() {
        let input = VarInt::new(0xFFFF);
        let mut var_int = vec![];
        let _ = input.encode(&mut var_int);
        assert_eq!(var_int, &[0xFD, 0xFF, 0xFF]);
    }

    #[test]
    fn it_encodes_a_u32_varint() {
        let input = VarInt::new(0xFFFFFFFF);
        let mut var_int = vec![];
        let _ = input.encode(&mut var_int);
        assert_eq!(var_int, &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn it_encodes_a_u64_varint() {
        let input = VarInt::new(0xFFFFFFFF + 1);
        let mut var_int = vec![];
        let _ = input.encode(&mut var_int);
        assert_eq!(var_int,
                   &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]);
    }
}

#[derive(Debug, PartialEq)]
pub struct VarInt {
    value: u64
}

impl VarInt {
    pub fn new(i: u64) -> VarInt {
        VarInt {
            value: i
        }
    }
}

impl Encode for VarInt {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        if self.value < 0xFD {
            (self.value as u8).encode(&mut buff)?;
            return Ok(());
        }
        if self.value <= 0xFFFF {
            0xFDu8.encode(&mut buff)?;
            (self.value as u16).encode(&mut buff)?;
            return Ok(());
        }
        if self.value <= 0xFFFFFFFF {
            0xFEu8.encode(&mut buff)?;
            (self.value as u32).encode(&mut buff)?;
            return Ok(());
        }
        0xFFu8.encode(&mut buff)?;
        self.value.encode(&mut buff)?;
        Ok(())
    }
}