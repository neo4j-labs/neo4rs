use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x11)]
pub struct Begin {
    extra: BoltMap,
}

impl Begin {
    pub fn new(extra: BoltMap) -> Begin {
        Begin { extra }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_begin() {
        let begin = Begin::new(
            vec![("tx_timeout".into(), 2000.into())]
                .into_iter()
                .collect(),
        );

        let bytes: Bytes = begin.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB1,
                0x11,
                map::TINY | 1,
                string::TINY | 10,
                b't',
                b'x',
                b'_',
                b't',
                b'i',
                b'm',
                b'e',
                b'o',
                b'u',
                b't',
                integer::INT_16,
                0x07,
                0xD0
            ])
        );
    }
}
