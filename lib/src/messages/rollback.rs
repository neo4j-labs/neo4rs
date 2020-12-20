use crate::errors::*;
use bytes::*;
use std::convert::TryInto;

pub const MARKER: u8 = 0xB0;
pub const SIGNATURE: u8 = 0x13;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rollback;

impl Rollback {
    pub fn new() -> Rollback {
        Rollback {}
    }
}

impl TryInto<Bytes> for Rollback {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        Ok(Bytes::from_static(&[MARKER, SIGNATURE]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_rollback() {
        let rollback = Rollback::new();

        let bytes: Bytes = rollback.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[MARKER, SIGNATURE,]));
    }
}
