use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x10)]
pub struct Run {
    query: BoltString,
    parameters: BoltMap,
    extra: BoltMap,
}

impl Run {
    pub fn new(query: BoltString, parameters: BoltMap) -> Run {
        Run {
            query,
            parameters,
            extra: BoltMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_run() {
        let run = Run::new(
            "query".into(),
            vec![("k".into(), "v".into())].into_iter().collect(),
        );

        let bytes: Bytes = run.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB1,
                0x10,
                string::TINY | 5,
                b'q',
                b'u',
                b'e',
                b'r',
                b'y',
                map::TINY | 1,
                string::TINY | 1,
                b'k',
                string::TINY | 1,
                b'v',
                map::TINY | 0,
            ])
        );
    }

    #[test]
    fn should_serialize_run_with_no_params() {
        let run = Run::new("query".into(), BoltMap::new());

        let bytes: Bytes = run.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB1,
                0x10,
                string::TINY | 5,
                b'q',
                b'u',
                b'e',
                b'r',
                b'y',
                map::TINY | 0,
                map::TINY | 0,
            ])
        );
    }
}
