use std::marker::PhantomData;

use bytes::Bytes;
use serde::{
    de::{Deserialize, DeserializeOwned, Deserializer, EnumAccess, Unexpected, VariantAccess as _},
    Serialize,
};

mod detail;
mod packstream;
mod summary;

pub use detail::Detail;
#[cfg(debug_assertions)]
pub use packstream::debug::Dbg;
use packstream::ser::AsMap;
pub use packstream::{de, ser};
pub use summary::{Failure, Streaming, StreamingSummary, Success, Summary};

pub(crate) trait Message: Serialize {
    /// Serialize this type into a packstream encoded byte slice.
    fn to_bytes(&self) -> Result<Bytes, ser::Error>;
}

impl<T: Serialize> Message for T {
    fn to_bytes(&self) -> Result<Bytes, ser::Error> {
        packstream::to_bytes(self)
    }
}

pub(crate) trait MessageResponse: Sized {
    /// Deserialize this type from a packstream encoded byte slice.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Response<R, S> {
    Detail(R),
    Success(S),
    Ignored,
    Failure(Failure),
}

impl<'de, R: Deserialize<'de>, S: Deserialize<'de>> Deserialize<'de> for Response<R, S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<T>(PhantomData<T>);

        impl<'de, R: Deserialize<'de>, S: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<(R, S)> {
            type Value = Response<R, S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A Bolt response struct")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, data) = data.variant::<u8>()?;
                match tag {
                    0x70 => Ok(Response::Success(data.newtype_variant::<S>()?)),
                    0x71 => Ok(Response::Detail(data.newtype_variant::<R>()?)),
                    0x7E => Ok(Response::Ignored),
                    0x7F => Ok(Response::Failure(data.newtype_variant::<Failure>()?)),
                    _ => Err(serde::de::Error::invalid_type(
                        // TODO: proper error
                        Unexpected::Other(&format!("struct with tag {tag:02X}")),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_enum(
            "Response",
            &["Detail", "Success", "Ignore", "Failure"],
            Visitor(PhantomData),
        )
    }
}
