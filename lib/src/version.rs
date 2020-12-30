use bytes::{BufMut, Bytes, BytesMut};
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Version {
    V4_1,
    V4,
}

impl Version {
    pub fn supported_versions() -> Bytes {
        let mut bytes = BytesMut::with_capacity(16);
        let versions: [u32; 4] = [0x0104, 0x0004, 0, 0];
        for version in versions.iter() {
            bytes.put_u32(*version);
        }
        bytes.freeze()
    }

    pub fn parse(version_bytes: [u8; 4]) -> Version {
        match u32::from_be_bytes(version_bytes) {
            260 => Version::V4_1,
            4 => Version::V4,
            _ => panic!("unknown version {:?}", version_bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_parse_version() {
        assert_eq!(Version::parse([0, 0, 1, 4]), Version::V4_1);
    }
}
