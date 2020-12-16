use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x11;

#[derive(Debug, PartialEq, Clone)]
pub struct Begin {
    extra: BoltMap,
}

impl Begin {
    pub fn new(extra: BoltMap) -> Begin {
        Begin { extra }
    }
}

impl TryInto<Bytes> for Begin {
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
                MARKER,
                SIGNATURE,
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
