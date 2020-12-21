use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB0;
pub const SIGNATURE: u8 = 0x13;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB0, 0x13)]
pub struct Rollback;

impl Rollback {
    pub fn new() -> Rollback {
        Rollback {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_rollback() {
        let rollback = Rollback::new();

        let bytes: Bytes = rollback.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[MARKER, SIGNATURE,]));
    }
}
