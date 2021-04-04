use byteorder::{LittleEndian, ReadBytesExt};
use serde::de::{self, Deserialize, DeserializeSeed, Visitor, SeqAccess};
use serde;
use std::str;
use {Error, Result};


pub fn decode_compact_size<R: ::std::io::Read>(reader: &mut R) -> Result<usize> {

    let byte1: u8 = reader.read_u8()?;
    let result = match byte1 {
        0xff => reader.read_u64::<LittleEndian>()? as usize,
        0xfe => reader.read_u32::<LittleEndian>()? as usize,
        0xfd => reader.read_u16::<LittleEndian>()? as usize,
        _ => byte1 as usize
    };
    Ok(result)
}



pub struct Deserializer<'de> {
    bytes: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        Deserializer { bytes: bytes }
    }

    pub fn deserialize<T>(&mut self) -> Result<T> where T: serde::Deserialize<'de>
    {
        Deserialize::deserialize(self)
    }

    fn decode_compact_size(&mut self) -> Result<usize> {

        let byte1: u8 = Deserialize::deserialize(&mut *self)?;
        let result = match byte1 {
            0xff => { let u: u64 = Deserialize::deserialize(&mut *self)?; u as usize },
            0xfe => { let u: u32 = Deserialize::deserialize(&mut *self)?; u as usize },
            0xfd => { let u: u16 = Deserialize::deserialize(&mut *self)?; u as usize },
            _ => byte1 as usize
        };
        Ok(result)
    }

    #[inline]
    fn read_slice(&mut self) -> Result<&'de [u8]> {
        let len = self.decode_compact_size()?;
        let (slice, rest) = self.bytes.split_at(len);
        self.bytes = rest;
        Ok(slice)
    }

}

macro_rules! impl_nums {
    ($ty:ty, $dser_method:ident, $visitor_method:ident, $reader_method:ident) => {
        #[inline]
        fn $dser_method<V>(self, visitor: V) -> Result<V::Value>
            where V: Visitor<'de>
        {
            let value = self.bytes.$reader_method::<LittleEndian>()?;
            visitor.$visitor_method(value)
        }
    };
}

macro_rules! impl_not_implemented {
    ($method: ident) => {
        fn $method<V>(self, _visitor: V) -> Result<V::Value>
            where V: Visitor<'de>
        {
            unimplemented!()
        }
    };
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    impl_not_implemented!(deserialize_any);
    impl_not_implemented!(deserialize_ignored_any);
    impl_not_implemented!(deserialize_bool);
    impl_not_implemented!(deserialize_option);
    impl_not_implemented!(deserialize_unit);
    impl_not_implemented!(deserialize_char);
    impl_not_implemented!(deserialize_map);
    impl_not_implemented!(deserialize_str);
    impl_not_implemented!(deserialize_string);
    impl_not_implemented!(deserialize_identifier);

    impl_nums!(u16, deserialize_u16, visit_u16, read_u16);
    impl_nums!(u32, deserialize_u32, visit_u32, read_u32);
    impl_nums!(u64, deserialize_u64, visit_u64, read_u64);
    impl_nums!(i16, deserialize_i16, visit_i16, read_i16);
    impl_nums!(i32, deserialize_i32, visit_i32, read_i32);
    impl_nums!(i64, deserialize_i64, visit_i64, read_i64);
    impl_nums!(f32, deserialize_f32, visit_f32, read_f32);
    impl_nums!(f64, deserialize_f64, visit_f64, read_f64);

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_u8(self.bytes.read_u8()?)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_i8(self.bytes.read_i8()?)
    }



    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_borrowed_bytes(self.read_slice()?)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_borrowed_bytes(self.read_slice()?)
    }

    #[inline]
    fn deserialize_enum<V>(self,
                           _enum: &'static str,
                           _variants: &'static [&'static str],
                           _visitor: V)
                           -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    // Tuple deserialization is invoked for a fixed length array
    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(self)

    }

    // Variable length sequence is decoded as compact-length prefixed
    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        struct SeqAccess<'a, 'de: 'a> {
            deserializer: &'a mut Deserializer<'de>,
            remaining: usize,
        }

        impl<'de, 'a> de::SeqAccess<'de> for SeqAccess<'a, 'de> {
            type Error = Error;

            #[inline]
            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
                where T: DeserializeSeed<'de>
            {
                if self.remaining > 0 {
                    self.remaining -= 1;
                    seed.deserialize(&mut *self.deserializer).map(Some)
                } else {
                    Ok(None)
                }
            }
        }

        let len = self.decode_compact_size()?;

        visitor.visit_seq(SeqAccess {
            deserializer: self,
            remaining: len,
        })
    }


    #[inline]
    fn deserialize_struct<V>(self,
                             _name: &str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(self)

    }


    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_tuple_struct<V>(self,
                                   _name: &'static str,
                                   _len: usize,
                                   visitor: V)
                                   -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(self)
    }


}

// For tuples, structs, tuple structs, and fixed size seqs.
impl<'de> SeqAccess<'de> for Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        seed.deserialize(self).map(Some)
    }
}

