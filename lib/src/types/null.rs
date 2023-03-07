use bytes::*;
use neo4rs_macros::BoltStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xC0)]
pub struct BoltNull;

impl Serialize for BoltNull {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl Default for BoltNull {
    fn default() -> Self {
        BoltNull
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

    #[test]
    fn should_serialize_null() {
        let null = BoltNull::default();
        let b: Bytes = null.into_bytes(Version::V4_1).unwrap();
        assert_eq!(&b[..], &[0xC0]);
    }
}
