use crate::auth::{ClientCertificate, ConnectionTLSConfig, MutualTLS};
use crate::errors::{Error, Result};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use serde::{Deserialize, Deserializer, Serialize};
use std::path::Path;
use std::{ops::Deref, sync::Arc};

const DEFAULT_FETCH_SIZE: usize = 200;
const DEFAULT_MAX_CONNECTIONS: usize = 16;

/// Newtype for the name of the database.
/// Stores the name as an `Arc<str>` to avoid cloning the name around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Database(Arc<str>);

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self.0).serialize(serializer)
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl<'de> Deserialize<'de> for Database {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Database::from(s))
    }
}

impl From<&str> for Database {
    fn from(s: &str) -> Self {
        Database(s.into())
    }
}

impl From<String> for Database {
    fn from(s: String) -> Self {
        Database(s.into())
    }
}

impl AsRef<str> for Database {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for Database {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Newtype for the name of the ipersonated user.
/// Stores the name as an `Arc<str>` to avoid cloning the name around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImpersonateUser(Arc<str>);

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl Serialize for ImpersonateUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self.0).serialize(serializer)
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl<'de> Deserialize<'de> for ImpersonateUser {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ImpersonateUser::from(s))
    }
}

impl From<&str> for ImpersonateUser {
    fn from(s: &str) -> Self {
        ImpersonateUser(s.into())
    }
}

impl From<String> for ImpersonateUser {
    fn from(s: String) -> Self {
        ImpersonateUser(s.into())
    }
}

impl AsRef<str> for ImpersonateUser {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ImpersonateUser {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The configuration that is used once a connection is alive.
#[derive(Debug, Clone)]
pub struct LiveConfig {
    pub(crate) db: Option<Database>,
    pub(crate) fetch_size: usize,
    pub(crate) imp_user: Option<ImpersonateUser>,
}

/// The configuration used to connect to the database, see [`crate::Graph::connect`].
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) uri: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) max_connections: usize,
    pub(crate) db: Option<Database>,
    pub(crate) fetch_size: usize,
    pub(crate) imp_user: Option<ImpersonateUser>,
    pub(crate) tls_config: ConnectionTLSConfig,
}

impl Config {
    pub(crate) fn into_live_config(self) -> LiveConfig {
        LiveConfig {
            db: self.db,
            fetch_size: self.fetch_size,
            imp_user: self.imp_user,
        }
    }
}

/// A builder to override default configurations and build the [`Config`].
pub struct ConfigBuilder {
    uri: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Option<Database>,
    fetch_size: usize,
    max_connections: usize,
    imp_user: Option<ImpersonateUser>,
    tls_config: ConnectionTLSConfig,
}

impl ConfigBuilder {
    /// Creates a new `ConfigBuilder` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// The uri of the Neo4j server, e.g. "127.0.0.1:7687".
    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// The username for authenticating with the Neo4j server.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// The password for authenticating with the Neo4j server.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// The name of the database to connect to.
    ///
    /// Defaults to the server configured default database if not set.
    /// The database can also be specified on a per-query level, which will
    /// override this value.
    pub fn db(mut self, db: impl Into<Database>) -> Self {
        self.db = Some(db.into());
        self
    }

    /// `fetch_size` indicates the number of rows to fetch from server in one request.
    /// It is recommended to use a large `fetch_size` if you are working with large data sets.
    ///
    /// Defaults to 200 if not set.
    pub fn fetch_size(mut self, fetch_size: usize) -> Self {
        self.fetch_size = fetch_size;
        self
    }

    /// The maximum number of connections in the connection pool.
    ///
    /// Defaults to 16 if not set.
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// A CA certificate to use to validate the server's certificate.
    ///
    /// This is required if the server's certificate is not signed by a known CA.
    pub fn with_client_certificate(mut self, client_cert: impl AsRef<Path>) -> Self {
        self.tls_config =
            ConnectionTLSConfig::ClientCACertificate(ClientCertificate::new(client_cert));
        self
    }

    //Used for bidirectional authentication
    pub fn with_mutual_tls_validation(
        mut self,
        client_cert: Option<impl AsRef<Path>>,
        ssl_cert: impl AsRef<Path>,
        ssl_key: impl AsRef<Path>,
    ) -> Self {
        self.tls_config =
            ConnectionTLSConfig::MutualTLS(MutualTLS::new(client_cert, ssl_cert, ssl_key));
        self
    }

    /// Skip SSL validation. This is not recommended for production use.
    /// This is true by default when connecting to the server using `neo4j+ssc` or 'bolt+ssc' schemes.
    pub fn skip_ssl_validation(mut self) -> Self {
        self.tls_config = ConnectionTLSConfig::NoSSLValidation;
        self
    }

    /// Set a user to impersonate when executing queries
    pub fn with_impersonate_user(mut self, imp_user: impl Into<ImpersonateUser>) -> Self {
        self.imp_user = Some(imp_user.into());
        self
    }

    pub fn build(self) -> Result<Config> {
        if let (Some(uri), Some(user), Some(password)) = (self.uri, self.user, self.password) {
            Ok(Config {
                uri,
                user,
                password,
                fetch_size: self.fetch_size,
                max_connections: self.max_connections,
                db: self.db,
                imp_user: self.imp_user,
                tls_config: self.tls_config,
            })
        } else {
            Err(Error::InvalidConfig)
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            uri: None,
            user: None,
            password: None,
            db: None,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            imp_user: None,
            fetch_size: DEFAULT_FETCH_SIZE,
            tls_config: ConnectionTLSConfig::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_config() {
        let config = ConfigBuilder::default()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .password("some_password")
            .db("some_db")
            .fetch_size(10)
            .max_connections(5)
            .build()
            .unwrap();
        assert_eq!(config.uri, "127.0.0.1:7687");
        assert_eq!(config.user, "some_user");
        assert_eq!(config.password, "some_password");
        assert_eq!(config.db.as_deref(), Some("some_db"));
        assert_eq!(config.fetch_size, 10);
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.tls_config, ConnectionTLSConfig::None);
    }

    #[test]
    fn should_build_with_defaults() {
        let config = ConfigBuilder::default()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .password("some_password")
            .build()
            .unwrap();
        assert_eq!(config.uri, "127.0.0.1:7687");
        assert_eq!(config.user, "some_user");
        assert_eq!(config.password, "some_password");
        assert_eq!(config.db, None);
        assert_eq!(config.fetch_size, 200);
        assert_eq!(config.max_connections, 16);
        assert_eq!(config.tls_config, ConnectionTLSConfig::None);
    }

    #[test]
    fn should_build_with_tls_config() {
        let config = ConfigBuilder::default()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .password("some_password")
            .skip_ssl_validation()
            .build()
            .unwrap();
        assert_eq!(config.uri, "127.0.0.1:7687");
        assert_eq!(config.user, "some_user");
        assert_eq!(config.password, "some_password");
        assert_eq!(config.db, None);
        assert_eq!(config.fetch_size, 200);
        assert_eq!(config.max_connections, 16);
        assert_eq!(config.tls_config, ConnectionTLSConfig::NoSSLValidation);
    }

    #[test]
    fn should_reject_invalid_config() {
        assert!(ConfigBuilder::default()
            .user("some_user")
            .password("some_password")
            .build()
            .is_err());

        assert!(ConfigBuilder::default()
            .uri("127.0.0.1:7687")
            .password("some_password")
            .build()
            .is_err());

        assert!(ConfigBuilder::default()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .build()
            .is_err());
    }
}
