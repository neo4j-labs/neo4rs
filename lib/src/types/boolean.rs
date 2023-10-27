use crate::{
    errors::{Error, Result},
    types::BoltWireFormat,
    version::Version,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};

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
}

impl BoltWireFormat for BoltBoolean {
    fn can_parse(_version: Version, input: &[u8]) -> bool {
        let input = input[0];
        input == TRUE || input == FALSE
    }

    fn parse(_version: Version, input: &mut Bytes) -> Result<Self> {
        let value = input.get_u8();
        match value {
            TRUE => Ok(BoltBoolean::new(true)),
            FALSE => Ok(BoltBoolean::new(false)),
            _ => Err(Error::InvalidTypeMarker("invalid boolean marker".into())),
        }
    }

    fn write_into(&self, _version: Version, bytes: &mut BytesMut) -> Result<()> {
        let value = if self.value { TRUE } else { FALSE };
        bytes.reserve(1);
        bytes.put_u8(value);
        Ok(())
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
        let mut b = Bytes::from_static(&[TRUE]);
        let bolt_boolean: BoltBoolean = BoltBoolean::parse(Version::V4_1, &mut b).unwrap();
        assert!(bolt_boolean.value);

        let mut b = Bytes::from_static(&[FALSE]);
        let bolt_boolean: BoltBoolean = BoltBoolean::parse(Version::V4_1, &mut b).unwrap();
        assert!(!bolt_boolean.value);
    }
}
