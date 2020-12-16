use crate::errors::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
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

    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
        input.borrow()[0] == MARKER
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltFloat {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltFloat> {
        let mut input = input.borrow_mut();
        let _marker = input.get_u8();
        let value = input.get_f64();
        Ok(BoltFloat::new(value))
    }
}

impl TryInto<Bytes> for BoltFloat {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
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
        let bolt_float = BoltFloat::new(1.23);
        let b: Bytes = bolt_float.try_into().unwrap();
        assert_eq!(
            b.bytes(),
            &[0xC1, 0x3F, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE]
        );

        let bolt_folat = BoltFloat::new(-1.23);
        let b: Bytes = bolt_folat.try_into().unwrap();
        assert_eq!(
            b.bytes(),
            &[0xC1, 0xBF, 0xF3, 0xAE, 0x14, 0x7A, 0xE1, 0x47, 0xAE,]
        );
    }
}
