#[cfg(feature = "unstable-serde-packstream-format")]
use crate::packstream::{de, ser};
use crate::DeError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("an IO error occurred: {detail}")]
    IOError {
        #[from]
        detail: std::io::Error,
    },

    #[error("Invalid URI: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[cfg(feature = "unstable-serde-packstream-format")]
    #[error(transparent)]
    WriteError(#[from] ser::Error),

    #[cfg(feature = "unstable-serde-packstream-format")]
    #[error(transparent)]
    ParseError(#[from] de::Error),

    #[error("Unsupported URI scheme: {0}")]
    UnsupportedScheme(String),

    #[error("Invalid DNS name: {0}")]
    InvalidDnsName(String),

    #[error("connection error")]
    ConnectionError,

    #[error("attempted to serialize excessively long string")]
    StringTooLong,

    #[error("attempted to serialize excessively large map")]
    MapTooBig,

    #[error("attempted to serialize excessively large byte array")]
    BytesTooBig,

    #[error("attempted to serialize excessively long list")]
    ListTooLong,

    #[error("Invalid integer for the parameter {0}: must be positive or -1, but was {1}")]
    InvalidInteger(&'static str, i64),

    #[error("The provided integer for {0} does not fit in the range of an i64: {1}")]
    IntegerOverflow(&'static str, #[source] std::num::TryFromIntError),

    #[error("invalid config")]
    InvalidConfig,

    #[error("Bolt Version {0}.{1} is not supported")]
    UnsupportedVersion(u8, u8),

    #[error(
        "Protocol mismatch: Expected a Bolt version as response, \
             got {0:08x} instead (maybe you connected to the HTTP port?)"
    )]
    ProtocolMismatch(u32),

    #[error("FAILURE response to {msg} [{code}]: {message}")]
    Failure {
        code: String,
        message: String,
        msg: &'static str,
    },

    #[error("{0}")]
    UnexpectedMessage(String),

    #[error("{0} message was ignored by the server")]
    Ignored(&'static str),

    #[error("{0}")]
    UnknownType(String),

    #[error("{0}")]
    UnknownMessage(String),

    #[error("conversion error")]
    ConversionError,

    #[error("{0}")]
    AuthenticationError(String),

    #[error("{0}")]
    InvalidTypeMarker(String),

    #[error("{0}")]
    DeserializationError(DeError),
}

impl std::convert::From<deadpool::managed::PoolError<Error>> for Error {
    fn from(e: deadpool::managed::PoolError<Error>) -> Self {
        match e {
            deadpool::managed::PoolError::Backend(e) => e,
            _ => Error::ConnectionError,
        }
    }
}
