#![allow(unused_imports, dead_code)]

use bytes::Bytes;
use serde::{
    de::{Deserialize, DeserializeOwned, DeserializeSeed},
    Deserializer,
};

pub mod de;
pub mod ser;
#[cfg(all(test, debug_assertions))]
pub use debug::Dbg;
#[cfg(test)]
pub use value::{bolt, BoltBytesBuilder};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Data {
    bytes: Bytes,
    keep_alive: Bytes,
}

impl Data {
    pub fn new(bytes: Bytes) -> Self {
        let keep_alive = bytes.clone();
        Self { bytes, keep_alive }
    }

    #[cfg(test)]
    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut Bytes {
        &mut self.bytes
    }

    pub fn reset(&mut self) {
        self.bytes = self.keep_alive.clone();
    }

    pub fn into_inner(self) -> Bytes {
        self.keep_alive
    }

    pub(crate) fn reset_to(&mut self, bytes: Bytes) -> Bytes {
        let old = std::mem::replace(&mut self.keep_alive, bytes.clone());
        self.bytes = bytes;
        old
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawBytes::deserialize(deserializer).map(|bytes| Self::new(bytes.0))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RawBytes(pub(crate) Bytes);

impl<'de> Deserialize<'de> for RawBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawBytesVisitor;

        impl<'de> serde::de::Visitor<'de> for RawBytesVisitor {
            type Value = Bytes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a pointer to a Bytes instance")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let bytes = v as usize as *mut Bytes;
                let bytes = unsafe { Box::from_raw(bytes) };
                let bytes = *bytes;
                Ok(bytes)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Bytes::copy_from_slice(v))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Bytes::from(v))
            }
        }

        deserializer
            .deserialize_newtype_struct("__neo4rs::RawBytes", RawBytesVisitor)
            .map(Self)
    }
}

/// Parse and deserialize a packstream value from the given bytes.
pub fn from_bytes<T>(mut bytes: Bytes) -> Result<T, de::Error>
where
    T: DeserializeOwned,
{
    let de = de::Deserializer::new(&mut bytes);
    let value = T::deserialize(de)?;

    Ok(value)
}

/// Parse and deserialize a packstream value from the given bytes.
pub(crate) fn from_bytes_ref<'de, T>(bytes: &'de mut Data) -> Result<T, de::Error>
where
    T: Deserialize<'de> + 'de,
{
    let de = de::Deserializer::new(bytes.bytes_mut());
    let value = T::deserialize(de)?;

    Ok(value)
}

/// Parse and deserialize a packstream value from the given bytes.
pub(crate) fn from_bytes_seed<'de, S>(bytes: &'de mut Data, seed: S) -> Result<S::Value, de::Error>
where
    S: DeserializeSeed<'de>,
{
    let de = de::Deserializer::new(bytes.bytes_mut());
    seed.deserialize(de)
}

/// Serialize and packstream encode the given value.
pub fn to_bytes<T>(value: &T) -> Result<Bytes, ser::Error>
where
    T: serde::Serialize,
{
    let mut ser = ser::Serializer::empty();
    value.serialize(&mut ser)?;

    Ok(ser.end())
}

#[cfg(test)]
mod value {
    use bytes::{BufMut, Bytes, BytesMut};

    pub fn bolt() -> BoltBytesBuilder {
        BoltBytesBuilder::new()
    }

    pub struct BoltBytesBuilder {
        data: BytesMut,
    }

    #[allow(unused)]
    impl BoltBytesBuilder {
        pub fn new() -> Self {
            Self {
                data: BytesMut::new(),
            }
        }

        pub fn null(mut self) -> Self {
            self.data.put_u8(0xC0);
            self
        }

        pub fn bool(mut self, value: bool) -> Self {
            self.data.put_u8(if value { 0xC3 } else { 0xC2 });
            self
        }

        pub fn tiny_int(mut self, value: i8) -> Self {
            self.data.put_i8(value);
            self
        }

        pub fn int8(mut self, value: i8) -> Self {
            self.data.put_u8(0xC8);
            self.data.put_i8(value);
            self
        }

