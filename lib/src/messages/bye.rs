use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB0, 0x02)]
pub struct Bye;

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_bye() {
        let bye = Bye {};

        let bytes: Bytes = bye.try_into().unwrap();

        assert_eq!(bytes, Bytes::from_static(&[0xB0, 0x02,]));
    }
}
