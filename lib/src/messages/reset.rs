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
    use bytes::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_reset() {
        let reset = Reset::new();

        let bytes: Bytes = reset.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[0xB0, 0x0F,]));
    }
}
