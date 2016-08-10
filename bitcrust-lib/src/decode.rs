use std::io;
use std::mem;
use std::error;
use std::error::Error;
use std::fmt;
use serde;



/// Decodes a slice encoded using bitcoin-protocol conventions
/// into an in memory object 
/// 
pub fn decode<T: serde::de::Deserialize>(buf: &[u8]) -> Result<T, DeserializeError> {
    
    let mut cur   = io::Cursor::new(buf);  
    let mut deser = BinDeserializer::new(&mut cur);
    let result    = try!(T::deserialize(&mut deser));
    
    // verify entire buffer has been consumed
    if deser.reader.position() != buf.len() as u64 {
        Err(DeserializeError::IncorrectLength)
    }
    else {
        Ok(result)
    }
   
}


/// Local macro to read a little-endian primitive from a reader
/// 
/// # Examples
/// ```
/// let r = read_primitive(i32, &rdr, 4);
/// ``` 
macro_rules! read_primitive(
    ($typ:path, $rdr:expr, $sz:expr) => 
    {
        {
            let mut buf = [0u8; $sz];
            try!($rdr.read_exact(&mut buf));
            let res: $typ = unsafe { mem::transmute(buf) };
            if cfg!(target_endian = "big") {
                res.swap_bytes()
            }
            else {
                res
            }
        }                
    };
);





/// Binary serde deserializer
///
/// This deserializes conforming to the bitcoin-protocol
/// Primitives are deserialized LittleEndian
pub struct BinDeserializer<'a, T: io::Read + 'a> {
    reader: &'a mut T,
}

impl<'a, T: io::Read + 'a> BinDeserializer<'a, T> {
    
    /// Constructs a new deserializer based on a reader
    fn new(reader: &'a mut T) -> BinDeserializer<'a, T> {
        BinDeserializer {
            reader: reader,
        }
    }
    
    /// Read a variable length integer from the reader
    fn read_compact_size(&mut self) -> Result<u64, DeserializeError> {
        let byte1 = read_primitive!(u8, self.reader, 1);
        Ok(match byte1 {
            0xff => read_primitive!(u64, self.reader, 8),
            0xfe => read_primitive!(u32, self.reader, 4) as u64,
            0xfd => read_primitive!(u16, self.reader, 2) as u64,
            _    => byte1 as u64
        })          
    }
}
    
#[derive(Debug)]
pub enum DeserializeError {
    IncorrectLength
}


impl fmt::Display for DeserializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(formatter)
    }
}

impl error::Error for DeserializeError {
    fn description(&self) -> &str {
        "Incorrect length"
    }
}

/// Serde wants our error to implement `serde::de::Error`,
/// but they aren't used here
impl serde::de::Error for DeserializeError { 
    fn custom<T: Into<String>>(_: T) -> Self {
        unimplemented!()
    }

    /// Raised when a `Deserialize` type unexpectedly hit the end of the stream.
    fn end_of_stream() -> Self {
        DeserializeError::IncorrectLength
    }
}
/// Conversion from IO error is always incorrect length
/// As we can assume the entire buffer has been read 
impl From<io::Error> for DeserializeError {
    fn from(_: io::Error) -> DeserializeError {
        DeserializeError::IncorrectLength        
    }
}

  

/// VecDeserializer is created when a Vec or a fixed-length array is serialized
struct SeqDeserializer<'a, 'b: 'a, T: io::Read + 'b> {
    deserializer: &'a mut BinDeserializer<'b, T>,
     
    /// len is None when the array is fixed-length (in which case Serde will
    /// handle the proper amount of invocations). For a Vec, len is read as 
    /// a compact size
    len: Option<u64>
}

impl<'a, 'b: 'a, R: io::Read + 'b> serde::de::SeqVisitor for SeqDeserializer<'a,'b, R> {
    type Error = DeserializeError;

    fn visit<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: serde::de::Deserialize,
    {
        match self.len {
            None => {
                // No length, we'll just deserialize an item
                // Serde will ensure the invocation-count matches the array-length
                let value = try!(serde::Deserialize::deserialize(self.deserializer));
                Ok(Some(value))
            },
            Some(l) =>  {
                // Length given, use countdown
                if l == 0 {
                    Ok(None)                    
                }
                else {
                    self.len = Some(l-1);
                    let value = try!(serde::Deserialize::deserialize(self.deserializer));
                    Ok(Some(value))        
                }
            }
        }
    }

    fn end(&mut self) -> Result<(), Self::Error> {
       Ok(())
    }
}



// Implementation of the deserialization methods from Serde
// See Serde, or the Bincode crate for more info
impl<'a, T: io::Read > serde::Deserializer for BinDeserializer<'a, T> {
    type Error = DeserializeError;
    
