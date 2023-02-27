use crate::errors::*;
use crate::version::Version;
use bytes::*;
use std::cell::RefCell;
use std::rc::Rc;

pub const FALSE: u8 = 0xC2;
pub const TRUE: u8 = 0xC3;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoltBoolean {
    pub value: bool,
}

impl BoltBoolean {
    pub fn new(value: bool) -> BoltBoolean {
        BoltBoolean { value }
    }

    pub fn can_parse(_: Version, input: Rc<RefCell<Bytes>>) -> bool {
        let input = input.borrow()[0];
        input == TRUE || input == FALSE
    }
}

impl BoltBoolean {
    pub fn into_bytes(self, _: Version) -> Result<Bytes> {
        if self.value {
            Ok(Bytes::copy_from_slice(&[TRUE]))
        } else {
            Ok(Bytes::copy_from_slice(&[FALSE]))
        }
    }

    pub fn parse(_: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltBoolean> {
        let value = input.borrow_mut().get_u8();
        match value {
            TRUE => Ok(BoltBoolean::new(true)),
            FALSE => Ok(BoltBoolean::new(false)),
            _ => Err(Error::InvalidTypeMarker("invalid boolean marker".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_boolean() {
        let bolt_boolean = BoltBoolean::new(true);
        let b: Bytes = bolt_boolean.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[0xC3]);

        let bolt_boolean = BoltBoolean::new(false);
        let b: Bytes = bolt_boolean.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[0xC2]);
    }

    #[test]
    fn should_deserialize_boolean() {
        let b = Rc::new(RefCell::new(Bytes::copy_from_slice(&[TRUE])));
        let bolt_boolean: BoltBoolean = BoltBoolean::parse(Version::V4_1, b).unwrap();
        assert_eq!(bolt_boolean.value, true);

        let b = Rc::new(RefCell::new(Bytes::copy_from_slice(&[FALSE])));
        let bolt_boolean: BoltBoolean = BoltBoolean::parse(Version::V4_1, b).unwrap();
        assert_eq!(bolt_boolean.value, false);
    }
}
