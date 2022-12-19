use std::{io, string::FromUtf8Error};

use deadpool::managed::PoolError;

use crate::types::{BoltDate, BoltType};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("an IO error occurred")]
    IoError(#[from] io::Error),

    #[error("connection pool error: {0:?}")]
    ConnectionError(Box<PoolError<Self>>),

    #[error("attempted to serialize excessively long string")]
    StringTooLong,

    #[error("attempted to serialize excessively large map")]
    MapTooBig,

    #[error("attempted to serialize excessively large byte array")]
    BytesTooBig,

    #[error("attempted to serialize excessively long list")]
    ListTooLong,

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("version {0} is not supported")]
    UnsupportedVersion(u32),

    #[error("an unexpected response was received for `{request}`: {response}")]
    UnexpectedMessage {
        request: String,
        response: String
    },

    #[error("attempted to parse unknown type: `{0}`")]
    UnknownType(String),

    #[error("received unknown message: {0}")]
    UnknownMessage(String),

    #[error("attempted to convert {0:?} into differing native type")]
    ConvertError(BoltType),

    #[error("failed to convert `{0:?}` into native type")]
    DateConvertError(BoltDate),

    #[error("authentication error: {0}")]
    AuthenticationError(String),

    #[error("invalid {type_name} marker: {marker}")]
    InvalidTypeMarker {
        type_name: &'static str,
        marker: u8,
    },

    #[error("deserialization error")]
    DeserializationError(#[from] FromUtf8Error),
}

impl From<PoolError<Error>> for Error {
    fn from(e: PoolError<Error>) -> Self {
        match e {
            PoolError::Backend(e) => e,
            _ => Error::ConnectionError(Box::new(e)),
        }
    }
}

pub fn unexpected<T: std::fmt::Debug>(response: T, request: &str) -> Error {
    Error::UnexpectedMessage { request: request.into(), response: format!("{:?}", response) }
}
