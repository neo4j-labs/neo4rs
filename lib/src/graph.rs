use crate::{
    config::{Config, ConfigBuilder, Database, LiveConfig},
    errors::Result,
    pool::{create_pool, ConnectionPool},
    query::Query,
    stream::DetachedRowStream,
    txn::Txn,
};

/// A neo4j database abstraction.
/// This type can be cloned and shared across threads, internal resources
/// are reference-counted.
#[derive(Clone)]
pub struct Graph {
    config: LiveConfig,
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
        let config = config.into_live_config();
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
        self.start_txn_on(self.config.db.clone()).await
    }

    /// Starts a new transaction on the provided database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    pub async fn start_txn_on(&self, db: impl Into<Database>) -> Result<Txn> {
        let connection = self.pool.get().await?;
        Txn::new(db.into(), self.config.fetch_size, connection).await
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
        let mut connection = self.pool.get().await?;
        q.run(db, &mut connection).await
    }

    /// Executes a query on the configured database and returns a [`DetachedRowStream`]
    pub async fn execute(&self, q: Query) -> Result<DetachedRowStream> {
        self.execute_on(&self.config.db, q).await
    }

    /// Executes a query on the provided database and returns a [`DetaRowStream`]
    pub async fn execute_on(&self, db: &str, q: Query) -> Result<DetachedRowStream> {
        let connection = self.pool.get().await?;
        q.execute(db, self.config.fetch_size, connection).await
    }
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync>() {}
    assert_send_sync::<Graph>();
};
