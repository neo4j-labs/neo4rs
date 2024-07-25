use crate::auth::ClientCertificate;
use crate::errors::{Error, Result};
use std::path::Path;
use std::{ops::Deref, sync::Arc};

const DEFAULT_DATABASE: &str = "neo4j";
const DEFAULT_FETCH_SIZE: usize = 200;
const DEFAULT_MAX_CONNECTIONS: usize = 16;

/// Newtype for the name of the database.
/// Stores the name as an `Arc<str>` to avoid cloning the name around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Database(Arc<str>);

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

impl Default for Database {
    fn default() -> Self {
        Database(DEFAULT_DATABASE.into())
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

/// The configuration that is used once a connection is alive.
#[derive(Debug, Clone)]
pub struct LiveConfig {
    pub(crate) db: Database,
    pub(crate) fetch_size: usize,
}

/// The configuration used to connect to the database, see [`crate::Graph::connect`].
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) uri: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) max_connections: usize,
    pub(crate) db: Database,
    pub(crate) fetch_size: usize,
    pub(crate) client_certificate: Option<ClientCertificate>,
}

impl Config {
    pub(crate) fn into_live_config(self) -> LiveConfig {
        LiveConfig {
            db: self.db,
            fetch_size: self.fetch_size,
        }
    }
}

/// A builder to override default configurations and build the [`Config`].
pub struct ConfigBuilder {
    uri: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Database,
    fetch_size: usize,
    max_connections: usize,
    client_certificate: Option<ClientCertificate>,
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
    /// Defaults to "neo4j" if not set.
    pub fn db(mut self, db: impl Into<Database>) -> Self {
        self.db = db.into();
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

    pub fn with_client_certificate(mut self, client_cert: impl AsRef<Path>) -> Self {
        self.client_certificate = Some(ClientCertificate::new(client_cert));
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
                client_certificate: self.client_certificate,
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
            db: DEFAULT_DATABASE.into(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            fetch_size: DEFAULT_FETCH_SIZE,
            client_certificate: None,
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
        assert_eq!(&*config.db, "some_db");
        assert_eq!(config.fetch_size, 10);
        assert_eq!(config.max_connections, 5);
        assert!(config.client_certificate.is_none());
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
        assert_eq!(&*config.db, "neo4j");
        assert_eq!(config.fetch_size, 200);
        assert_eq!(config.max_connections, 16);
        assert!(config.client_certificate.is_none());
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
