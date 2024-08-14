use crate::errors::{Error, Result};
use bytes::{BufMut, BytesMut};
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Version {
    V4,
    V4_1,
}

impl Version {
    pub fn add_supported_versions(bytes: &mut BytesMut) {
        bytes.reserve(16);
        bytes.put_u32(0x0104); // V4_1
        bytes.put_u32(0x0004); // V4
        bytes.put_u32(0);
        bytes.put_u32(0);
    }

    pub fn parse(version_bytes: [u8; 4]) -> Result<Version> {
        match version_bytes {
            [0, 0, 1, 4] => Ok(Version::V4_1),
            [0, 0, 0, 4] => Ok(Version::V4),
            [0, 0, minor, major] => Err(Error::UnsupportedVersion(major, minor)),
            otherwise => Err(Error::ProtocolMismatch(u32::from_be_bytes(otherwise))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_parse_version() {
        assert_eq!(Version::parse([0, 0, 1, 4]).unwrap(), Version::V4_1);
        assert_eq!(Version::parse([0, 0, 0, 4]).unwrap(), Version::V4);
    }
}
