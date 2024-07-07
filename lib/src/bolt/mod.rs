#![allow(unused_imports, dead_code)]

use std::marker::PhantomData;

use bytes::Bytes;
use serde::{
    de::{Deserialize, DeserializeOwned, Deserializer, EnumAccess, Unexpected, VariantAccess as _},
    Serialize,
};

mod detail;
mod request;
mod structs;
mod summary;

pub use request::{
    Commit, Discard, Goodbye, Hello, HelloBuilder, Pull, Reset, Rollback, WrapExtra,
};
pub use structs::{
    Bolt, BoltRef, Date, DateDuration, DateTime, DateTimeZoneId, DateTimeZoneIdRef, Duration,
    LegacyDateTime, LegacyDateTimeZoneId, LegacyDateTimeZoneIdRef, LocalDateTime, LocalTime, Node,
    NodeRef, Path, PathRef, Point2D, Point3D, Relationship, RelationshipRef, Segment, Time,
};
pub use summary::{Failure, Success, Summary};

use crate::packstream::{self, de, from_bytes, from_bytes_ref, ser, to_bytes, Data};

pub(crate) trait Message: Serialize {
    /// Serialize this type into a packstream encoded byte slice.
    fn to_bytes(&self) -> Result<Bytes, ser::Error>;
}

impl<T: Serialize> Message for T {
    fn to_bytes(&self) -> Result<Bytes, ser::Error> {
        to_bytes(self)
    }
}

pub(crate) trait MessageResponse: Sized {
    /// Deserialize this type from a packstream encoded byte slice.
    fn parse(bytes: Bytes) -> Result<Self, de::Error>;
}

pub(crate) trait MessageResponseRef<'de>: Sized {
    /// Deserialize this type from a packstream encoded byte slice.
    fn parse_ref(bytes: &'de mut Data) -> Result<Self, de::Error>;
}

impl<T: DeserializeOwned + std::fmt::Debug> MessageResponse for T {
    fn parse(bytes: Bytes) -> Result<Self, de::Error> {
        from_bytes(bytes)
    }
}

impl<'de, T: Deserialize<'de> + std::fmt::Debug + 'de> MessageResponseRef<'de> for T {
    fn parse_ref(bytes: &'de mut Data) -> Result<Self, de::Error> {
        from_bytes_ref(bytes)
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

impl<R: std::fmt::Debug, S: std::fmt::Debug> Response<R, S> {
    pub fn into_error(self, msg: &'static str) -> crate::errors::Error {
        match self {
            Response::Failure(f) => f.into_error(),
            otherwise => crate::Error::UnexpectedMessage(format!(
                "unexpected response for {}: {:?}",
                msg, otherwise
            )),
        }
    }
}
