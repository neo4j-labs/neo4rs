use bytes::{Bytes, BytesMut};

use crate::{errors::Result, Version};

pub(crate) trait BoltWireFormat: Sized {
    // TODO: remove
    fn can_parse(version: Version, input: &[u8]) -> bool;

    fn parse(version: Version, input: &mut Bytes) -> Result<Self>;

    fn write_into(&self, version: Version, bytes: &mut BytesMut) -> Result<()>;

    fn into_bytes(self, version: Version) -> Result<Bytes> {
        let mut bytes = BytesMut::new();
        self.write_into(version, &mut bytes)?;
        Ok(bytes.freeze())
    }
}
