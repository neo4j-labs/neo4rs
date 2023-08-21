use crate::errors::*;
use crate::version::Version;
use bytes::*;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

pub const MARKER: u8 = 0xC1;

#[derive(Debug, PartialEq, Clone)]
pub struct BoltFloat {
    pub value: f64,
}

impl BoltFloat {
    pub fn new(value: f64) -> BoltFloat {
        BoltFloat { value }
    }

    pub fn can_parse(_: Version, input: Rc<RefCell<Bytes>>) -> bool {
        input.borrow()[0] == MARKER
    }
}

impl BoltFloat {
    pub fn parse(_: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltFloat> {
        let mut input = input.borrow_mut();
        let _marker = input.get_u8();
        let value = input.get_f64();
        Ok(BoltFloat::new(value))
    }

    pub fn into_bytes(self, _: Version) -> Result<Bytes> {
        let mut bytes = BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<f64>());
        bytes.put_u8(MARKER);
        bytes.put_f64(self.value);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_float() {
        let b: Bytes = BoltFloat::new(1.23).into_bytes(Version::V4_1).unwrap();
        assert_eq!(
            &b[..],
            &[0xC1, 0x3F, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE]
        );

        let b: Bytes = BoltFloat::new(-1.23).into_bytes(Version::V4_1).unwrap();
        assert_eq!(
            &b[..],
            &[0xC1, 0xBF, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE,]
        );
    }

    #[test]
    fn should_deserialize_float() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xC1, 0x3F, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE,
        ])));
        let bolt_float: BoltFloat = BoltFloat::parse(Version::V4_1, input).unwrap();
        assert_eq!(bolt_float.value, 1.23);

        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xC1, 0xBF, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE,
        ])));
        let bolt_float: BoltFloat = BoltFloat::parse(Version::V4_1, input).unwrap();
        assert_eq!(bolt_float.value, -1.23);
    }
}
