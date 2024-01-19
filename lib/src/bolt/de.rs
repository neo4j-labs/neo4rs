use std::{fmt, marker::PhantomData};

use bytes::{Buf, Bytes};
use serde::{
    de::{
        self, value::SeqDeserializer, DeserializeSeed, EnumAccess, IntoDeserializer as _,
        MapAccess, SeqAccess, VariantAccess, Visitor,
    },
    forward_to_deserialize_any,
};

pub(super) struct Deserializer<'a> {
    bytes: &'a mut Bytes,
}

impl<'a: 'de, 'de> de::Deserializer<'de> for Deserializer<'a> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 str
        string newtype_struct ignored_any
        map unit_struct struct enum identifier
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_next_item(Visitation::default(), visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_next_item(Visitation::MapAsSeq, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_next_item(Visitation::SeqAsTuple(len), visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct U32ToCharVisitor<V> {
            visitor: V,
        }

        impl<'de, V: Visitor<'de>> U32ToCharVisitor<V> {
            fn visit<T: TryInto<u32>, E: de::Error>(self, value: T) -> Result<V::Value, E> {
                let char = value
                    .try_into()
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or_else(|| {
                        de::Error::invalid_value(
                            de::Unexpected::Other("u32"),
                            &"a valid unicode code point",
                        )
                    })?;
                self.visitor.visit_char(char)
            }
        }

        impl<'de, V: Visitor<'de>> Visitor<'de> for U32ToCharVisitor<V> {
            type Value = V::Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a u32")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit(value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit(value)
            }
        }

        self.deserialize_u32(U32ToCharVisitor { visitor })
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_next_item(Visitation::BytesAsBytes, visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.bytes.is_empty() {
            visitor.visit_none()
        } else if self.bytes[0] == 0xC0 {
            self.bytes.advance(1);
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if !self.bytes.is_empty() && self.bytes[0] == 0xC0 {
            self.bytes.advance(1);
        }

        visitor.visit_unit()
    }
}

impl<'de> Deserializer<'de> {
    pub(super) fn new(bytes: &'de mut Bytes) -> Self {
        Self { bytes }
    }

    fn parse_next_item<V: Visitor<'de>>(
        self,
        v: Visitation,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if self.bytes.is_empty() {
            return Err(Error::Empty);
        }

        if let Visitation::SeqAsTuple(2) = v {
            return if self.bytes[0] == 0x92 {
                self.bytes.advance(1);
                Self::parse_list(v, 2, self.bytes, visitor)
            } else {
                visitor.visit_seq(ItemsParser::new(2, self.bytes))
            };
        }

        Self::parse(v, self.bytes, visitor)
    }

    fn skip_next_item(self) -> Result<(), Error> {
        self.parse_next_item(Visitation::BytesAsBytes, de::IgnoredAny)
            .map(|_| ())
    }

    fn parse<V: Visitor<'de>>(
        v: Visitation,
        bytes: &'de mut Bytes,
        visitor: V,
    ) -> Result<V::Value, Error> {
        let marker = bytes.get_u8();

        let (hi, lo) = (marker >> 4, marker & 0x0F);

        match hi {
            0x8 => Self::parse_string(lo as _, bytes, visitor),
            0x9 => Self::parse_list(v, lo as _, bytes, visitor),
            0xA => Self::parse_map(v, lo as _, bytes, visitor),
            0xB => Self::parse_struct(lo as _, bytes, visitor),
            0xC => match lo {
                0x0 => visitor.visit_unit(),
                0x1 => visitor.visit_f64(bytes.get_f64()),
                0x2 => visitor.visit_bool(false),
                0x3 => visitor.visit_bool(true),
                0x8 => visitor.visit_i8(bytes.get_i8()),
                0x9 => visitor.visit_i16(bytes.get_i16()),
                0xA => visitor.visit_i32(bytes.get_i32()),
                0xB => visitor.visit_i64(bytes.get_i64()),
                0xC => Self::parse_bytes(v, bytes.get_u8() as _, bytes, visitor),
                0xD => Self::parse_bytes(v, bytes.get_u16() as _, bytes, visitor),
                0xE => Self::parse_bytes(v, bytes.get_u32() as _, bytes, visitor),
                _ => Err(Error::UnknownMarker(marker)),
            },
            0xD => match lo {
                0x0 => Self::parse_string(bytes.get_u8() as _, bytes, visitor),
                0x1 => Self::parse_string(bytes.get_u16() as _, bytes, visitor),
                0x2 => Self::parse_string(bytes.get_u32() as _, bytes, visitor),
                0x4 => Self::parse_list(v, bytes.get_u8() as _, bytes, visitor),
                0x5 => Self::parse_list(v, bytes.get_u16() as _, bytes, visitor),
                0x6 => Self::parse_list(v, bytes.get_u32() as _, bytes, visitor),
                0x8 => Self::parse_map(v, bytes.get_u8() as _, bytes, visitor),
                0x9 => Self::parse_map(v, bytes.get_u16() as _, bytes, visitor),
                0xA => Self::parse_map(v, bytes.get_u32() as _, bytes, visitor),
                // C, D => struct 8/16
                _ => Err(Error::UnknownMarker(marker)),
            },
            0xE => Err(Error::UnknownMarker(marker)),
            _ => visitor.visit_i8(marker as i8),
        }
    }

    fn parse_bytes<V: Visitor<'de>>(
        v: Visitation,
        len: usize,
        bytes: &'de mut Bytes,
        visitor: V,
    ) -> Result<V::Value, Error> {
        debug_assert!(bytes.len() >= len);

        let bytes = bytes.split_to(len);
        if v.visit_bytes_as_bytes() {
            let bytes: &'de [u8] = unsafe { std::mem::transmute(bytes.as_ref()) };
            visitor.visit_borrowed_bytes(bytes)
        } else {
            visitor.visit_seq(SeqDeserializer::new(bytes.into_iter()))
        }
    }

    fn parse_string<V: Visitor<'de>>(
        len: usize,
        bytes: &'de mut Bytes,
        visitor: V,
    ) -> Result<V::Value, Error> {
        debug_assert!(bytes.len() >= len);

        let bytes = bytes.split_to(len);
        let bytes: &'de [u8] = unsafe { std::mem::transmute(bytes.as_ref()) };

        match std::str::from_utf8(bytes) {
            Ok(s) => visitor.visit_borrowed_str(s),
            Err(e) => Err(Error::InvalidUtf8(e)),
        }
    }

    fn parse_list<V: Visitor<'de>>(
        v: Visitation,
        len: usize,
        bytes: &mut Bytes,
        visitor: V,
    ) -> Result<V::Value, Error> {
        let items = ItemsParser::new(len, bytes);
        match v {
            Visitation::SeqAsTuple(tuple_len) => match len.checked_sub(tuple_len) {
                None => Err(Error::InvalidLength {
                    expected: tuple_len,
                    actual: len,
                }),
                Some(excess) => visitor.visit_seq(items.with_excess(excess)),
            },
            _ => visitor.visit_seq(items),
        }
    }

    fn parse_map<V: Visitor<'de>>(
        v: Visitation,
        len: usize,
        bytes: &mut Bytes,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if v.visit_map_as_seq() {
            visitor.visit_seq(ItemsParser::new(len, bytes))
        } else {
            visitor.visit_map(ItemsParser::new(len, bytes))
        }
    }

    fn parse_struct<V: Visitor<'de>>(
        len: usize,
        bytes: &mut Bytes,
        visitor: V,
    ) -> Result<V::Value, Error> {
        let tag = bytes.get_u8();
        visitor.visit_enum(StructParser {
            tag,
            items: ItemsParser::new(len, bytes),
        })
    }
}

