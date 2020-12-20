use crate::errors::*;
use bytes::*;
use std::convert::TryInto;

pub const MARKER: u8 = 0xC0;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BoltNull;

impl BoltNull {
    pub fn new() -> BoltNull {
        BoltNull {}
    }
}

impl TryInto<Bytes> for BoltNull {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        Ok(Bytes::copy_from_slice(&[MARKER]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_null() {
        let null = BoltNull::new();
        let b: Bytes = null.try_into().unwrap();
        assert_eq!(b.bytes(), &[0xC0]);
    }
}
