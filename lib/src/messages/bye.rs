use bytes::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB0;
pub const SIGNATURE: u8 = 0x02;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
pub struct Bye;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_bye() {
        let bye = Bye {};

        let bytes: Bytes = bye.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[MARKER, SIGNATURE,]));
    }
}
