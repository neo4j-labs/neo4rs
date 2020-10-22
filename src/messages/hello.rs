use crate::error::*;
use crate::types::*;
use bytes::*;
use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x01;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Hello {
    extra: BoltMap,
}

impl Hello {
    pub fn new(extra: BoltMap) -> Hello {
        Hello { extra }
    }
}

impl TryInto<Bytes> for Hello {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let extra: Bytes = self.extra.try_into()?;
        let mut bytes =
            BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<u8>() + extra.len());
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(extra);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
