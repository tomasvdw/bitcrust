use std::io;
use std::net::Ipv6Addr;
use byteorder::{LittleEndian, BigEndian, WriteBytesExt};

#[cfg(test)]
mod tests {
    use super::Encode;

    #[test]
    fn it_encodes_a_u8() {
        let mut actual = vec![];
        let _ = 23u8.encode(&mut actual);
        assert_eq!(vec![23], actual);
    }

    #[test]
    fn it_encodes_a_u16() {
        let mut actual = vec![];
        let _ = 23u16.encode(&mut actual);
        assert_eq!(vec![23, 0], actual);
    }

    #[test]
    fn it_encodes_a_u32() {
        let mut actual = vec![];
        let _ = 23u32.encode(&mut actual);
        assert_eq!(vec![23, 0, 0, 0], actual);
    }

    #[test]
    fn it_encodes_a_u64() {
        let mut actual = vec![];
        let _ = 23u64.encode(&mut actual);
        assert_eq!(vec![23, 0, 0, 0, 0, 0, 0, 0], actual);
    }

    #[test]
    fn it_encodes_a_i8() {
        let mut actual = vec![];
        let _ = 23i8.encode(&mut actual);
        assert_eq!(vec![23], actual);
    }

    #[test]
    fn it_encodes_a_i16() {
        let mut actual = vec![];
        let _ = 23i16.encode(&mut actual);
        assert_eq!(vec![23, 0], actual);
    }

    #[test]
    fn it_encodes_a_i32() {
        let mut actual = vec![];
        let _ = 23i32.encode(&mut actual);
        assert_eq!(vec![23, 0, 0, 0], actual);
    }

    #[test]
    fn it_encodes_a_i64() {
        let mut actual = vec![];
        let _ = 23i64.encode(&mut actual);
        assert_eq!(vec![23, 0, 0, 0, 0, 0, 0, 0], actual);
    }

    #[test]
    fn it_encodes_a_true() {
        let mut actual = vec![];
        let _ = true.encode(&mut actual);
        assert_eq!(vec![1], actual);
    }

    #[test]
    fn it_encodes_a_false() {
        let mut actual = vec![];
        let _ = false.encode(&mut actual);
        assert_eq!(vec![0], actual);
    }
}


pub trait Encode {
    fn encode(&self, &mut Vec<u8>) -> Result<(), io::Error>;
}

impl Encode for u8 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_u8(*self)
    }
}

impl Encode for u16 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_u16::<LittleEndian>(*self)
    }
}

impl Encode for u32 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_u32::<LittleEndian>(*self)
    }
}

impl Encode for u64 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_u64::<LittleEndian>(*self)
    }
}

impl Encode for i8 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_i8(*self)
    }
}

impl Encode for i16 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_i16::<LittleEndian>(*self)
    }
}

impl Encode for i32 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_i32::<LittleEndian>(*self)
    }
}

impl Encode for i64 {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        buff.write_i64::<LittleEndian>(*self)
    }
}

impl Encode for bool {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        match *self {
            true => buff.push(1),
            false => buff.push(0),
        };
        Ok(())
    }
}

impl Encode for Ipv6Addr {
// write IP
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        for octet in self.segments().iter() {
            buff.write_u16::<BigEndian>(*octet)?;
        }
        Ok(())
    }
}

impl Encode for [u8] {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        for byte in self {
            buff.write_u8(*byte)?;
        }
        // buff.extend(self);
        Ok(())
    }
}

impl Encode for [u8; 8] {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        for byte in self {
            buff.write_u8(*byte)?;
        }
        // buff.extend(self);
        Ok(())
    }
}

impl Encode for [u8; 32] {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        for byte in self {
            buff.write_u8(*byte)?;
        }
        // buff.extend(self);
        Ok(())
    }
}

impl Encode for String {
    fn encode(&self, buff: &mut Vec<u8>) -> Result<(), io::Error> {
        for byte in self.as_bytes() {
            buff.write_u8(*byte)?;
        }
        Ok(())
    }
}

impl<T> Encode for Vec<T> 
    where T: Encode {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        for item in self {
            item.encode(&mut buff)?;
        }
        Ok(())
    }
}

impl<T> Encode for Option<T> 
    where T: Encode {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        if let &Some(ref item) = self {
            item.encode(&mut buff)?;
        }
        Ok(())
    }
}