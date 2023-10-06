use crate::config::{Config, ConfigBuilder};
use crate::errors::*;
use crate::pool::{create_pool, ConnectionPool};
use crate::query::Query;
use crate::stream::RowStream;
use crate::txn::Txn;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A neo4j database abstraction
pub struct Graph {
    config: Config,
    pool: ConnectionPool,
}

/// Returns a [`Query`] which provides methods like [`Query::param`] to add parameters to the query
pub fn query(q: &str) -> Query {
    Query::new(q.to_owned())
}

impl Graph {
    /// Connects to the database with configurations provided.
    ///
    /// You can build a config using [`ConfigBuilder::default()`].
    pub async fn connect(config: Config) -> Result<Self> {
        let pool = create_pool(&config).await?;
        Ok(Graph { config, pool })
    }

    /// Connects to the database with default configurations
    pub async fn new(
        uri: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self> {
        let config = ConfigBuilder::default()
            .uri(uri)
            .user(user)
            .password(password)
            .build()?;
        Self::connect(config).await
    }

    /// Starts a new transaction on the configured database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    pub async fn start_txn(&self) -> Result<Txn> {
        self.start_txn_on(&self.config.db).await
    }

    /// Starts a new transaction on the provided database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    pub async fn start_txn_on(&self, db: &str) -> Result<Txn> {
        let connection = self.pool.get().await?;
        Txn::new(db, self.config.fetch_size, connection).await
    }

    /// Runs a query on the configured database using a connection from the connection pool,
    /// It doesn't return any [`RowStream`] as the `run` abstraction discards any stream.
    ///
    /// Use [`Graph::run`] for cases where you just want a write operation
    ///
    /// use [`Graph::execute`] when you are interested in the result stream
    pub async fn run(&self, q: Query) -> Result<()> {
        self.run_on(&self.config.db, q).await
    }

    /// Runs a query on the provided database using a connection from the connection pool.
    /// It doesn't return any [`RowStream`] as the `run` abstraction discards any stream.
    ///
    /// Use [`Graph::run`] for cases where you just want a write operation
    ///
    /// use [`Graph::execute`] when you are interested in the result stream
    pub async fn run_on(&self, db: &str, q: Query) -> Result<()> {
        let connection = Arc::new(Mutex::new(self.pool.get().await?));
        q.run(db, connection).await
    }

    /// Executes a query on the configured database and returns a [`RowStream`]
    pub async fn execute(&self, q: Query) -> Result<RowStream> {
        self.execute_on(&self.config.db, q).await
    }

    /// Executes a query on the provided database and returns a [`RowStream`]
    pub async fn execute_on(&self, db: &str, q: Query) -> Result<RowStream> {
        let connection = Arc::new(Mutex::new(self.pool.get().await?));
        q.execute(db, self.config.fetch_size, connection).await
    }
}
