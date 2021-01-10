use neo4rs_macros::BoltStruct;

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
    use crate::version::Version;
    use bytes::*;

    #[test]
    fn should_serialize_rollback() {
        let rollback = Rollback::new();

        let bytes: Bytes = rollback.into_bytes(Version::V4_1).unwrap();

        assert_eq!(bytes, Bytes::from_static(&[0xB0, 0x13,]));
    }
}
