use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x10;

#[derive(Debug, PartialEq, Clone)]
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

impl TryInto<Bytes> for Run {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let query: Bytes = self.query.try_into()?;
        let parameters: Bytes = self.parameters.try_into()?;
        let extra: Bytes = self.extra.try_into()?;
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>()
                + mem::size_of::<u8>()
                + query.len()
                + parameters.len()
                + extra.len(),
        );
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(query);
        bytes.put(parameters);
        bytes.put(extra);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                MARKER,
                SIGNATURE,
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
                MARKER,
                SIGNATURE,
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
