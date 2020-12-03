use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

use std::mem;
use std::rc::Rc;

pub const TINY: u8 = 0xA0;
pub const SMALL: u8 = 0xD8;
pub const MEDIUM: u8 = 0xD9;
pub const LARGE: u8 = 0xDA;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoltMap {
    pub value: HashMap<BoltString, BoltType>,
}

impl BoltMap {
    pub fn new() -> Self {
        BoltMap {
            value: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn put(&mut self, key: BoltString, value: BoltType) {
        self.value.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<BoltType> {
        self.value.get(&key.into()).map(|v| v.clone())
    }

    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
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
        let mut bolt_map = BoltMap::new();
        for (s, t) in iter.into_iter() {
            bolt_map.put(s, t);
        }
        bolt_map
    }
}

impl TryInto<Bytes> for BoltMap {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let mut key_value_bytes = BytesMut::new();
        let length = self.value.len();
        for (key, value) in self.value {
            let key_bytes: Bytes = key.try_into()?;
            let value_bytes: Bytes = value.try_into()?;
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
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltMap {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltMap> {
        let marker = input.borrow_mut().get_u8();
        let size = match marker {
            0xA0..=0xAF => 0x0F & marker as usize,
            SMALL => input.borrow_mut().get_u8() as usize,
            MEDIUM => input.borrow_mut().get_u16() as usize,
            LARGE => input.borrow_mut().get_u32() as usize,
            _ => {
                return Err(Error::InvalidTypeMarker {
                    detail: format!("invalid marker {}", marker),
                })
            }
        };

        let mut map = BoltMap::new();
        for _ in 0..size {
            let key: BoltString = input.clone().try_into()?;
            let value: BoltType = input.clone().try_into()?;
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
        let map = BoltMap::new();

        let b: Bytes = map.try_into().unwrap();

        assert_eq!(b.bytes(), Bytes::from_static(&[TINY]));
    }

    #[test]
    fn should_serialize_map_of_strings() {
        let mut map = BoltMap::new();
        map.put("a".into(), "b".into());

        let b: Bytes = map.try_into().unwrap();

        assert_eq!(
            b.bytes(),
            Bytes::from_static(&[0xA1, 0x81, 0x61, 0x81, 0x62])
        );
    }

    #[test]
    fn should_deserialize_map_of_strings() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xA1, 0x81, 0x61, 0x81, 0x62,
        ])));

        let map: BoltMap = input.try_into().unwrap();

        assert_eq!(map.value.len(), 1);
    }

    #[test]
    fn should_deserialize_small_map() {
        let mut map = BoltMap::new();
        for i in 0..=16 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let bytes: Rc<RefCell<Bytes>> = Rc::new(RefCell::new(map.clone().try_into().unwrap()));
        assert_eq!(bytes.borrow()[0], SMALL);
        let deserialized_map: BoltMap = bytes.try_into().unwrap();
        assert_eq!(map, deserialized_map);
    }

    #[test]
    fn should_deserialize_medium_map() {
        let mut map = BoltMap::new();
        for i in 0..=256 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let bytes: Rc<RefCell<Bytes>> = Rc::new(RefCell::new(map.clone().try_into().unwrap()));
        assert_eq!(bytes.borrow()[0], MEDIUM);
        let deserialized_map: BoltMap = bytes.try_into().unwrap();
        assert_eq!(map, deserialized_map);
    }

    #[test]
    fn should_deserialize_large_map() {
        let mut map = BoltMap::new();
        for i in 0..=65_536 {
            map.put(i.to_string().into(), i.to_string().into());
        }

        let bytes: Rc<RefCell<Bytes>> = Rc::new(RefCell::new(map.clone().try_into().unwrap()));
        assert_eq!(bytes.borrow()[0], LARGE);
        let deserialized_map: BoltMap = bytes.try_into().unwrap();
        assert_eq!(map, deserialized_map);
    }
}
