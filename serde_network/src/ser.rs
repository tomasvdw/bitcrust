use byteorder::{LittleEndian, WriteBytesExt};
use serde;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
                 SerializeMap, SerializeStruct, SerializeStructVariant};
use std::io::Write;
use {Error, Result};

pub fn encode_compact_size<W: Write>(writer: &mut W, sz: usize) -> Result<()> {
    if sz < 0xFD {
        writer.write_u8(sz as u8)?;
    }
    else if sz < 0xFFFF {
        writer.write_u8(0xFD)?;
        writer.write_u16::<LittleEndian>(sz as u16)?;
    }
    else if sz < 0xFFFF_FFFF {
        writer.write_u8(0xFE)?;
        writer.write_u32::<LittleEndian>(sz as u32)?;
    }
    else {
        writer.write_u8(0xFF)?;
        writer.write_u64::<LittleEndian>(sz as u64)?;
    }

    Ok(())
}



pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
    where W: Write
{
    pub fn new(w: W) -> Self {
        Serializer { writer: w }
    }

    pub fn serialize<T>(&mut self, value: &T) -> Result<()> where T: serde::Serialize {
        value.serialize(self)
    }

}

impl<'a, W> serde::Serializer for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;


    #[inline]
    fn serialize_unit(self) -> Result<()> {

        unimplemented!()
    }

    #[inline]
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer.write_u8(v as u8).map_err(From::from)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.writer.write_u8(v).map_err(From::from)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.writer.write_u16::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.writer.write_u32::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.writer.write_u64::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {

        self.writer.write_i8(v).map_err(From::from)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.writer.write_i16::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.writer.write_i32::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.writer.write_i64::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.writer.write_f32::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.writer.write_f64::<LittleEndian>(v).map_err(From::from)
    }

    #[inline]
    fn serialize_str(self, _v: &str) -> Result<()> {
        unimplemented!()
    }

    #[inline]
    fn serialize_char(self, _c: char) -> Result<()> {
        unimplemented!()
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {

        self.writer.write_all(v).map_err(From::from)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.writer.write_u8(0).map_err(From::from)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<()>
        where T: serde::Serialize
    {
        self.writer.write_u8(1)?;
        v.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.expect("do not know how to serialize a sequence with no length");
        encode_compact_size(&mut self.writer, len)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!()
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(self,
                                _name: &'static str,
                                variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            variant_index: u32,
                                            _variant: &'static str,
                                            value: &T)
                                            -> Result<()>
        where T: serde::ser::Serialize
    {
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              variant_index: u32,
                              _variant: &'static str)
                              -> Result<()> {
        self.serialize_u32(variant_index)
    }
}

impl<'a, W> SerializeSeq for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> SerializeTuple for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> SerializeTupleStruct for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> SerializeTupleVariant for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> SerializeMap for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, key: &K) -> Result<()>
        where K: serde::Serialize
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> SerializeStruct for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<V: ?Sized>(&mut self, _key: &'static str, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> SerializeStructVariant for &'a mut Serializer<W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<V: ?Sized>(&mut self, _key: &'static str, value: &V) -> Result<()>
        where V: serde::Serialize
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}
