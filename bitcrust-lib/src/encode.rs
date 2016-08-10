use std::io;
use std::mem;
use std::error;
use std::error::Error;
use std::fmt;
use serde;

  

/// Encodes a memory object into a byte slice using bitcoin-protocol conventions
pub fn encode<T: serde::ser::Serialize>(obj: &T) -> Result<Vec<u8>, SerializeError> {
    let target  = Vec::new();
    let mut cur    = io::Cursor::new(target);
    
    {
        let mut ser = BinSerializer::new(&mut cur);
        try!(obj.serialize(&mut ser));
    }
    Ok(cur.into_inner())
    
}

/// Local macro to write a little-endian primitive to a writer
/// 
/// # Examples
/// ```
/// let r = write_primitive(i32, &rdr, 4);
/// ``` 
macro_rules! write_primitive(
    ($val:expr, $wrt:expr, $sz:expr) => 
    {
        {
            let swapped = if cfg!(target_endian = "big") {
                $val.swap_bytes()
            }
            else {
                $val
            };
            
            let buf:[u8; $sz] = unsafe { mem::transmute(swapped) };
            
            try!($wrt.write_all(&buf));
        }                
    };
);


/// Binary serde serializer
///
/// This serializes conforming to the bitcoin-protocol
/// Primitives are serialized LittleEndian
pub struct BinSerializer<'a, T: io::Write + 'a> {
    writer: &'a mut T
}

impl<'a, T: io::Write + 'a> BinSerializer<'a, T> {
    
    /// Constructs a new deserializer
    fn new(writer: &'a mut T) -> BinSerializer<'a, T> {
        BinSerializer {
            writer: writer
        }
    }

    /// Write a variable length integer to the writer
    fn write_compact_size(&mut self, len: u64) -> Result<(), SerializeError> {
        match len {
            0...0xfc      => {
                write_primitive!(len as u8,  self.writer, 1)
            },
            0xfd...0xffff => { 
                write_primitive!(0xfdu8,  self.writer, 1);
                write_primitive!(len as u16, self.writer, 2) 
            },
            0x10000...0xffffffff => { 
                write_primitive!(0xfeu8,  self.writer, 1);
                write_primitive!(len as u32, self.writer, 4) 
            },
            _ => {
                write_primitive!(0xffu8,  self.writer, 1);
                write_primitive!(len as u64, self.writer, 8) 
            }
           
        };
        Ok(())
             
    }
}

    
#[derive(Debug)]
pub enum SerializeError {
    IncorrectLength
}

/// Conversion from IO error is always incorrect length
/// As we can assume the entire buffer has been read 
impl From<io::Error> for SerializeError {
    fn from(_: io::Error) -> SerializeError {
        SerializeError::IncorrectLength        
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(formatter)
    }
}

impl error::Error for SerializeError {
    fn description(&self) -> &str {
        "Incorrect length"
    }
}

/// Serde wants our error to implement `serde::de::Error`,
/// but they aren't used here
impl serde::ser::Error for SerializeError { 
    fn custom<T: Into<String>>(_: T) -> Self {
        unimplemented!()
    }

    
}


/// Adds a function that is required but not implemented
macro_rules! implement_unimplemented( ($func:ident, $typ:ty) => {
    fn $func(&mut self, _: $typ) -> Result<(), Self::Error> {
        unimplemented!();
    }
    
};);

impl<'a, T: io::Write > serde::ser::Serializer for BinSerializer<'a, T> {
    type Error = SerializeError;
    
    /// Not needed field-types throw an unimplemented error
    implement_unimplemented!(serialize_bool, bool);
    implement_unimplemented!(serialize_u64, u64);
    implement_unimplemented!(serialize_f64, f64);
    implement_unimplemented!(serialize_str, &str);
    
    
    /// Primitives
    
