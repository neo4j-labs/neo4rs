pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("an IO error occurred: {detail}")]
    IOError {
        #[from]
        detail: std::io::Error,
    },

    #[error("Invalid URI: {0}")]
    UrlParseError(#[from] url::ParseError),

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

    #[error("invalid config")]
    InvalidConfig,

    #[error("{0}")]
    UnsupportedVersion(String),

    #[error("{0}")]
    UnexpectedMessage(String),

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
    DeserializationError(String),
}

impl std::convert::From<deadpool::managed::PoolError<Error>> for Error {
    fn from(e: deadpool::managed::PoolError<Error>) -> Self {
        match e {
            deadpool::managed::PoolError::Backend(e) => e,
            _ => Error::ConnectionError,
        }
    }
}

impl std::convert::From<deadpool::managed::BuildError<Error>> for Error {
    fn from(value: deadpool::managed::BuildError<Error>) -> Self {
        match value {
            deadpool::managed::BuildError::Backend(e) => e,
            _ => Error::ConnectionError,
        }
    }
}

pub fn unexpected<T: std::fmt::Debug>(response: T, request: &str) -> Error {
    Error::UnexpectedMessage(format!(
        "unexpected response for {}: {:?}",
        request, response
    ))
}