        pub fn int16(mut self, value: i16) -> Self {
            self.data.put_u8(0xC9);
            self.data.put_i16(value);
            self
        }

        pub fn int32(mut self, value: i32) -> Self {
            self.data.put_u8(0xCA);
            self.data.put_i32(value);
            self
        }

        pub fn int64(mut self, value: i64) -> Self {
            self.data.put_u8(0xCB);
            self.data.put_i64(value);
            self
        }

        pub fn float(mut self, value: f64) -> Self {
            self.data.put_u8(0xC1);
            self.data.put_f64(value);
            self
        }

        pub fn bytes8(mut self, len: u8, value: &[u8]) -> Self {
            self.data.put_u8(0xCC);
            self.data.put_u8(len);
            self.data.put_slice(value);
            self
        }

        pub fn bytes16(mut self, len: u16, value: &[u8]) -> Self {
            self.data.put_u8(0xCD);
            self.data.put_u16(len);
            self.data.put_slice(value);
            self
        }

        pub fn bytes32(mut self, len: u32, value: &[u8]) -> Self {
            self.data.put_u8(0xCE);
            self.data.put_u32(len);
            self.data.put_slice(value);
            self
        }

        pub fn tiny_string(mut self, value: &str) -> Self {
            assert!(value.len() <= 15);
            self.data.put_u8(0x80 | value.len() as u8);
            self.data.put_slice(value.as_bytes());
            self
        }

        pub fn string8(mut self, value: &str) -> Self {
            assert!(value.len() <= 255);
            self.data.put_u8(0xD0);
            self.data.put_u8(value.len() as u8);
            self.data.put_slice(value.as_bytes());
            self
        }

        pub fn string16(mut self, value: &str) -> Self {
            assert!(value.len() <= 65535);
            self.data.put_u8(0xD1);
            self.data.put_u16(value.len() as u16);
            self.data.put_slice(value.as_bytes());
            self
        }

        pub fn string32(mut self, value: &str) -> Self {
            assert!(value.len() <= 2147483647);
            self.data.put_u8(0xD2);
            self.data.put_u32(value.len() as u32);
            self.data.put_slice(value.as_bytes());
            self
        }

        pub fn tiny_list(mut self, len: u8) -> Self {
            self.data.put_u8(0x90 | len);
            self
        }

        pub fn list8(mut self, len: u8) -> Self {
            self.data.put_u8(0xD4);
            self.data.put_u8(len);
            self
        }

        pub fn list16(mut self, len: u16) -> Self {
            self.data.put_u8(0xD5);
            self.data.put_u16(len);
            self
        }

        pub fn list32(mut self, len: u32) -> Self {
            self.data.put_u8(0xD6);
            self.data.put_u32(len);
            self
        }

        pub fn tiny_map(mut self, len: u8) -> Self {
            self.data.put_u8(0xA0 | len);
            self
        }

        pub fn map8(mut self, len: u8) -> Self {
            self.data.put_u8(0xD8);
            self.data.put_u8(len);
            self
        }

        pub fn map16(mut self, len: u16) -> Self {
            self.data.put_u8(0xD9);
            self.data.put_u16(len);
            self
        }

        pub fn map32(mut self, len: u32) -> Self {
            self.data.put_u8(0xDA);
            self.data.put_u32(len);
            self
        }

        pub fn structure(mut self, len: u8, tag: u8) -> Self {
            self.data.put_u8(0xB0 | len);
            self.data.put_u8(tag);
            self
        }

        pub fn extend(mut self, bytes: impl AsRef<[u8]>) -> Self {
            self.data.put_slice(bytes.as_ref());
            self
        }

        pub fn build(self) -> Bytes {
            self.data.freeze()
        }
    }

    impl Default for BoltBytesBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl From<BoltBytesBuilder> for Bytes {
        fn from(builder: BoltBytesBuilder) -> Self {
            builder.build()
        }
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use bytes::{Buf as _, Bytes};

    pub struct Dbg<'a>(pub &'a Bytes);

    struct Tag(u8);

