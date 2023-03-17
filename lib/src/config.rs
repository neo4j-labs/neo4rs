pub use crate::errors::*;

const DEFAULT_DATABASE: &'static str = "neo4j";
const DEFAULT_FETCH_SIZE: usize = 200;
const DEFAULT_MAX_CONNECTIONS: usize = 16;

/// The configuration used to connect to the database, see [`crate::Graph::connect`].
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) uri: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) max_connections: usize,
    pub(crate) db: String,
    pub(crate) fetch_size: usize,
}

/// A builder to override default configurations and build the [`Config`].
pub struct ConfigBuilder {
    uri: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Option<String>,
    fetch_size: Option<usize>,
    max_connections: Option<usize>,
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
    pub fn db(mut self, db: impl Into<String>) -> Self {
        self.db = Some(db.into());
        self
    }

    /// `fetch_size` indicates the number of rows to fetch from server in one request.
    /// It is recommended to use a large `fetch_size` if you are working with large data sets.
    ///
    /// Defaults to 200 if not set.
    pub fn fetch_size(mut self, fetch_size: usize) -> Self {
        self.fetch_size = Some(fetch_size);
        self
    }

    /// The maximum number of connections in the connection pool.
    ///
    /// Defaults to 16 if not set.
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    pub fn build(self) -> Result<Config> {
        if let (
            Some(uri),
            Some(user),
            Some(password),
            Some(fetch_size),
            Some(max_connections),
            Some(db),
        ) = (
            self.uri,
            self.user,
            self.password,
            self.fetch_size,
            self.max_connections,
            self.db,
        ) {
            Ok(Config {
                uri,
                user,
                password,
                fetch_size,
                max_connections,
                db,
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
            db: Some(DEFAULT_DATABASE.into()),
            max_connections: Some(DEFAULT_MAX_CONNECTIONS),
            fetch_size: Some(DEFAULT_FETCH_SIZE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_build_config() {
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
        assert_eq!(config.db, "some_db");
        assert_eq!(config.fetch_size, 10);
        assert_eq!(config.max_connections, 5);
    }

    #[tokio::test]
    async fn should_build_with_defaults() {
        let config = ConfigBuilder::default()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .password("some_password")
            .build()
            .unwrap();
        assert_eq!(config.uri, "127.0.0.1:7687");
        assert_eq!(config.user, "some_user");
        assert_eq!(config.password, "some_password");
        assert_eq!(config.db, "neo4j");
        assert_eq!(config.fetch_size, 200);
        assert_eq!(config.max_connections, 16);
    }

    #[tokio::test]
    async fn should_reject_invalid_config() {
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
