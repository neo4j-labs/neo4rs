use crate::error::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::rc::Rc;

pub const INT_8: u8 = 0xC8;
pub const INT_16: u8 = 0xC9;
pub const INT_32: u8 = 0xCA;
pub const INT_64: u8 = 0xCB;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BoltInteger {
    pub value: i64,
}

impl BoltInteger {
    pub fn new(value: i64) -> BoltInteger {
        BoltInteger { value }
    }
}

impl From<i64> for BoltInteger {
    fn from(v: i64) -> Self {
        BoltInteger::new(v)
    }
}

pub fn is_present(input: Rc<RefCell<Bytes>>) -> bool {
    let marker = input.borrow()[0];
    (-16..=127).contains(&(marker as i8))
        || marker == INT_8
        || marker == INT_16
        || marker == INT_32
        || marker == INT_64
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltInteger {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltInteger> {
        let mut input = input.borrow_mut();
        let value: i64 = match input.get_u8() {
            marker if (-16..=127).contains(&(marker as i8)) => marker as i64,
            INT_8 => input.get_i8() as i64,
            INT_16 => input.get_i16() as i64,
            INT_32 => input.get_i32() as i64,
            INT_64 => input.get_i64() as i64,
            _ => {
                return Err(Error::InvalidTypeMarker {
                    detail: "invalid integer marker".into(),
                })
            }
        };

        Ok(value.into())
    }
}
impl TryInto<Bytes> for BoltInteger {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        let mut bytes = BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<i64>());
        match self.value {
            -16..=127 => bytes.put_u8(self.value as u8),
            -128..=-17 => {
                bytes.put_u8(INT_8);
                bytes.put_i8(self.value as i8);
            }
            128..=32_767 | -32_768..=-129 => {
                bytes.put_u8(INT_16);
                bytes.put_i16(self.value as i16);
            }
            32_768..=2_147_483_647 | -2_147_483_648..=-32_769 => {
                bytes.put_u8(INT_32);
                bytes.put_i32(self.value as i32);
            }
            2_147_483_648..=9_223_372_036_854_775_807
            | -9_223_372_036_854_775_808..=-2_147_483_649 => {
                bytes.put_u8(INT_64);
                bytes.put_i64(self.value as i64);
            }
        }
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_integer() {
        let bolt_int = BoltInteger::new(42);
        let b: Bytes = bolt_int.try_into().unwrap();
        assert_eq!(b.bytes(), &[0x2A]);

        let bolt_int = BoltInteger::new(-127);
        let b: Bytes = bolt_int.try_into().unwrap();
        assert_eq!(b.bytes(), &[INT_8, 0x81]);

        let bolt_int = BoltInteger::new(129);
        let b: Bytes = bolt_int.try_into().unwrap();
        assert_eq!(b.bytes(), &[INT_16, 0x00, 0x81]);

        let bolt_int = BoltInteger::new(32_768);
        let b: Bytes = bolt_int.try_into().unwrap();
        assert_eq!(b.bytes(), &[INT_32, 0x00, 0x00, 0x80, 0x00]);

        let bolt_int = BoltInteger::new(2_147_483_648);
        let b: Bytes = bolt_int.try_into().unwrap();
        assert_eq!(
            b.bytes(),
            &[INT_64, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn should_deserialize_integer() {
        let b = Rc::new(RefCell::new(Bytes::from_static(&[0x2A])));
        let bolt_int: BoltInteger = b.try_into().unwrap();
        assert_eq!(bolt_int.value, 42);

        let b = Rc::new(RefCell::new(Bytes::from_static(&[INT_8, 0x81])));
        let bolt_int: BoltInteger = b.try_into().unwrap();
        assert_eq!(bolt_int.value, -127);

        let b = Rc::new(RefCell::new(Bytes::from_static(&[INT_16, 0x00, 0x81])));
        let bolt_int: BoltInteger = b.try_into().unwrap();
        assert_eq!(bolt_int.value, 129);

        let b = Rc::new(RefCell::new(Bytes::from_static(&[
            INT_32, 0x00, 0x00, 0x80, 0x00,
        ])));
        let bolt_int: BoltInteger = b.try_into().unwrap();
        assert_eq!(bolt_int.value, 32_768);

        let b = Rc::new(RefCell::new(Bytes::from_static(&[
            INT_64, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
        ])));
        let bolt_int: BoltInteger = b.try_into().unwrap();
        assert_eq!(bolt_int.value, 2_147_483_648);
    }
}
