use crate::error::*;
use crate::types::*;
use bytes::*;
use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB2;
pub const SIGNATURE: u8 = 0x01;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Init {
    client_name: BoltString,
    auth_token: BoltMap,
}

impl Init {
    pub fn new(client_name: BoltString, auth_token: BoltMap) -> Init {
        Init {
            client_name,
            auth_token,
        }
    }
}

impl TryInto<Bytes> for Init {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let client_name: Bytes = self.client_name.try_into()?;
        let auth_token: Bytes = self.auth_token.try_into()?;
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() + mem::size_of::<u8>() + client_name.len() + auth_token.len(),
        );
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(client_name);
        bytes.put(auth_token);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_init_message() {
        let init = Init::new(
            "MyClient/1.0".into(),
            vec![("scheme".into(), "basic".into())]
                .into_iter()
                .collect(),
        );

        let b: Bytes = init.try_into().unwrap();

        assert_eq!(
            b.bytes(),
            Bytes::from_static(&[
                MARKER,
                SIGNATURE,
                string::TINY | 12,
                b'M',
                b'y',
                b'C',
                b'l',
                b'i',
                b'e',
                b'n',
                b't',
                b'/',
                b'1',
                b'.',
                b'0',
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
