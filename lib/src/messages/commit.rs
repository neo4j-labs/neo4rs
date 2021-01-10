use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB0, 0x12)]
pub struct Commit;

impl Commit {
    pub fn new() -> Commit {
        Commit {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;

    #[test]
    fn should_serialize_commit() {
        let commit = Commit::new();

        let bytes: Bytes = commit.into_bytes(Version::V4_1).unwrap();

        assert_eq!(bytes, Bytes::from_static(&[0xB0, 0x12,]));
    }
}