    #[inline]
    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        write_primitive!(v, self.writer, 1);
        Ok(())
    }
    
    
    #[inline]
    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error>
    {
        write_primitive!(v, self.writer, 2);
        Ok(())
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error>
    {
        write_primitive!(v, self.writer, 4);
        Ok(())
    }
    
    #[inline]
    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error>
    {
        write_primitive!(v, self.writer, 4);
        Ok(())
    }
    
    #[inline]
    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error>
    {
        write_primitive!(v, self.writer, 8);
        Ok(())
    }
    
    fn serialize_unit(&mut self) -> Result<(), Self::Error> { Ok(()) }
    
    fn serialize_none(&mut self) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn serialize_some<Q>(&mut self, _: Q) -> Result<(), Self::Error>
        where Q: serde::Serialize,
    {
        unimplemented!();
    }

    

    fn serialize_tuple<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: serde::ser::SeqVisitor,
    {
        while let Some(()) = try!(visitor.visit(self)) { }

        Ok(())
    }

    fn serialize_seq_elt<V>(&mut self, value: V) -> Result<(), Self::Error>
        where V: serde::Serialize,
    {
        value.serialize(self)
    }
    
    /// Serializes an element of a struct.
    ///
    /// By default, struct elements are serialized as a map element with the field name as the key.
    #[inline]
    fn serialize_struct_elt<V>(&mut self,
                               _: &'static str,
                               value: V) -> Result<(), Self::Error>
        where V: serde::Serialize,
    {
        value.serialize(self)
    }
    

    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: serde::ser::MapVisitor,
    {
        let len = match visitor.len() {
            Some(len) => len,
            None => panic!("do not know how to serialize a map with no length"),
        };

        try!(self.serialize_usize(len));

        while let Some(()) = try!(visitor.visit(self)) { }

        Ok(())
    }

    fn serialize_map_elt<K, V>(&mut self, _: K, value: V) -> Result<(), Self::Error>
        where K: serde::Serialize,
              V: serde::Serialize,
    {
        
        value.serialize(self)
    }

    fn serialize_fixed_size_array<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: serde::ser::SeqVisitor,
    {
        while let Some(()) = try!(visitor.visit(self)) { };
        Ok(())
    }

    /// This method hints that the `Serialize` type is expecting a sequence value. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    #[inline]
    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: serde::ser::SeqVisitor,
    {
        

        let len = match visitor.len() {
            None    => return Ok(()),
            Some(x) => x
        };
        
        try!(self.write_compact_size(len as u64));
        
        while let Some(()) = try!(visitor.visit(self)) { };
        
         
        Ok(())
        
    }
    
    
    
    
    #[inline]
    fn serialize_struct<V>(&mut self,
                       _name: &'static str,
                       mut visitor: V) -> Result<(), Self::Error>
        where V: serde::ser::MapVisitor,
    {
        while let Some(()) = try!(visitor.visit(self)) { }  ;
        
        Ok(())  

    }
    
    #[inline]
    fn serialize_tuple_struct<V>(&mut self,
                       _name: &'static str,
                       mut visitor: V) -> Result<(), Self::Error>
        where V: serde::ser::SeqVisitor,
    {
        while let Some(()) = try!(visitor.visit(self)) { };
        
        Ok(())


    }
}


#[cfg(test)]
mod tests
{
    use super::*;
    
    
    
    #[test]
    fn test_serialize_u8()
    {
       
        #[derive(Serialize)]
        struct Stru {
            a: u8,
        }
        //let mut bytes: Vec<u8> = vec![];
        let bytes = encode(&Stru { a: 3}).unwrap();
         
        assert_eq!(bytes[0], 3);
        assert_eq!(bytes.len(), 1);
          
    }
    
    #[test]
    fn test_serialize_fixed_size()
    {
        #[derive(Serialize)]
        struct H4 {
            a: [u8; 4],
        }
        
        #[derive(Serialize)]
        struct Stru {
            a: H4,
        }
        
        let bytes = encode(&Stru { a: H4 { a: [1u8,2,3,4]  } } ).unwrap();
         
        assert_eq!(bytes[1], 2);
        assert_eq!(bytes.len(), 4);
          
    }
    
    #[test]
    fn test_serialize_compact_size()
    {
       
        
        #[derive(Serialize)]
        struct Stru {
            a: Vec<u8>,
        }
        
        let bytes = encode(&Stru { a: vec![1u8,2u8,3u8,4u8, 5u8] } ).unwrap();
         
        assert_eq!(bytes[0], 5);
        assert_eq!(bytes[2], 2);
        assert_eq!(bytes.len(), 6);
          
    }
    

}