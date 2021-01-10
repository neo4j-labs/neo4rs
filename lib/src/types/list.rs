use crate::errors::*;
use crate::types::*;
use crate::version::Version;
use bytes::*;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

pub const TINY: u8 = 0x90;
pub const SMALL: u8 = 0xD4;
pub const MEDIUM: u8 = 0xD5;
pub const LARGE: u8 = 0xD6;

#[derive(Debug, PartialEq, Clone)]
pub struct BoltList {
    pub value: Vec<BoltType>,
}

impl Default for BoltList {
    fn default() -> Self {
        Self::new()
    }
}

impl BoltList {
    pub fn new() -> Self {
        BoltList { value: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        BoltList {
            value: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn push(&mut self, value: BoltType) {
        self.value.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&BoltType> {
        self.value.get(index)
    }

    pub fn can_parse(_: Version, input: Rc<RefCell<Bytes>>) -> bool {
        let marker = input.borrow()[0];
        (TINY..=(TINY | 0x0F)).contains(&marker)
            || marker == SMALL
            || marker == MEDIUM
            || marker == LARGE
    }

    pub fn iter(&self) -> impl Iterator<Item = &BoltType> {
        self.value.iter()
    }
}

impl IntoIterator for BoltList {
    type Item = BoltType;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}

impl Into<Vec<String>> for BoltList {
    fn into(self) -> Vec<String> {
        self.value.into_iter().map(|x| x.to_string()).collect()
    }
}

impl From<Vec<BoltType>> for BoltList {
    fn from(xs: Vec<BoltType>) -> BoltList {
        let mut list = BoltList::with_capacity(xs.len());
        for x in xs.into_iter() {
            list.push(x);
        }
        list
    }
}

impl BoltList {
    pub fn into_bytes(self, version: Version) -> Result<Bytes> {
        let mut values = BytesMut::new();
        let length = self.value.len();

        for elem in self.value {
            values.put(elem.into_bytes(version)?);
        }

        let mut bytes =
            BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<u32>() + values.len());

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
            65_536..=2_147_483_648 => {
                bytes.put_u8(LARGE);
                bytes.put_u32(length as u32);
            }
            _ => return Err(Error::ListTooLong),
        }

        bytes.put(values);
        Ok(bytes.freeze())
    }

    pub fn parse(version: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltList> {
        let marker = input.borrow_mut().get_u8();
        let size = match marker {
            0x90..=0x9F => 0x0F & marker as usize,
            SMALL => input.borrow_mut().get_u8() as usize,
            MEDIUM => input.borrow_mut().get_u16() as usize,
            LARGE => input.borrow_mut().get_u32() as usize,
            _ => {
                return Err(Error::InvalidTypeMarker(format!(
                    "invalid list marker {}",
                    marker
                )))
            }
        };

        let mut list = BoltList::with_capacity(size);
        for _ in 0..size {
            list.push(BoltType::parse(version, input.clone())?);
        }

        Ok(list)
    }
}

impl Into<Vec<BoltType>> for BoltList {
    fn into(self) -> Vec<BoltType> {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_empty_list() {
        let list = BoltList::new();

        let b: Bytes = list.into_bytes(Version::V4_1).unwrap();

        assert_eq!(&b[..], Bytes::from_static(&[TINY]));
    }

    #[test]
    fn should_serialize_list() {
        let mut list = BoltList::new();
        list.push("a".into());
        list.push(1.into());

        let b: Bytes = list.into_bytes(Version::V4_1).unwrap();

        assert_eq!(&b[..], Bytes::from_static(&[0x92, 0x81, 0x61, 0x01]));
    }

    #[test]
    fn should_deserialize_list() {
        let b = Rc::new(RefCell::new(Bytes::from_static(&[0x92, 0x81, 0x61, 0x01])));

        let bolt_list: BoltList = BoltList::parse(Version::V4_1, b).unwrap();

        assert_eq!(bolt_list.len(), 2);
        match bolt_list.get(0).unwrap() {
            BoltType::String(s) => assert_eq!(s.value, "a"),
            _ => unreachable!("error deserialiisation of string in list"),
        }

        match bolt_list.get(1).unwrap() {
            BoltType::Integer(s) => assert_eq!(s.value, 1),
            _ => unreachable!("error deserialiisation integer in list"),
        }
    }
}
