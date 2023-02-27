use crate::errors::*;
use crate::version::Version;
use bytes::*;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

pub const SMALL: u8 = 0xCC;
pub const MEDIUM: u8 = 0xCD;
pub const LARGE: u8 = 0xCE;

#[derive(Debug, PartialEq, Clone)]
pub struct BoltBytes {
    pub value: Bytes,
}

impl BoltBytes {
    pub fn new(value: Bytes) -> Self {
        BoltBytes { value }
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn can_parse(_: Version, input: Rc<RefCell<Bytes>>) -> bool {
        let marker = input.borrow()[0];
        [SMALL, MEDIUM, LARGE].contains(&marker)
    }
}

impl BoltBytes {
    pub fn into_bytes(self, _: Version) -> Result<Bytes> {
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() + mem::size_of::<u32>() + self.value.len(),
        );

        match &self.value {
            value if value.len() <= 255 => {
                bytes.put_u8(SMALL);
                bytes.put_u8(value.len() as u8);
            }
            value if value.len() > 255 && value.len() <= 65_535 => {
                bytes.put_u8(MEDIUM);
                bytes.put_u16(value.len() as u16);
            }
            value if value.len() > 65_535 && value.len() <= 2_147_483_648 => {
                bytes.put_u8(LARGE);
                bytes.put_u32(value.len() as u32);
            }
            _ => return Err(Error::BytesTooBig),
        }
        bytes.put(self.value);
        Ok(bytes.freeze())
    }

    pub fn parse(_: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltBytes> {
        let marker = input.borrow_mut().get_u8();
        let size = match marker {
            SMALL => input.borrow_mut().get_u8() as usize,
            MEDIUM => input.borrow_mut().get_u16() as usize,
            LARGE => input.borrow_mut().get_u32() as usize,
            _ => {
                return Err(Error::InvalidTypeMarker(format!(
                    "invalid bytes marker {}",
                    marker
                )))
            }
        };

        Ok(BoltBytes::new(input.borrow_mut().split_to(size)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_small_bytes() {
        let bolt_bytes = BoltBytes::new(Bytes::from_static("hello".as_bytes()));

        let serialized: Bytes = bolt_bytes.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            &serialized[..],
            Bytes::from_static(&[SMALL, 0x05, b'h', b'e', b'l', b'l', b'o'])
        );

        let deserialized: BoltBytes =
            BoltBytes::parse(Version::V4_1, Rc::new(RefCell::new(serialized))).unwrap();

        assert_eq!(
            String::from_utf8(deserialized.value.to_vec()).unwrap(),
            "hello".to_owned()
        );
    }

    #[test]
    fn should_serialize_medium_bytes() {
        let raw_bytes = Bytes::copy_from_slice(&vec![0; 256]);
        let bolt_bytes = BoltBytes::new(raw_bytes.clone());
        let serialized: Bytes = bolt_bytes.into_bytes(Version::V4_1).unwrap();

        assert_eq!(serialized[0], MEDIUM);
        assert_eq!(u16::from_be_bytes([serialized[1], serialized[2]]), 256);

        let deserialized: BoltBytes =
            BoltBytes::parse(Version::V4_1, Rc::new(RefCell::new(serialized))).unwrap();
        assert_eq!(deserialized.value, raw_bytes);
    }

    #[test]
    fn should_serialize_large_bytes() {
        let raw_bytes = Bytes::copy_from_slice(&vec![0; 65_537]);
        let bolt_bytes = BoltBytes::new(raw_bytes.clone());
        let serialized: Bytes = bolt_bytes.into_bytes(Version::V4_1).unwrap();

        assert_eq!(serialized[0], LARGE);
        assert_eq!(
            u32::from_be_bytes([serialized[1], serialized[2], serialized[3], serialized[4]]),
            65_537
        );

        let deserialized: BoltBytes =
            BoltBytes::parse(Version::V4_1, Rc::new(RefCell::new(serialized))).unwrap();
        assert_eq!(deserialized.value, raw_bytes);
    }
}
