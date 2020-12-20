use crate::types::*;
use bytes::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x01;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
pub struct Hello {
    extra: BoltMap,
}

impl Hello {
    pub fn new(extra: BoltMap) -> Hello {
        Hello { extra }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_hello() {
        let hello = Hello::new(
            vec![("scheme".into(), "basic".into())]
                .into_iter()
                .collect(),
        );

        let bytes: Bytes = hello.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
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