    impl std::fmt::Display for Tag {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.0.is_ascii_alphanumeric() {
                write!(f, "'{}' (0x{:02X})", self.0 as char, self.0)
            } else {
                write!(f, "0x{:02X}", self.0)
            }
        }
    }

    impl std::fmt::Debug for Dbg<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut bytes = self.0.clone();

            macro_rules! string {
                ($bytes:expr) => {
                    match ::std::str::from_utf8(&$bytes) {
                        Ok(s) => write!(f, "\"{}\"", s),
                        Err(e) => write!(f, "[{:?}] (invalid utf8: {:?})", bytes, e),
                    }
                };
            }

            macro_rules! split {
                ($bytes:ident.split_to($len:expr)) => {{
                    let len = $len;
                    $bytes.split_to(len)
                }};
            }

            while !bytes.is_empty() {
                let marker = bytes.get_u8();
                write!(f, " {:02X} ", marker)?;

                let (hi, lo) = (marker >> 4, marker & 0x0F);

                match hi {
                    0x8 => string!(bytes.split_to(lo as _)),
                    0x9 => write!(f, "(List[{}])", lo),
                    0xA => write!(f, "(Map[{}])", lo),
                    0xB => write!(f, "(Struct<tag={} len={1}>)", Tag(bytes.get_u8()), lo),
                    0xC => match lo {
                        0x0 => write!(f, "NULL"),
                        0x1 => write!(f, "Float({})", bytes.get_f64()),
                        0x2 => write!(f, "FALSE"),
                        0x3 => write!(f, "TRUE"),
                        0x8 => write!(f, "Int({})", bytes.get_i8()),
                        0x9 => write!(f, "Int({})", bytes.get_i16()),
                        0xA => write!(f, "Int({})", bytes.get_i32()),
                        0xB => write!(f, "Int({})", bytes.get_i64()),
                        0xC => write!(f, "[{:0X}]", split!(bytes.split_to(bytes.get_u8() as _))),
                        0xD => write!(f, "[{:0X}]", split!(bytes.split_to(bytes.get_u16() as _))),
                        0xE => write!(f, "[{:0X}]", split!(bytes.split_to(bytes.get_u32() as _))),
                        _ => write!(f, "Unknown marker"),
                    },
                    0xD => match lo {
                        0x0 => string!(split!(bytes.split_to(bytes.get_u8() as _))),
                        0x1 => string!(split!(bytes.split_to(bytes.get_u16() as _))),
                        0x2 => string!(split!(bytes.split_to(bytes.get_u32() as _))),
                        0x4 => write!(f, "(List[{}])", bytes.get_u8()),
                        0x5 => write!(f, "(List[{}])", bytes.get_u16()),
                        0x6 => write!(f, "(List[{}])", bytes.get_u32()),
                        0x8 => write!(f, "(Map[{}])", bytes.get_u8()),
                        0x9 => write!(f, "(Map[{}])", bytes.get_u16()),
                        0xA => write!(f, "(Map[{}])", bytes.get_u32()),
                        // C, D => struct 8/16
                        _ => write!(f, "Unknown marker"),
                    },
                    0xE => write!(f, "Unknown marker"),
                    _ => write!(f, "Int({})", marker as i8),
                }?
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fmt::Debug};

    use crate::packstream::bolt;

    use super::*;

    use serde::{Deserialize, Serialize};
    use test_case::test_case;

    #[test]
    fn unit() {
        roundtrip(&[], ());
    }

    #[test]
    fn null() {
        roundtrip(&[0xC0], None::<i32>);
    }

    #[test_case(0xC2, false ; "false value")]
    #[test_case(0xC3, true ; "true value")]
    fn boolean(byte: u8, expected: bool) {
        roundtrip(&[byte], expected);
    }

    #[test_case(&[0x2A], 42; "tiny_int")]
    #[test_case(&[0xF4], -12; "neg_tiny_int")]
    #[test_case(&[0xC8, 0xD6], -42; "int_8")]
    #[test_case(&[0xC9, 0x00, 0xD6], 214; "int_16_u8")]
    #[test_case(&[0xC9, 0x05, 0x39], 1337; "int_16_i16")]
    #[test_case(&[0xC9, 0xFA, 0xC7], -1337; "neg_int_16")]
    #[test_case(&[0xCA, 0x00, 0x00, 0xA4, 0x64], 42084; "int_32_u16")]
    #[test_case(&[0xCA, 0x00, 0x40, 0x1B, 0x79], 4201337; "int_32_i32")]
    #[test_case(&[0xCA, 0xFF, 0xBF, 0xE4, 0x87], -4201337; "neg_int_32")]
    #[test_case(&[0xCB, 0x00, 0x00, 0x00, 0x00, 0xfa, 0x97, 0x05, 0x79], 4204201337; "int_64_u32")]
    #[test_case(&[0xCB, 0x00, 0x00, 0x00, 0x03, 0x1c, 0xf0, 0x6c, 0xc4], 13370420420; "int_64_i64")]
    #[test_case(&[0xCB, 0xFF, 0xFF, 0xFF, 0xFF, 0x05, 0x68, 0xFA, 0x87], -4204201337; "neg_int_64")]
    fn int(input: &'static [u8], expected: i64) {
        roundtrip(input, expected);
    }

    #[test_case(&[0x2A]; "tiny_int")]
    #[test_case(&[0xC8, 0x2A]; "int_8")]
    #[test_case(&[0xC9, 0x00, 0x2A]; "int_16")]
    #[test_case(&[0xCA, 0x00, 0x00, 0x00, 0x2A]; "int_32")]
    #[test_case(&[0xCB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A]; "int_64")]
    fn int_42(input: &'static [u8]) {
        let bytes = Bytes::from_static(input);
        assert_eq!(from_bytes::<i64>(bytes).unwrap(), 42);
    }

    #[test_case(&[0xC1, 0x3F, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE], 1.23_f64; "float_f64")]
    #[test_case(&[0xC1, 0x3F, 0xF3, 0xAE, 0x14, 0x80, 0x00, 0x00, 0x00], 1.23_f32 as f64; "float_f32")]
    fn float(input: &'static [u8], expected: f64) {
        roundtrip(input, expected)
    }

    #[test_case(&[0xCC, 0x00], Bytes::new(); "empty bytes")]
    #[test_case(&[0xCC, 0x03, 0x01, 0x02, 0x03], Bytes::from_static(&[1, 2, 3]); "123")]
    #[test_case(&[0xCC, 0x0C, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x72, 0x75, 0x73, 0x74, 0x21], Bytes::from_static(b"hello, rust!"); "hell")]
    fn bytes(input: &'static [u8], expected: Bytes) {
        roundtrip(input, expected)
    }

    #[test_case(&[0xCC, 0x00], Bytes::new(); "empty bytes")]
    #[test_case(&[0xCC, 0x04, 0x68, 0x65, 0x6C, 0x6C], Bytes::from_static(b"hell"); "hell")]
    #[test_case(&[0xCD, 0x00, 0x04, 0x6F, 0x2C, 0x20, 0x72], Bytes::from_static(b"o, r"); "o, r")]
    #[test_case(&[0xCE, 0x00, 0x00, 0x00, 0x04, 0x75, 0x73, 0x74, 0x21], Bytes::from_static(b"ust!"); "ust!")]
    fn bytes_parse(input: &'static [u8], expected: Bytes) {
        let bytes = Bytes::from_static(input);
        assert_eq!(from_bytes::<Bytes>(bytes).unwrap(), expected);
    }

    #[test_case(&[0xCC, 0x00], Vec::new(); "empty bytes")]
    #[test_case(&[0xCC, 0x04, 0x68, 0x65, 0x6C, 0x6C], Vec::from(b"hell".as_slice()); "hell")]
    #[test_case(&[0xCD, 0x00, 0x04, 0x6F, 0x2C, 0x20, 0x72], Vec::from(b"o, r".as_slice()); "o, r")]
    #[test_case(&[0xCE, 0x00, 0x00, 0x00, 0x04, 0x75, 0x73, 0x74, 0x21], Vec::from(b"ust!".as_slice()); "ust!")]
    fn bytes_vec(input: &'static [u8], expected: Vec<u8>) {
        let bytes = Bytes::from_static(input);
        assert_eq!(from_bytes::<Vec<u8>>(bytes).unwrap(), expected);
    }

    #[test_case(&[0x80], ""; "empty string")]
    #[test_case(&[0x81, 0x41], "A"; "A")]
    #[test_case(&[0xD0, 0x1A, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A], "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; "alphabet")]
    #[test_case(&[0xD0, 0x12, 0x47, 0x72, 0xC3, 0xB6, 0xC3, 0x9F, 0x65, 0x6E, 0x6D, 0x61, 0xC3, 0x9F, 0x73, 0x74, 0xC3, 0xA4, 0x62, 0x65],"Größenmaßstäbe"; "umlauts")]
    #[test_case(&[0xD0, 0x19, 0xF0, 0x9F, 0x91, 0xA9, 0xE2, 0x80, 0x8D, 0xF0, 0x9F, 0x91, 0xA9, 0xE2, 0x80, 0x8D, 0xF0, 0x9F, 0x91, 0xA7, 0xE2, 0x80, 0x8D, 0xF0, 0x9F, 0x91, 0xA7], "👩‍👩‍👧‍👧"; "emojis")]
    fn string(input: &'static [u8], expected: &str) {
        roundtrip(input, expected.to_owned())
    }

    #[test_case(&[0x81, 0x41], "A"; "tiny")]
    #[test_case(&[0xD0, 0x01, 0x41], "A"; "string_8")]
    #[test_case(&[0xD1, 0x00, 0x01, 0x41], "A"; "string_16")]
    #[test_case(&[0xD2, 0x00, 0x00, 0x00, 0x01, 0x41], "A"; "string_32")]
    fn string_parse(input: &'static [u8], expected: &str) {
        let bytes = Bytes::copy_from_slice(input);
        assert_eq!(from_bytes::<String>(bytes).unwrap(), expected);
    }

    #[test_case(&[0x90], Vec::new(); "empty list")]
    #[test_case(&[0x93, 0x01, 0x02, 0x03], vec![1, 2, 3]; "list_2")]
    #[test_case(&[0xD4, 0x28, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28], (1..=40).collect(); "list_3")]
    fn list(input: &'static [u8], expected: Vec<i64>) {
        roundtrip(input, expected)
    }

    #[test]
    fn tuple1() {
        let input = bolt().tiny_list(1).tiny_int(42).build();
        roundtrip(&input, (42,))
    }

    #[test]
    fn tuple2() {
        let input = bolt().tiny_list(2).tiny_int(42).tiny_string("1337").build();
        roundtrip(&input, (42, "1337".to_owned()))
    }

    #[test_case(&[0xA0], BTreeMap::new(); "empty")]
    #[test_case(&[0xA1, 0x83, 0x6F, 0x6E, 0x65, 0x2A], [("one".to_owned(), 42)].into_iter().collect(); "tiny")]
    #[test_case(&[0xD8, 0x1A, 0x81, 0x41, 0x01, 0x81, 0x42, 0x02, 0x81, 0x43, 0x03, 0x81, 0x44, 0x04, 0x81, 0x45, 0x05, 0x81, 0x46, 0x06, 0x81, 0x47, 0x07, 0x81, 0x48, 0x08, 0x81, 0x49, 0x09, 0x81, 0x4A, 0x0A, 0x81, 0x4B, 0x0B, 0x81, 0x4C, 0x0C, 0x81, 0x4D, 0x0D, 0x81, 0x4E, 0x0E, 0x81, 0x4F, 0x0F, 0x81, 0x50, 0x10, 0x81, 0x51, 0x11, 0x81, 0x52, 0x12, 0x81, 0x53, 0x13, 0x81, 0x54, 0x14, 0x81, 0x55, 0x15, 0x81, 0x56, 0x16, 0x81, 0x57, 0x17, 0x81, 0x58, 0x18, 0x81, 0x59, 0x19, 0x81, 0x5A, 0x1A], ('A'..='Z').map(|c| (c.to_string(), ((c as u32) - ('@' as u32)) as i64)).collect(); "map_8")]
    fn dictionary(input: &'static [u8], expected: BTreeMap<String, i64>) {
        roundtrip(input, expected)
    }

    #[test_case(&[0xA0], Vec::new(); "empty")]
    #[test_case(&[0xA1, 0x83, 0x6F, 0x6E, 0x65, 0x2A], [("one".to_owned(), 42)].into_iter().collect(); "tiny")]
    #[test_case(&[0xD8, 0x1A, 0x81, 0x41, 0x01, 0x81, 0x42, 0x02, 0x81, 0x43, 0x03, 0x81, 0x44, 0x04, 0x81, 0x45, 0x05, 0x81, 0x46, 0x06, 0x81, 0x47, 0x07, 0x81, 0x48, 0x08, 0x81, 0x49, 0x09, 0x81, 0x4A, 0x0A, 0x81, 0x4B, 0x0B, 0x81, 0x4C, 0x0C, 0x81, 0x4D, 0x0D, 0x81, 0x4E, 0x0E, 0x81, 0x4F, 0x0F, 0x81, 0x50, 0x10, 0x81, 0x51, 0x11, 0x81, 0x52, 0x12, 0x81, 0x53, 0x13, 0x81, 0x54, 0x14, 0x81, 0x55, 0x15, 0x81, 0x56, 0x16, 0x81, 0x57, 0x17, 0x81, 0x58, 0x18, 0x81, 0x59, 0x19, 0x81, 0x5A, 0x1A], ('A'..='Z').map(|c| (c.to_string(), ((c as u32) - ('@' as u32)) as i64)).collect(); "map_8")]
    fn dictionary_vec(input: &'static [u8], expected: Vec<(String, i64)>) {
        let bytes = Bytes::copy_from_slice(input);
        assert_eq!(from_bytes::<Vec<(String, i64)>>(bytes).unwrap(), expected);
    }

    #[test]
    fn dictionary_duplicates() {
        let bytes = Bytes::from_static(&[
            0xA3, 0x85, 0x6B, 0x65, 0x79, 0x5F, 0x31, 0x01, 0x85, 0x6B, 0x65, 0x79, 0x5F, 0x32,
            0x02, 0x85, 0x6B, 0x65, 0x79, 0x5F, 0x31, 0x03,
        ]);

        let expected = [("key_1".to_owned(), 3), ("key_2".to_owned(), 2)]
            .into_iter()
            .collect();

        assert_eq!(
            from_bytes::<BTreeMap<String, i64>>(bytes.clone()).unwrap(),
            expected
        );

        let expected = vec![
            ("key_1".to_owned(), 1),
            ("key_2".to_owned(), 2),
            ("key_1".to_owned(), 3),
        ];

        assert_eq!(from_bytes::<Vec<(String, i64)>>(bytes).unwrap(), expected);
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
    enum Structure {
        Unit,
        Int(i64),
        Pair(bool, String),
        List(Vec<i64>),
    }

    #[test_case(&[0xB0, 0x00], Structure::Unit; "unit")]
    #[test_case(&[0xB1, 0x01, 0x2A], Structure::Int(42); "int")]
    #[test_case(&[0xB2, 0x02, 0xC2, 0x81, 0x42], Structure::Pair(false, "B".to_string()); "pair")]
    #[test_case(&[0xB1, 0x03, 0x91, 0x2A], Structure::List(vec![42]); "list")]
    fn structure(input: &'static [u8], expected: Structure) {
        roundtrip(input, expected)
    }

    fn roundtrip<T: DeserializeOwned + Serialize + PartialEq + Debug>(input: &[u8], expected: T) {
        let bytes = Bytes::copy_from_slice(input);
        assert_eq!(from_bytes::<T>(bytes).unwrap(), expected);

        let actual = to_bytes(&expected).unwrap();
        let expected = Bytes::copy_from_slice(input);
        assert_eq!(actual, expected);
    }
}