struct ItemsParser<'a> {
    len: usize,
    excess: usize,
    bytes: SharedBytes<'a>,
}

impl<'a> ItemsParser<'a> {
    fn new(len: usize, bytes: &'a mut Bytes) -> Self {
        Self {
            len,
            excess: 0,
            bytes: SharedBytes::new(bytes),
        }
    }

    fn with_excess(mut self, excess: usize) -> Self {
        self.excess = excess;
        self
    }
}

impl<'a, 'de> SeqAccess<'de> for ItemsParser<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.len == 0 {
            let bytes = self.bytes.get();
            for _ in 0..self.excess {
                Deserializer { bytes }.skip_next_item()?;
            }
            return Ok(None);
        }
        self.len -= 1;

        let bytes = self.bytes.get();
        seed.deserialize(Deserializer { bytes }).map(Some)
    }
}

impl<'a, 'de> MapAccess<'de> for ItemsParser<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;

        let bytes = self.bytes.get();
        seed.deserialize(Deserializer { bytes }).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let bytes = self.bytes.get();
        seed.deserialize(Deserializer { bytes })
    }
}

impl<'a, 'de> VariantAccess<'de> for ItemsParser<'a> {
    type Error = Error;

    fn unit_variant(mut self) -> Result<(), Self::Error> {
        self.next_value()
        // if self.len != 0 {
        //     return Err(Error::InvalidLength {
        //         expected: 0,
        //         actual: self.len,
        //     });
        // }
        // Ok(())
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.next_value_seed(seed)
        // let bytes = self.bytes.get();
        // seed.deserialize(Deserializer { bytes })
    }

    fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct TupleVariant<V> {
            len: usize,
            visitor: V,
        }

        impl<'de, V> DeserializeSeed<'de> for TupleVariant<V>
        where
            V: Visitor<'de>,
        {
            type Value = V::Value;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_tuple(self.len, self.visitor)
            }
        }

        self.next_value_seed(TupleVariant { len, visitor })

        // if len != self.len {
        //     return Err(Error::InvalidLength {
        //         expected: len,
        //         actual: self.len,
        //     });
        // }
        // visitor.visit_seq(self)
    }

    fn struct_variant<V>(
        mut self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct StructVariant<V> {
            visitor: V,
        }

        impl<'de, V> DeserializeSeed<'de> for StructVariant<V>
        where
            V: Visitor<'de>,
        {
            type Value = V::Value;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_map(self.visitor)
            }
        }

        self.next_value_seed(StructVariant { visitor })

        // visitor.visit_map(self)
    }
}

struct StructParser<'a> {
    tag: u8,
    items: ItemsParser<'a>,
}

impl<'a, 'de> EnumAccess<'de> for StructParser<'a> {
    type Error = Error;

    type Variant = ItemsParser<'a>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.tag.into_deserializer())
            .map(|o| (o, self.items))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Visitation {
    #[default]
    Default,
    BytesAsBytes,
    MapAsSeq,
    SeqAsTuple(usize),
}

impl Visitation {
    fn visit_bytes_as_bytes(self) -> bool {
        matches!(self, Self::BytesAsBytes)
    }

    fn visit_map_as_seq(self) -> bool {
        matches!(self, Self::MapAsSeq)
    }
}

struct SharedBytes<'a> {
    bytes: *mut Bytes,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> SharedBytes<'a> {
    fn new(bytes: &'a mut Bytes) -> Self {
        Self {
            bytes,
            _lifetime: PhantomData,
        }
    }

    fn get<'x>(&mut self) -> &'x mut Bytes {
        unsafe { &mut *self.bytes }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Not enough data to parse a bolt stream.")]
    Empty,

    #[error("Unknown marker: {0:02X}")]
    UnknownMarker(u8),

    #[error("The bytes do no contain valid UTF-8 to produce a string: {0}")]
    InvalidUtf8(#[source] std::str::Utf8Error),

    #[error("Invalid length: expected {expected}, actual {actual}")]
    InvalidLength { expected: usize, actual: usize },

    // TODO: copy DeError
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::DeserializationError(msg.to_string())
    }
}
