use crate::errors::*;
use crate::version::Version;
use bytes::*;
use std::cell::RefCell;
use std::convert::From;
use std::fmt::Display;
use std::mem;
use std::rc::Rc;

pub const TINY: u8 = 0x80;
pub const SMALL: u8 = 0xD0;
pub const MEDIUM: u8 = 0xD1;
pub const LARGE: u8 = 0xD2;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BoltString {
    pub value: String,
}

impl BoltString {
    pub fn new(value: &str) -> Self {
        BoltString {
            value: value.to_string(),
        }
    }

    pub fn can_parse(_: Version, input: Rc<RefCell<Bytes>>) -> bool {
        let marker = input.borrow()[0];
        (TINY..=(TINY | 0x0F)).contains(&marker)
            || marker == SMALL
            || marker == MEDIUM
            || marker == LARGE
    }
}

impl Display for BoltString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<&str> for BoltString {
    fn from(v: &str) -> Self {
        BoltString::new(v)
    }
}

impl From<String> for BoltString {
    fn from(v: String) -> Self {
        BoltString::new(&v)
    }
}

impl Into<String> for BoltString {
    fn into(self) -> String {
        self.value
    }
}

impl BoltString {
    pub fn into_bytes(self, _: Version) -> Result<Bytes> {
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() + mem::size_of::<u32>() + self.value.len(),
        );
        match self.value.len() {
            0..=15 => bytes.put_u8(TINY | self.value.len() as u8),
            16..=255 => {
                bytes.put_u8(SMALL);
                bytes.put_u8(self.value.len() as u8);
            }
            256..=65_535 => {
                bytes.put_u8(MEDIUM);
                bytes.put_u16(self.value.len() as u16);
            }
            65_536..=4_294_967_295 => {
                bytes.put_u8(LARGE);
                bytes.put_u32(self.value.len() as u32);
            }
            _ => return Err(Error::StringTooLong),
        };
        bytes.put_slice(self.value.as_bytes());
        Ok(bytes.freeze())
    }

    pub fn parse(_: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltString> {
        let mut input = input.borrow_mut();
        let marker = input.get_u8();
        let length = match marker {
            0x80..=0x8F => 0x0F & marker as usize,
            SMALL => input.get_u8() as usize,
            MEDIUM => input.get_u16() as usize,
            LARGE => input.get_u32() as usize,
            _ => {
                return Err(Error::InvalidTypeMarker(format!(
                    "invalid string marker {}",
                    marker
                )))
            }
        };
        let byte_array = input.split_to(length).to_vec();
        let string_value = std::string::String::from_utf8(byte_array)
            .map_err(|e| Error::DeserializationError(e.to_string()))?;
        Ok(string_value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_empty_string() {
        let s = BoltString::new("");
        let b: Bytes = s.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], Bytes::from_static(&[TINY]));
    }

    #[test]
    fn should_deserialize_empty_string() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[TINY])));
        let s: BoltString = BoltString::parse(Version::V4_1, input).unwrap();
        assert_eq!(s, "".into());
    }

    #[test]
    fn should_serialize_tiny_string() {
        let s = BoltString::new("a");
        let b: Bytes = s.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], Bytes::from_static(&[0x81, 0x61]));
    }

    #[test]
    fn should_deserialize_tiny_string() {
        let serialized_bytes = Rc::new(RefCell::new(Bytes::from_static(&[0x81, 0x61])));
        let result: BoltString = BoltString::parse(Version::V4_1, serialized_bytes).unwrap();
        assert_eq!(result, "a".into());
    }

    #[test]
    fn should_serialize_small_string() {
        let s = BoltString::new(&"a".repeat(16));

        let mut b: Bytes = s.into_bytes(Version::V4_1).unwrap();

        assert_eq!(b.get_u8(), SMALL);
        assert_eq!(b.get_u8(), 0x10);
        assert_eq!(b.len(), 0x10);
        for value in b {
            assert_eq!(value, 0x61);
        }
    }

    #[test]
    fn should_deserialize_small_string() {
        let serialized_bytes = Rc::new(RefCell::new(Bytes::from_static(&[SMALL, 0x01, 0x61])));
        let result: BoltString = BoltString::parse(Version::V4_1, serialized_bytes).unwrap();
        assert_eq!(result, "a".into());
    }

    #[test]
    fn should_serialize_medium_string() {
        let s = BoltString::new(&"a".repeat(256));

        let mut b: Bytes = s.into_bytes(Version::V4_1).unwrap();

        assert_eq!(b.get_u8(), MEDIUM);
        assert_eq!(b.get_u16(), 0x100);
        assert_eq!(b.len(), 0x100);
        for value in b {
            assert_eq!(value, 0x61);
        }
    }

    #[test]
    fn should_deserialize_medium_string() {
        let serialized_bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            MEDIUM, 0x00, 0x01, 0x61,
        ])));
        let result: BoltString = BoltString::parse(Version::V4_1, serialized_bytes).unwrap();
        assert_eq!(result, "a".into());
    }

    #[test]
    fn should_serialize_large_string() {
        let s = BoltString::new(&"a".repeat(65_536));

        let mut b: Bytes = s.into_bytes(Version::V4_1).unwrap();

        assert_eq!(b.get_u8(), LARGE);
        assert_eq!(b.get_u32(), 0x10000);
        assert_eq!(b.len(), 0x10000);
        for value in b {
            assert_eq!(value, 0x61);
        }
    }

    #[test]
    fn should_deserialize_large_string() {
        let serialized_bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            LARGE, 0x00, 0x00, 0x00, 0x01, 0x61,
        ])));
        let result: BoltString = BoltString::parse(Version::V4_1, serialized_bytes).unwrap();
        assert_eq!(result, "a".into());
    }
}