    // We don't support generic deserialization
    #[inline]
    fn deserialize<V>(&mut self, _: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor,
    {        
        unimplemented!();
    }
    
    // all needed primitives:
    
    
    #[inline]
    fn deserialize_u8<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor, 
    {
        visitor.visit_u8(read_primitive!(u8, self.reader , 1))
    }
   
    #[inline]
    fn deserialize_u16<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
            where V: serde::de::Visitor,
    {
        visitor.visit_u16(read_primitive!(u16, self.reader , 2))
    }
    
    #[inline]
    fn deserialize_u32<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
            where V: serde::de::Visitor,
    {
        visitor.visit_u32(read_primitive!(u32, self.reader , 4))
    }

    #[inline]
    fn deserialize_i32<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
            where V: serde::de::Visitor,
    {
        visitor.visit_i32(read_primitive!(i32, self.reader , 4))
    }
      
    #[inline]
    fn deserialize_i64<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
            where V: serde::de::Visitor,
    {
        visitor.visit_i64(read_primitive!(i64, self.reader , 8))
    }
        
    fn deserialize_fixed_size_array<V>(&mut self,
                                       _len: usize,
                                       mut visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor,
    {
        visitor.visit_seq(SeqDeserializer
            {deserializer: self, len: None}
        )
    
    }
   
    /// This method hints that the `Deserialize` type is expecting a sequence value. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    #[inline]
    fn deserialize_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor,
    {

        let size: u64 = try!(self.read_compact_size());
        
        visitor.visit_seq(SeqDeserializer
            {deserializer: self, len: Some(size)}
        )
    }
    
    #[inline]
    fn deserialize_struct<V>(&mut self,
                       _name: &'static str,
                       _fields: &'static [&'static str],
                       mut visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor,
    {
        visitor.visit_seq(SeqDeserializer
            {deserializer: self, len: None}
        )
    }
    
    // A tuple struct; same as struct but we mark it, such that 
    // seqs are will not be length prefixed inside 
    #[inline]
    fn deserialize_tuple_struct<V>(&mut self,
                       _name: &'static str,
                       _: usize,
                       visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor,
    {
       self.deserialize_struct(_name, &[], visitor)
    }
}


#[cfg(test)]
mod tests
{
    use super::*;
    
    
    #[test]
    fn test_deserialize_arrays()
    {
        #[derive(Deserialize)]
        struct H4([u8; 2]);
        
        #[derive(Deserialize)]
        struct Stru {
            ar2: H4,      // not length prefixed 
            ar:  Vec<u8>, // length prefixed 
            
        }
        let obj: Stru = decode(&[1u8, 2u8, 2u8, 4u8, 3u8]).unwrap();
        assert_eq!(obj.ar2.0[0] , 1);
        assert_eq!(obj.ar2.0[1] , 2);
        assert_eq!(obj.ar[0] , 4);
        assert_eq!(obj.ar[1] , 3);        
    }
    
    #[test]
    fn test_primitives() {
        assert_eq!(0xFFFEu16, decode(&[0xFEu8, 0xFFu8]).unwrap());
        assert_eq!(0x01FEu16, decode(&[0xFEu8, 0x01u8]).unwrap());
        assert_eq!(0xBEu8, decode(&[0xBEu8]).unwrap());
        
        assert_eq!(0x11BBCCDDEEFF1122i64, 
            decode(&[0x22u8, 0x11u8, 0xFFu8, 0xEEu8, 0xDDu8, 0xCCu8, 0xBBu8, 0x11u8]).unwrap());
            
        assert_eq!(-1i64, decode(&[0xFFu8; 8]).unwrap());
        
    }

    #[test]
    #[should_panic]
    fn test_too_long() {
        assert_eq!(0xFFFEu16, decode(&[0xFEu8, 0xFFu8, 0x11u8]).unwrap());        
    }
    
    #[test]
    #[should_panic]
    fn test_too_short() {
        assert_eq!(0xFFFEu32, decode(&[0xFEu8, 0xFFu8, 0x11u8]).unwrap());        
    }
    
    #[test]
    fn test_compactsize() {
        
        #[derive(Deserialize)]
        struct VecStruct {
            Data: Vec<u8>
        }
        
        // single byte compactsize
        let mut inp:Vec<u8> = vec!(5u8);
        inp.extend_from_slice(&[17u8; 5]);
        let s:VecStruct = decode(&inp[..]).unwrap();
        assert_eq!(s.Data.len(), 5);
        assert_eq!(s.Data[4], 17);
        
        // single byte compactsize
        let mut inp:Vec<u8> = vec!(0xFCu8);
        inp.extend_from_slice(&[17u8; 0xFC]);
        let s:VecStruct = decode(&inp[..]).unwrap();
        assert_eq!(s.Data.len(), 0xFC);
        assert_eq!(s.Data[4], 17);
        
        // double byte compactsize
        let mut inp:Vec<u8> = vec!(0xFD, 0xEFu8, 0xBEu8);
        inp.extend_from_slice(&[17u8; 0xBEEF]);
        let s:VecStruct = decode(&inp[..]).unwrap();
        assert_eq!(s.Data.len(), 0xBEEF);
        assert_eq!(s.Data[4], 17);
        
        // four byte compactsize
        let mut inp:Vec<u8> = vec!(0xFE, 0, 0, 1, 0 );
        inp.extend_from_slice(&[17u8; 0x10000]);
        let s:VecStruct = decode(&inp[..]).unwrap();
        assert_eq!(s.Data.len(), 0x10000);
        assert_eq!(s.Data[4], 17);
        
        
    }
    

}