use crate::error::*;
use bytes::*;
use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB0;
pub const SIGNATURE: u8 = 0x02;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Bye;

impl TryInto<Bytes> for Bye {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let mut bytes = BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<u8>());
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_bye() {
        let bye = Bye {};

        let bytes: Bytes = bye.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[MARKER, SIGNATURE,]));
    }
}
