use crate::errors::*;
use bytes::*;
use std::convert::TryInto;

pub const MARKER: u8 = 0xB0;
pub const SIGNATURE: u8 = 0x0F;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Reset;

impl Reset {
    pub fn new() -> Reset {
        Reset
    }
}

impl TryInto<Bytes> for Reset {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        Ok(Bytes::from_static(&[MARKER, SIGNATURE]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_reset() {
        let reset = Reset::new();

        let bytes: Bytes = reset.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[MARKER, SIGNATURE,]));
    }
}
