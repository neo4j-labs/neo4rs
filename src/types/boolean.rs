use crate::errors::*;
use bytes::*;
use std::convert::TryInto;

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
}
