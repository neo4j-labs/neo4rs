pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IOError { detail: String },
    ConnectionError,
    StringTooLong,
    MapTooBig,
    BytesTooBig,
    ListTooLong,
    InvalidConfig,
    UnsupportedVersion(String),
    UnexpectedMessage(String),
    UnknownType(String),
    UnknownMessage(String),
    ConversionError,
    AuthenticationError(String),
    InvalidTypeMarker(String),
    DeserializationError(String),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError {
            detail: e.to_string(),
        }
    }
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
