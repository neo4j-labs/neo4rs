use crate::{
    errors::Neo4jError,
    types::{serde::DeError, BoltMap},
    BoltType, Neo4jErrorKind,
};
use ::serde::Deserialize;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x7F)]
pub struct Failure {
    metadata: BoltMap,
}

impl Failure {
    pub fn get<'this, T>(&'this self, key: &str) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.metadata.get::<T>(key)
    }

    pub(crate) fn into_error(self) -> Neo4jError {
        let mut meta = self.metadata.value;
        let (code, message) = (meta.remove("code"), meta.remove("message"));
        let (code, message) = match (code, message) {
            (Some(BoltType::String(s)), Some(BoltType::String(m))) => (s.value, m.value),
            _ => (String::new(), String::new()),
        };
        Neo4jErrorKind::new(&code).new_error(code, message)
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
        let mut data = Bytes::from_static(&[
            0xB1, 0x7F, 0xA2, 0x84, 0x63, 0x6F, 0x64, 0x65, 0xD0, 0x25, 0x4E, 0x65, 0x6F, 0x2E,
            0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x45, 0x72, 0x72, 0x6F, 0x72, 0x2E, 0x53, 0x65,
            0x63, 0x75, 0x72, 0x69, 0x74, 0x79, 0x2E, 0x55, 0x6E, 0x61, 0x75, 0x74, 0x68, 0x6F,
            0x72, 0x69, 0x7A, 0x65, 0x64, 0x87, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0xD0,
            0x39, 0x54, 0x68, 0x65, 0x20, 0x63, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x20, 0x69, 0x73,
            0x20, 0x75, 0x6E, 0x61, 0x75, 0x74, 0x68, 0x6F, 0x72, 0x69, 0x7A, 0x65, 0x64, 0x20,
            0x64, 0x75, 0x65, 0x20, 0x74, 0x6F, 0x20, 0x61, 0x75, 0x74, 0x68, 0x65, 0x6E, 0x74,
            0x69, 0x63, 0x61, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x66, 0x61, 0x69, 0x6C, 0x75, 0x72,
            0x65, 0x2E,
        ]);

        let failure: Failure = Failure::parse(Version::V4_1, &mut data).unwrap();

        assert_eq!(
            failure.get::<String>("code").unwrap(),
            "Neo.ClientError.Security.Unauthorized"
        );
        assert_eq!(
            failure.get::<String>("message").unwrap(),
            "The client is unauthorized due to authentication failure."
        );
    }
}
