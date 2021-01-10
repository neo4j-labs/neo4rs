use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB0, 0x0F)]
pub struct Reset;

impl Reset {
    pub fn new() -> Reset {
        Reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;

    #[test]
    fn should_serialize_reset() {
        let reset = Reset::new();

        let bytes: Bytes = reset.into_bytes(Version::V4_1).unwrap();

        assert_eq!(bytes, Bytes::from_static(&[0xB0, 0x0F,]));
    }
}
