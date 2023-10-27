use crate::types::BoltMap;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x2F)]
pub struct Discard {
    extra: BoltMap,
}

impl Default for Discard {
    fn default() -> Self {
        Discard::new(-1, -1)
    }
}

impl Discard {
    pub fn new(n: i64, qid: i64) -> Discard {
        let mut extra = BoltMap::default();
        extra.put("n".into(), n.into());
        extra.put("qid".into(), qid.into());
        Discard { extra }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::BoltWireFormat, version::Version};
    use bytes::Bytes;

    #[test]
    fn should_serialize_discard_message() {
        let discard = Discard::new(42, 1);
        let mut bytes: Bytes = discard.into_bytes(Version::V4_1).unwrap();
        let marker_signature = bytes.split_to(2);
        assert_eq!(&*marker_signature, &[0xB1, 0x2F]);
        let extra: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();

        assert_eq!(extra.get::<i64>("n").unwrap(), 42);
        assert_eq!(extra.get::<i64>("qid").unwrap(), 1);
    }

    #[test]
    fn should_serialize_discard_with_default_value() {
        let discard = Discard::default();
        let mut bytes: Bytes = discard.into_bytes(Version::V4_1).unwrap();
        let marker_signature = bytes.split_to(2);
        assert_eq!(&*marker_signature, &[0xB1, 0x2F]);
        let extra: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();

        assert_eq!(extra.get::<i64>("n").unwrap(), 255);
        assert_eq!(extra.get::<i64>("qid").unwrap(), 255);
    }
}
