#![cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]

use crate::types::BoltMap;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x3F)]
#[cfg_attr(
    feature = "unstable-bolt-protocol-impl-v2",
    deprecated(since = "0.9.0", note = "Use `crate::bolt::Pull` instead.")
)]
pub struct Pull {
    extra: BoltMap,
}

impl Default for Pull {
    fn default() -> Self {
        Pull::new(-1, -1)
    }
}

impl Pull {
    #[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(dead_code))]
    pub fn new(n: i64, qid: i64) -> Pull {
        let mut extra = BoltMap::default();
        extra.put("n".into(), n.into());
        extra.put("qid".into(), qid.into());
        Pull { extra }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoltWireFormat;
    use crate::version::Version;
    use bytes::Bytes;

    #[test]
    fn should_serialize_pull_message() {
        let pull = Pull::new(42, 1);
        let mut bytes: Bytes = pull.into_bytes(Version::V4_1).unwrap();
        let marker_signature = bytes.split_to(2);
        assert_eq!(&*marker_signature, &[0xB1, 0x3F]);
        let extra: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();

        assert_eq!(extra.get::<i64>("n").unwrap(), 42);
        assert_eq!(extra.get::<i64>("qid").unwrap(), 1);
    }

    #[test]
    fn should_serialize_pull_with_default_value() {
        let pull = Pull::default();
        let mut bytes: Bytes = pull.into_bytes(Version::V4_1).unwrap();
        let marker_signature = bytes.split_to(2);
        assert_eq!(&*marker_signature, &[0xB1, 0x3F]);
        let extra: BoltMap = BoltMap::parse(Version::V4_1, &mut bytes).unwrap();

        assert_eq!(extra.get::<i64>("n").unwrap(), 255);
        assert_eq!(extra.get::<i64>("qid").unwrap(), 255);
    }
}
