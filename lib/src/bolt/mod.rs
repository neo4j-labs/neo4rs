use bytes::Bytes;
use serde::de::DeserializeOwned;

mod packstream;

pub use packstream::{de, from_bytes};

pub(crate) trait MessageResponse: DeserializeOwned {
    fn parse(bytes: Bytes) -> Result<Self, de::Error>;
}

impl<T: DeserializeOwned> MessageResponse for T {
    fn parse(bytes: Bytes) -> Result<Self, de::Error> {
        packstream::from_bytes(bytes)
    }
}
