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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Neo4jErrorKind {
    Client(Neo4jClientErrorKind),
    Transient,
    Database,
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Neo4jClientErrorKind {
    Security(Neo4jSecurityErrorKind),
    SessionExpired,
    FatalDiscovery,
    TransactionTerminated,
    ProtocolViolation,
    Other,
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Neo4jSecurityErrorKind {
    Authentication,
    AuthorizationExpired,
    TokenExpired,
    Other,
    Unknown,
}

impl Neo4jErrorKind {
    pub(crate) fn new(code: &str) -> Self {
        let code = Self::adjust_code(code).unwrap_or(code);
        Self::classify(code)
    }

    fn adjust_code(code: &str) -> Option<&str> {
        match code {
            "Neo.TransientError.Transaction.LockClientStopped" => {
                Some("Neo.ClientError.Transaction.LockClientStopped")
            }
            "Neo.TransientError.Transaction.Terminated" => {
                Some("Neo.ClientError.Transaction.Terminated")
            }
            _ => None,
        }
    }

    fn classify(code: &str) -> Self {
        let mut parts = code.split('.').skip(1);
        let [class, subclass, kind] = [parts.next(), parts.next(), parts.next()];

        match class {
            Some("ClientError") => match (subclass, kind) {
                (Some("Security"), Some("Unauthorized")) => Self::Client(
                    Neo4jClientErrorKind::Security(Neo4jSecurityErrorKind::Authentication),
                ),
                (Some("Security"), Some("AuthorizationExpired")) => Self::Client(
                    Neo4jClientErrorKind::Security(Neo4jSecurityErrorKind::AuthorizationExpired),
                ),
                (Some("Security"), Some("TokenExpired")) => Self::Client(
                    Neo4jClientErrorKind::Security(Neo4jSecurityErrorKind::TokenExpired),
                ),
                (Some("Database"), Some("DatabaseNotFound")) => {
                    Self::Client(Neo4jClientErrorKind::FatalDiscovery)
                }
                (Some("Transaction"), Some("Terminated")) => {
                    Self::Client(Neo4jClientErrorKind::TransactionTerminated)
                }
                (Some("Security"), Some(_)) => Self::Client(Neo4jClientErrorKind::Security(
                    Neo4jSecurityErrorKind::Other,
                )),
                (Some("Security"), _) => Self::Client(Neo4jClientErrorKind::Security(
                    Neo4jSecurityErrorKind::Unknown,
                )),
                (Some("Request"), _) => Self::Client(Neo4jClientErrorKind::ProtocolViolation),
                (Some("Cluster"), Some("NotALeader")) => {
                    Self::Client(Neo4jClientErrorKind::SessionExpired)
                }
                (Some("General"), Some("ForbiddenOnReadOnlyDatabase")) => {
                    Self::Client(Neo4jClientErrorKind::SessionExpired)
                }
                (Some(_), _) => Self::Client(Neo4jClientErrorKind::Other),
                _ => Self::Client(Neo4jClientErrorKind::Unknown),
            },
            Some("TransientError") => Self::Transient,
            Some(_) => Self::Database,
            None => Self::Unknown,
        }
    }

    pub(crate) fn can_retry(&self) -> bool {
        matches!(
            self,
            Self::Client(
                Neo4jClientErrorKind::Security(Neo4jSecurityErrorKind::AuthorizationExpired)
                    | Neo4jClientErrorKind::SessionExpired
            ) | Self::Transient
        )
    }
}

impl From<&str> for Neo4jErrorKind {
    fn from(code: &str) -> Self {
        Self::new(code)
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
