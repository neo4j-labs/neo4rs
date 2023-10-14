use crate::{
    errors::{Error, Result},
    types::{serde::DeError, BoltString, BoltType, BoltWireFormat},
    version::Version,
};
use ::serde::Deserialize;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{collections::HashMap, iter::FromIterator, mem};

pub const TINY: u8 = 0xA0;
pub const SMALL: u8 = 0xD8;
pub const MEDIUM: u8 = 0xD9;
pub const LARGE: u8 = 0xDA;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct BoltMap {
    pub value: HashMap<BoltString, BoltType>,
}

impl BoltMap {
    pub fn new() -> Self {
        BoltMap {
            value: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        BoltMap {
            value: HashMap::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn put(&mut self, key: BoltString, value: BoltType) {
        self.value.insert(key, value);
    }

    pub fn get<'this, T>(&'this self, key: &str) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
        Self: Sized,
    {
        match self.value.get(key) {
            Some(v) => v.to(),
            None => Err(DeError::NoSuchProperty),
        }
    }
}

impl FromIterator<(BoltString, BoltType)> for BoltMap {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (BoltString, BoltType)>,
    {
        let mut bolt_map = BoltMap::default();
        for (s, t) in iter.into_iter() {
            bolt_map.put(s, t);
        }
        bolt_map
    }
}

impl BoltWireFormat for BoltMap {
    fn can_parse(_version: Version, input: &[u8]) -> bool {
        let marker = input[0];
        (TINY..=(TINY | 0x0F)).contains(&marker)
            || marker == SMALL
            || marker == MEDIUM
            || marker == LARGE
    }

    fn parse(version: Version, input: &mut Bytes) -> Result<Self> {
        let marker = input.get_u8();
        let size = match marker {
            0xA0..=0xAF => 0x0F & marker as usize,
            SMALL => input.get_u8() as usize,
            MEDIUM => input.get_u16() as usize,
            LARGE => input.get_u32() as usize,
            _ => {
                return Err(Error::InvalidTypeMarker(format!(
                    "invalid map marker {}",
                    marker
                )))
            }
        };

        let mut map = BoltMap::default();
        for _ in 0..size {
            let key = BoltString::parse(version, input)?;
            let value = BoltType::parse(version, input)?;
            map.put(key, value);
        }

        Ok(map)
    }

    fn write_into(&self, version: Version, bytes: &mut BytesMut) -> Result<()> {
        let length = self.value.len();
        match length {
            0..=15 => {
                bytes.reserve(1);
                bytes.put_u8(TINY | length as u8)
            }
            16..=255 => {
                bytes.reserve(2);
                bytes.put_u8(SMALL);
                bytes.put_u8(length as u8);
            }
            256..=65_535 => {
                bytes.reserve(1 + mem::size_of::<u16>());
                bytes.put_u8(MEDIUM);
                bytes.put_u16(length as u16);
            }
            65_536..=4_294_967_295 => {
                bytes.reserve(1 + mem::size_of::<u32>());
                bytes.put_u8(LARGE);
                bytes.put_u32(length as u32);
            }
            _ => return Err(Error::MapTooBig),
        }

        for (key, value) in &self.value {
            key.write_into(version, bytes)?;
            value.write_into(version, bytes)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_empty_map() {
        let map = BoltMap::default();

        let b: Bytes = map.into_bytes(Version::V4_1).unwrap();

        assert_eq!(&b[..], Bytes::from_static(&[TINY]));
    }

    #[test]
    fn should_serialize_map_of_strings() {
        let mut map = BoltMap::default();
        map.put("a".into(), "b".into());

        let b: Bytes = map.into_bytes(Version::V4_1).unwrap();

        assert_eq!(&b[..], Bytes::from_static(&[0xA1, 0x81, 0x61, 0x81, 0x62]));
    }

    #[test]
    fn should_deserialize_map_of_strings() {
        let mut input = Bytes::from_static(&[0xA1, 0x81, 0x61, 0x81, 0x62]);

        let map: BoltMap = BoltMap::parse(Version::V4_1, &mut input).unwrap();

        assert_eq!(map.value.len(), 1);
    }

    #[test]
    fn should_deserialize_small_map() {
        let mut map = BoltMap::default();
        for i in 0..=16 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let mut bytes = map.clone().into_bytes(Version::V4_1).unwrap();
        assert_eq!(bytes[0], SMALL);
        let deserialized_map: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();
        assert_eq!(map, deserialized_map);
    }

    #[test]
    fn should_deserialize_medium_map() {
        let mut map = BoltMap::default();
        for i in 0..=256 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let mut bytes = map.clone().into_bytes(Version::V4_1).unwrap();
        assert_eq!(bytes[0], MEDIUM);
        let deserialized_map: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();
        assert_eq!(map, deserialized_map);
    }

    #[test]
    fn should_deserialize_large_map() {
        let mut map = BoltMap::default();
        for i in 0..=65_536 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let mut bytes = map.clone().into_bytes(Version::V4_1).unwrap();
        assert_eq!(bytes[0], LARGE);
        let deserialized_map: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();
        assert_eq!(map, deserialized_map);
    }
}
