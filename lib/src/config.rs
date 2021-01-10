pub use crate::errors::*;

const DEFAULT_FETCH_SIZE: usize = 200;
const DEFAULT_MAX_CONNECTIONS: usize = 16;

/// The configuration used to connect to the database, see [`Graph::connect`]
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) uri: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) max_connections: usize,
    pub(crate) db: String,
    pub(crate) fetch_size: usize,
}

/// A builder to override default configurations and build the [`Config`]
pub struct ConfigBuilder {
    uri: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Option<String>,
    fetch_size: Option<usize>,
    max_connections: Option<usize>,
}

impl ConfigBuilder {
    ///the uri of the neo4j server
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_owned());
        self
    }

    ///username for authentication
    pub fn user(mut self, user: &str) -> Self {
        self.user = Some(user.to_owned());
        self
    }

    ///password for authentication
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    ///the name of the database, defaults to "neo4j" if not configured.
    pub fn db(mut self, db: &str) -> Self {
        self.db = Some(db.to_owned());
        self
    }

    ///fetch_size indicates the number of rows to fetch from server in one request, it is
    ///recommended to use a large fetch_size if you are working with large data sets.
    ///default fetch_size is 200
    pub fn fetch_size(mut self, fetch_size: usize) -> Self {
        self.fetch_size = Some(fetch_size);
        self
    }

    ///maximum number of connections in the connection pool
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    pub fn build(self) -> Result<Config> {
        if self.uri.is_none()
            || self.user.is_none()
            || self.password.is_none()
            || self.fetch_size.is_none()
            || self.max_connections.is_none()
            || self.db.is_none()
        {
            Err(Error::InvalidConfig)
        } else {
            //The config attributes are validated before unwrapping
            Ok(Config {
                uri: self.uri.unwrap(),
                user: self.user.unwrap(),
                password: self.password.unwrap(),
                fetch_size: self.fetch_size.unwrap(),
                max_connections: self.max_connections.unwrap(),
                db: self.db.unwrap(),
            })
        }
    }
}

/// Creates a config builder with reasonable default values wherever appropriate.
pub fn config() -> ConfigBuilder {
    ConfigBuilder {
        uri: None,
        user: None,
        password: None,
        db: Some("".to_owned()),
        max_connections: Some(DEFAULT_MAX_CONNECTIONS),
        fetch_size: Some(DEFAULT_FETCH_SIZE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_build_config() {
        let config = config()
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
        let config = config()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .password("some_password")
            .build()
            .unwrap();
        assert_eq!(config.uri, "127.0.0.1:7687");
        assert_eq!(config.user, "some_user");
        assert_eq!(config.password, "some_password");
        assert_eq!(config.db, "");
        assert_eq!(config.fetch_size, 200);
        assert_eq!(config.max_connections, 16);
    }

    #[tokio::test]
    async fn should_reject_invalid_config() {
        assert!(config()
            .user("some_user")
            .password("some_password")
            .build()
            .is_err());

        assert!(config()
            .uri("127.0.0.1:7687")
            .password("some_password")
            .build()
            .is_err());

        assert!(config()
            .uri("127.0.0.1:7687")
            .user("some_user")
            .build()
            .is_err());
    }
}
