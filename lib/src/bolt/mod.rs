use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

mod packstream;

pub use packstream::{de, from_bytes, ser, to_bytes};

pub(crate) trait Message: Serialize {
    fn to_bytes(&self) -> Result<Bytes, ser::Error>;
}

impl<T: Serialize> Message for T {
    fn to_bytes(&self) -> Result<Bytes, ser::Error> {
        packstream::to_bytes(self)
    }
}

pub(crate) trait MessageResponse: DeserializeOwned {
    fn parse(bytes: Bytes) -> Result<Self, de::Error>;
}

impl<T: DeserializeOwned> MessageResponse for T {
    fn parse(bytes: Bytes) -> Result<Self, de::Error> {
        packstream::from_bytes(bytes)
    }
}

pub(crate) trait ExpectedResponse {
    type Response: MessageResponse;
}
