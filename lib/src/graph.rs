use std::time::Duration;

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
    ///
    /// Transactions will not be automatically retried on any failure.
    pub async fn start_txn(&self) -> Result<Txn> {
        self.impl_start_txn_on(self.config.db.clone()).await
    }

    /// Starts a new transaction on the provided database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    pub async fn start_txn_on(&self, db: impl Into<Database>) -> Result<Txn> {
        self.impl_start_txn_on(Some(db.into())).await
    }

    async fn impl_start_txn_on(&self, db: Option<Database>) -> Result<Txn> {
        let connection = self.pool.get().await?;
        Txn::new(db, self.config.fetch_size, connection).await
    }

    /// Runs a query on the configured database using a connection from the connection pool,
    /// It doesn't return any [`DetachedRowStream`] as the `run` abstraction discards any stream.
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    ///
    /// Use [`Graph::run`] for cases where you just want a write operation
    ///
    /// use [`Graph::execute`] when you are interested in the result stream
    pub async fn run(&self, q: Query) -> Result<()> {
        self.impl_run_on(self.config.db.clone(), q).await
    }

    /// Runs a query on the provided database using a connection from the connection pool.
    /// It doesn't return any [`DetachedRowStream`] as the `run` abstraction discards any stream.
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    ///
    /// Use [`Graph::run`] for cases where you just want a write operation
    ///
    /// use [`Graph::execute`] when you are interested in the result stream
    pub async fn run_on(&self, db: impl Into<Database>, q: Query) -> Result<()> {
        self.impl_run_on(Some(db.into()), q).await
    }

    async fn impl_run_on(&self, db: Option<Database>, q: Query) -> Result<()> {
        backoff::future::retry_notify(
            self.pool.manager().backoff(),
            || {
                let pool = &self.pool;
                let query = &q;
                let db = db.as_deref();
                async move {
                    let mut connection = pool.get().await.map_err(crate::Error::from)?;
                    query.run_retryable(db, &mut connection).await
                }
            },
            Self::log_retry,
        )
        .await
    }

    /// Executes a query on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute(&self, q: Query) -> Result<DetachedRowStream> {
        self.impl_execute_on(self.config.db.clone(), q).await
    }

    /// Executes a query on the provided database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute_on(&self, db: impl Into<Database>, q: Query) -> Result<DetachedRowStream> {
        self.impl_execute_on(Some(db.into()), q).await
    }

    async fn impl_execute_on(&self, db: Option<Database>, q: Query) -> Result<DetachedRowStream> {
        backoff::future::retry_notify(
            self.pool.manager().backoff(),
            || {
                let pool = &self.pool;
                let fetch_size = self.config.fetch_size;
                let query = &q;
                let db = db.as_deref();
                async move {
                    let connection = pool.get().await.map_err(crate::Error::from)?;
                    query.execute_retryable(db, fetch_size, connection).await
                }
            },
            Self::log_retry,
        )
        .await
    }

    fn log_retry(e: crate::Error, delay: Duration) {
        let level = match delay.as_millis() {
            0..=499 => log::Level::Debug,
            500..=4999 => log::Level::Info,
            _ => log::Level::Warn,
        };
        log::log!(level, "Retrying query in {delay:?} due to error: {e}");
    }
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync>() {}
    assert_send_sync::<Graph>();
};
