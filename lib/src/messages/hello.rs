#![cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]

use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x01)]
#[cfg_attr(
    feature = "unstable-bolt-protocol-impl-v2",
    deprecated(since = "0.9.0", note = "Use `crate::bolt::Hello` instead.")
)]
pub struct Hello {
    extra: BoltMap,
}

impl Hello {
    #[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(dead_code))]
    pub fn new(extra: BoltMap) -> Hello {
        Hello { extra }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;

    #[test]
    fn should_serialize_hello() {
        let hello = Hello::new(
            vec![("scheme".into(), "basic".into())]
                .into_iter()
                .collect(),
        );

        let bytes: Bytes = hello.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB1,
                0x01,
                map::TINY | 1,
                string::TINY | 6,
                b's',
                b'c',
                b'h',
                b'e',
                b'm',
                b'e',
                string::TINY | 5,
                b'b',
                b'a',
                b's',
                b'i',
                b'c',
            ])
        );
    }
}
