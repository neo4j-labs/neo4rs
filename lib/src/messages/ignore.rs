use crate::errors::Neo4jError;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB0, 0x7E)]
pub struct Ignore;

impl Ignore {
    pub(crate) fn into_error(self) -> Neo4jError {
        Neo4jError::new(
            "Neo.ServerError.Ignored".into(),
            "The request was ignored by the server because it is in a FAILED or INTERRUPTED state"
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoltWireFormat;
    use crate::version::Version;
    use bytes::Bytes;

    #[test]
    fn should_deserialize_success() {
        let mut data = Bytes::from_static(&[0xB0, 0x7E]);

        let failure: Ignore = Ignore::parse(Version::V4_1, &mut data).unwrap();
        let failure = failure.into_error();

        assert_eq!(failure.code(), "Neo.ServerError.Ignored");
        assert_eq!(
            failure.message(),
            "The request was ignored by the server because it is in a FAILED or INTERRUPTED state"
        );
    }
}
