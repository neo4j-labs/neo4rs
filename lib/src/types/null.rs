use bytes::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Hash, Clone, BoltStruct)]
#[signature(0xC0)]
pub struct BoltNull;

impl BoltNull {
    pub fn new() -> BoltNull {
        BoltNull {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

    #[test]
    fn should_serialize_null() {
        let null = BoltNull::new();
        let b: Bytes = null.to_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[0xC0]);
    }
}
