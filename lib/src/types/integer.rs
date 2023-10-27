use crate::{
    errors::{Error, Result},
    types::BoltWireFormat,
    version::Version,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::mem;
use std::ops::{Add, Sub};

pub const INT_8: u8 = 0xC8;
pub const INT_16: u8 = 0xC9;
pub const INT_32: u8 = 0xCA;
pub const INT_64: u8 = 0xCB;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoltInteger {
    pub value: i64,
}

impl BoltInteger {
    pub fn new(value: i64) -> BoltInteger {
        BoltInteger { value }
    }
}

impl Add for BoltInteger {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.value + rhs.value).into()
    }
}

impl Sub for BoltInteger {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.value - rhs.value).into()
    }
}

impl From<i64> for BoltInteger {
    fn from(value: i64) -> Self {
        BoltInteger::new(value)
    }
}

impl From<BoltInteger> for i64 {
    fn from(value: BoltInteger) -> Self {
        value.value
    }
}

impl From<i32> for BoltInteger {
    fn from(value: i32) -> Self {
        BoltInteger::new(value as i64)
    }
}

impl BoltWireFormat for BoltInteger {
    fn can_parse(_version: Version, input: &[u8]) -> bool {
        let marker = input[0];
        (-16..=127).contains(&(marker as i8))
            || marker == INT_8
            || marker == INT_16
            || marker == INT_32
            || marker == INT_64
    }

    fn parse(_version: Version, input: &mut Bytes) -> Result<Self> {
        let value: i64 = match input.get_u8() {
            marker if (-16..=127).contains(&(marker as i8)) => marker as i64,
            INT_8 => input.get_i8() as i64,
            INT_16 => input.get_i16() as i64,
            INT_32 => input.get_i32() as i64,
            INT_64 => input.get_i64(),
            _ => return Err(Error::InvalidTypeMarker("invalid integer marker".into())),
        };

        Ok(BoltInteger::new(value))
    }

    fn write_into(&self, _version: Version, bytes: &mut BytesMut) -> Result<()> {
        match self.value {
            -16..=127 => {
                bytes.reserve(mem::size_of::<u8>());
                bytes.put_u8(self.value as u8)
            }
            -128..=-17 => {
                bytes.reserve(mem::size_of::<u8>() + mem::size_of::<i8>());
                bytes.put_u8(INT_8);
                bytes.put_i8(self.value as i8);
            }
            128..=32_767 | -32_768..=-129 => {
                bytes.reserve(mem::size_of::<u8>() + mem::size_of::<i16>());
                bytes.put_u8(INT_16);
                bytes.put_i16(self.value as i16);
            }
            32_768..=2_147_483_647 | -2_147_483_648..=-32_769 => {
                bytes.reserve(mem::size_of::<u8>() + mem::size_of::<i32>());
                bytes.put_u8(INT_32);
                bytes.put_i32(self.value as i32);
            }
            2_147_483_648..=9_223_372_036_854_775_807
            | -9_223_372_036_854_775_808..=-2_147_483_649 => {
                bytes.reserve(mem::size_of::<u8>() + mem::size_of::<i64>());
                bytes.put_u8(INT_64);
                bytes.put_i64(self.value);
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_integer() {
        let bolt_int = BoltInteger::new(42);
        let b: Bytes = bolt_int.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[0x2A]);

        let bolt_int = BoltInteger::new(-127);
        let b: Bytes = bolt_int.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[INT_8, 0x81]);

        let bolt_int = BoltInteger::new(129);
        let b: Bytes = bolt_int.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[INT_16, 0x00, 0x81]);

        let bolt_int = BoltInteger::new(32_768);
        let b: Bytes = bolt_int.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[INT_32, 0x00, 0x00, 0x80, 0x00]);

        let bolt_int = BoltInteger::new(2_147_483_648);
        let b: Bytes = bolt_int.into_bytes(Version::V4_1).unwrap();
        assert_eq!(
            &b[..],
            &[INT_64, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn should_deserialize_integer() {
        let mut b = Bytes::from_static(&[0x2A]);
        let bolt_int: BoltInteger = BoltInteger::parse(Version::V4_1, &mut b).unwrap();
        assert_eq!(bolt_int.value, 42);

        let mut b = Bytes::from_static(&[INT_8, 0x81]);
        let bolt_int: BoltInteger = BoltInteger::parse(Version::V4_1, &mut b).unwrap();
        assert_eq!(bolt_int.value, -127);

        let mut b = Bytes::from_static(&[INT_16, 0x00, 0x81]);
        let bolt_int: BoltInteger = BoltInteger::parse(Version::V4_1, &mut b).unwrap();
        assert_eq!(bolt_int.value, 129);

        let mut b = Bytes::from_static(&[INT_32, 0x00, 0x00, 0x80, 0x00]);
        let bolt_int: BoltInteger = BoltInteger::parse(Version::V4_1, &mut b).unwrap();
        assert_eq!(bolt_int.value, 32_768);

        let mut b = Bytes::from_static(&[INT_64, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00]);
        let bolt_int: BoltInteger = BoltInteger::parse(Version::V4_1, &mut b).unwrap();
        assert_eq!(bolt_int.value, 2_147_483_648);
    }
}
