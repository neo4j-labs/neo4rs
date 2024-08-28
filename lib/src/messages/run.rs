use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB3, 0x10)]
pub struct Run {
    query: BoltString,
    parameters: BoltMap,
    extra: BoltMap,
}

impl Run {
    pub fn new(db: Option<BoltString>, query: BoltString, parameters: BoltMap) -> Run {
        Run {
            query,
            parameters,
            extra: db
                .into_iter()
                .map(|db| ("db".into(), BoltType::String(db)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::version::Version;

    #[test]
    fn should_serialize_run() {
        let run = Run::new(
            Some("test".into()),
            "query".into(),
            vec![("k".into(), "v".into())].into_iter().collect(),
        );

        let bytes: Bytes = run.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3,
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
                map::TINY | 1,
                string::TINY | 2,
                b'd',
                b'b',
                string::TINY | 4,
                b't',
                b'e',
                b's',
                b't',
            ])
        );
    }

    #[test]
    fn should_serialize_run_with_no_params() {
        let run = Run::new(None, "query".into(), BoltMap::default());

        let bytes: Bytes = run.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3,
                0x10,
                string::TINY | 5,
                b'q',
                b'u',
                b'e',
                b'r',
                b'y',
                map::TINY,
                map::TINY,
            ])
        );
    }
}
