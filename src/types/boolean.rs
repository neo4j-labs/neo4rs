use crate::errors::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

pub const FALSE: u8 = 0xC2;
pub const TRUE: u8 = 0xC3;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BoltBoolean {
    pub value: bool,
}

impl BoltBoolean {
    pub fn new(value: bool) -> BoltBoolean {
        BoltBoolean { value }
    }

    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
        let input = input.borrow()[0];
        input == TRUE || input == FALSE
    }
}

impl TryInto<Bytes> for BoltBoolean {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        if self.value {
            Ok(Bytes::copy_from_slice(&[TRUE]))
        } else {
            Ok(Bytes::copy_from_slice(&[FALSE]))
        }
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltBoolean {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltBoolean> {
        let value = input.borrow_mut().get_u8();
        match value {
            TRUE => Ok(BoltBoolean::new(true)),
            FALSE => Ok(BoltBoolean::new(false)),
            _ => return Err(Error::InvalidTypeMarker("invalid boolean marker".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_boolean() {
        let bolt_boolean = BoltBoolean::new(true);
        let b: Bytes = bolt_boolean.try_into().unwrap();
        assert_eq!(b.bytes(), &[0xC3]);

        let bolt_boolean = BoltBoolean::new(false);
        let b: Bytes = bolt_boolean.try_into().unwrap();
        assert_eq!(b.bytes(), &[0xC2]);
    }

    #[test]
    fn should_deserialize_boolean() {
        let b = Rc::new(RefCell::new(Bytes::copy_from_slice(&[TRUE])));
        let bolt_boolean: BoltBoolean = b.try_into().unwrap();
        assert_eq!(bolt_boolean.value, true);

        let b = Rc::new(RefCell::new(Bytes::copy_from_slice(&[FALSE])));
        let bolt_boolean: BoltBoolean = b.try_into().unwrap();
        assert_eq!(bolt_boolean.value, false);
    }
}
