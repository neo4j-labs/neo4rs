#![cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]

use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB0, 0x02)]
#[cfg_attr(
    feature = "unstable-bolt-protocol-impl-v2",
    deprecated(since = "0.9.0", note = "Use `crate::bolt::Bye` instead.")
)]
pub struct Bye;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::BoltWireFormat, version::Version};
    use bytes::*;

    #[test]
    fn should_serialize_bye() {
        let bye = Bye {};

        let bytes: Bytes = bye.into_bytes(Version::V4_1).unwrap();

        assert_eq!(bytes, Bytes::from_static(&[0xB0, 0x02,]));
    }
}
