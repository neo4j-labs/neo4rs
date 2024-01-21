use std::fmt;

use bytes::{BufMut, Bytes, BytesMut};
use serde::ser::{self, Impossible, SerializeTupleVariant};

pub(super) struct Serializer {
    bytes: BytesMut,
}

impl Serializer {
    pub(super) fn empty() -> Self {
        Self::new(BytesMut::new())
    }

    pub(super) fn new(bytes: BytesMut) -> Self {
        Self { bytes }
    }

    pub(super) fn end(self) -> Bytes {
        self.bytes.freeze()
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.bytes.reserve(1);
        self.bytes.put_u8(0xC2 + (u8::from(v)));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        match v {
            -16..=127 => {
                self.bytes.reserve(1);
                self.bytes.put_i8(v as i8);
            }
            -128..=-17 => {
                self.bytes.reserve(2);
                self.bytes.put_u8(0xC8);
                self.bytes.put_i8(v as i8);
            }
            -32768..=32767 => {
                self.bytes.reserve(3);
                self.bytes.put_u8(0xC9);
                self.bytes.put_i16(v as i16);
            }
            -2147483648..=2147483647 => {
                self.bytes.reserve(5);
                self.bytes.put_u8(0xCA);
                self.bytes.put_i32(v as i32);
            }
            _ => {
                self.bytes.reserve(9);
                self.bytes.put_u8(0xCB);
                self.bytes.put_i64(v);
            }
        };
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.bytes.reserve(9);
        self.bytes.put_u8(0xC1);
        self.bytes.put_f64(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(u32::from(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let len = v.len();
        match len {
            0..=15 => {
                self.bytes.reserve(1 + len);
                self.bytes.put_u8(0x80 | len as u8);
            }
            16..=255 => {
                self.bytes.reserve(2 + len);
                self.bytes.put_u8(0xD0);
                self.bytes.put_u8(len as u8);
            }
            256..=65_535 => {
                self.bytes.reserve(3 + len);
                self.bytes.put_u8(0xD1);
                self.bytes.put_u16(len as u16);
            }
            65_536..=2_147_483_647 => {
                self.bytes.reserve(5 + len);
                self.bytes.put_u8(0xD2);
                self.bytes.put_u32(len as u32);
            }
            _ => return Err(Error::LengthOutOfBounds(len)),
        };
        self.bytes.put_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let len = v.len();
        match len {
            0..=255 => {
                self.bytes.reserve(2 + len);
                self.bytes.put_u8(0xCC);
                self.bytes.put_u8(len as u8);
            }
            256..=65_535 => {
                self.bytes.reserve(3 + len);
                self.bytes.put_u8(0xCD);
                self.bytes.put_u16(len as u16);
            }
            65_536..=2_147_483_647 => {
                self.bytes.reserve(5 + len);
                self.bytes.put_u8(0xCE);
                self.bytes.put_u32(len as u32);
            }
            _ => return Err(Error::LengthOutOfBounds(len)),
        };
        self.bytes.put_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.bytes.reserve(1);
        self.bytes.put_u8(0xC0);
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_tuple_variant(name, variant_index, variant, 0)?
            .end()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let mut s = self.serialize_tuple_variant(name, variant_index, variant, 1)?;
        (&mut s).serialize_field(value)?;
        s.end()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Error::UnknownLength)?;
        self.serialize_tuple(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        match len {
            0..=15 => {
                self.bytes.reserve(1);
                self.bytes.put_u8(0x90 | len as u8);
            }
            16..=255 => {
                self.bytes.reserve(2);
                self.bytes.put_u8(0xD4);
                self.bytes.put_u8(len as u8);
            }
            256..=65_535 => {
                self.bytes.reserve(3);
                self.bytes.put_u8(0xD5);
                self.bytes.put_u16(len as u16);
            }
            65_536..=2_147_483_647 => {
                self.bytes.reserve(5);
                self.bytes.put_u8(0xD6);
                self.bytes.put_u32(len as u32);
            }
            _ => return Err(Error::LengthOutOfBounds(len)),
        }
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // TODO: struct 8/16
        if len > 15 {
            return Err(Error::LengthOutOfBounds(len));
        }
        self.bytes.reserve(2);
        self.bytes.put_u8(0xB0 | len as u8);
        self.bytes.put_int(i64::from(variant_index), 1);
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len.ok_or(Error::UnknownLength)?;
        match len {
            0..=15 => {
                self.bytes.reserve(1);
                self.bytes.put_u8(0xA0 | len as u8);
            }
            16..=255 => {
                self.bytes.reserve(2);
                self.bytes.put_u8(0xD8);
                self.bytes.put_u8(len as u8);
            }
            256..=65_535 => {
                self.bytes.reserve(3);
                self.bytes.put_u8(0xD9);
                self.bytes.put_u16(len as u16);
            }
            65_536..=2_147_483_647 => {
                self.bytes.reserve(5);
                self.bytes.put_u8(0xDA);
                self.bytes.put_u32(len as u32);
            }
            _ => return Err(Error::LengthOutOfBounds(len)),
        }
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_tuple_variant(name, variant_index, variant, len)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        key.serialize(StrSerializer { inner: self })
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::Serializer::serialize_str(&mut **self, key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Unknown sequence length. The length of a sequence must be known in advance.")]
    UnknownLength,

    #[error("The value is too long: {0}")]
    LengthOutOfBounds(usize),

    #[error("Map key is not a string")]
    MapKeyNotString,

    // TODO: copy DeError
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Serialization(msg.to_string())
    }
}

struct StrSerializer<'a> {
    inner: &'a mut Serializer,
}

impl<'a> ser::Serializer for StrSerializer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_str(v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        Err(Error::MapKeyNotString)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        Err(Error::MapKeyNotString)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        Err(Error::MapKeyNotString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::MapKeyNotString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::MapKeyNotString)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_name() {
        // TODO: test serde token stuff
    }
}
