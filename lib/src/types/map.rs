use crate::errors::*;
use crate::types::*;
use crate::version::Version;
use bytes::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::iter::FromIterator;

use std::mem;
use std::rc::Rc;

pub const TINY: u8 = 0xA0;
pub const SMALL: u8 = 0xD8;
pub const MEDIUM: u8 = 0xD9;
pub const LARGE: u8 = 0xDA;

#[derive(Debug, PartialEq, Clone)]
pub struct BoltMap {
    pub value: HashMap<BoltString, BoltType>,
}

impl Default for BoltMap {
    fn default() -> Self {
        BoltMap {
            value: HashMap::new(),
        }
    }
}

impl BoltMap {
    pub fn with_capacity(capacity: usize) -> Self {
        BoltMap {
            value: HashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn put(&mut self, key: BoltString, value: BoltType) {
        self.value.insert(key, value);
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        match self.value.get(&BoltString::new(key)) {
            Some(bolt_type) => {
                if let Ok(value) = TryInto::<T>::try_into(bolt_type.clone()) {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
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

impl BoltMap {
    pub fn into_bytes(self, version: Version) -> Result<Bytes> {
        let mut key_value_bytes = BytesMut::new();
        let length = self.value.len();
        for (key, value) in self.value {
            let key_bytes: Bytes = key.into_bytes(version)?;
            let value_bytes: Bytes = value.into_bytes(version)?;
            key_value_bytes.put(key_bytes);
            key_value_bytes.put(value_bytes);
        }

        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() + mem::size_of::<u32>() + key_value_bytes.len(),
        );

        match length {
            0..=15 => bytes.put_u8(TINY | length as u8),
            16..=255 => {
                bytes.put_u8(SMALL);
                bytes.put_u8(length as u8);
            }
            256..=65_535 => {
                bytes.put_u8(MEDIUM);
                bytes.put_u16(length as u16);
            }
            65_536..=4_294_967_295 => {
                bytes.put_u8(LARGE);
                bytes.put_u32(length as u32);
            }
            _ => return Err(Error::MapTooBig),
        }

        bytes.put(key_value_bytes);
        Ok(bytes.freeze())
    }

    pub fn parse(version: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltMap> {
        let marker = input.borrow_mut().get_u8();
        let size = match marker {
            0xA0..=0xAF => 0x0F & marker as usize,
            SMALL => input.borrow_mut().get_u8() as usize,
            MEDIUM => input.borrow_mut().get_u16() as usize,
            LARGE => input.borrow_mut().get_u32() as usize,
            _ => {
                return Err(Error::InvalidTypeMarker(format!(
                    "invalid map marker {}",
                    marker
                )))
            }
        };

        let mut map = BoltMap::default();
        for _ in 0..size {
            let key: BoltString = BoltString::parse(version, input.clone())?;
            let value: BoltType = BoltType::parse(version, input.clone())?;
            map.put(key, value);
        }

        Ok(map)
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
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xA1, 0x81, 0x61, 0x81, 0x62,
        ])));

        let map: BoltMap = BoltMap::parse(Version::V4_1, input).unwrap();

        assert_eq!(map.value.len(), 1);
    }

    #[test]
    fn should_deserialize_small_map() {
        let mut map = BoltMap::default();
        for i in 0..=16 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let bytes: Rc<RefCell<Bytes>> =
            Rc::new(RefCell::new(map.clone().into_bytes(Version::V4_1).unwrap()));
        assert_eq!(bytes.borrow()[0], SMALL);
        let deserialized_map: BoltMap = BoltMap::parse(Version::V4_1, bytes).unwrap();
        assert_eq!(map, deserialized_map);
    }

    #[test]
    fn should_deserialize_medium_map() {
        let mut map = BoltMap::default();
        for i in 0..=256 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let bytes: Rc<RefCell<Bytes>> =
            Rc::new(RefCell::new(map.clone().into_bytes(Version::V4_1).unwrap()));
        assert_eq!(bytes.borrow()[0], MEDIUM);
        let deserialized_map: BoltMap = BoltMap::parse(Version::V4_1, bytes).unwrap();
        assert_eq!(map, deserialized_map);
    }

    #[test]
    fn should_deserialize_large_map() {
        let mut map = BoltMap::default();
        for i in 0..=65_536 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let bytes: Rc<RefCell<Bytes>> =
            Rc::new(RefCell::new(map.clone().into_bytes(Version::V4_1).unwrap()));
        assert_eq!(bytes.borrow()[0], LARGE);
        let deserialized_map: BoltMap = BoltMap::parse(Version::V4_1, bytes).unwrap();
        assert_eq!(map, deserialized_map);
    }
}
