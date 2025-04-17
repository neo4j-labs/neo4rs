#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::connection::{ConnectionInfo, Routing},
    crate::graph::ConnectionPoolManager::Routed,
    crate::routing::{ClusterRoutingTableProvider, RoutedConnectionManager},
    crate::summary::ResultSummary,
    log::debug,
    std::sync::Arc,
};

use crate::graph::ConnectionPoolManager::Direct;
use crate::pool::ManagedConnection;
use crate::RunResult;
use crate::{
    config::{Config, ConfigBuilder, Database, LiveConfig},
    errors::Result,
    pool::{create_pool, ConnectionPool},
    query::Query,
    stream::DetachedRowStream,
    txn::Txn,
    Operation,
};
use backoff::{Error, ExponentialBackoff};
use std::time::Duration;

#[derive(Clone)]
enum ConnectionPoolManager {
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    Routed(RoutedConnectionManager),
    Direct(ConnectionPool),
}

impl ConnectionPoolManager {
    #[allow(unused_variables)]
    async fn get(&self, operation: Option<Operation>) -> Result<ManagedConnection> {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            Routed(manager) => manager.get(operation).await,
            Direct(pool) => pool.get().await.map_err(crate::Error::from),
        }
    }

    fn backoff(&self) -> ExponentialBackoff {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            Routed(manager) => manager.backoff(),
            Direct(pool) => pool.manager().backoff(),
        }
    }
}

/// A neo4j database abstraction.
/// This type can be cloned and shared across threads, internal resources
/// are reference-counted.
#[derive(Clone)]
pub struct Graph {
    config: LiveConfig,
    pool: ConnectionPoolManager,
}

/// Returns a [`Query`] which provides methods like [`Query::param`] to add parameters to the query
pub fn query(q: &str) -> Query {
    Query::new(q.to_owned())
}

impl Graph {
    /// Connects to the database with configurations provided.
    ///
    /// You can build a config using [`ConfigBuilder::default()`].
    pub fn connect(config: Config) -> Result<Self> {
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        {
            let info = ConnectionInfo::new(
                &config.uri,
                &config.user,
                &config.password,
                &config.tls_config,
            )?;
            if matches!(info.init.routing, Routing::Yes(_)) {
                debug!("Routing enabled, creating a routed connection manager");
                let pool = Routed(RoutedConnectionManager::new(
                    &config,
                    Arc::new(ClusterRoutingTableProvider),
                )?);
                Ok(Graph {
                    config: config.into_live_config(),
                    pool,
                })
            } else {
                let pool = Direct(create_pool(&config)?);
                Ok(Graph {
                    config: config.into_live_config(),
                    pool,
                })
            }
        }
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            let pool = Direct(create_pool(&config)?);
            Ok(Graph {
                config: config.into_live_config(),
                pool,
            })
        }
    }

    /// Connects to the database with default configurations
    pub fn new(
        uri: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self> {
        let config = ConfigBuilder::default()
            .uri(uri)
            .user(user)
            .password(password)
            .build()?;
        Self::connect(config)
    }

    /// Starts a new transaction on the configured database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    pub async fn start_txn(&self) -> Result<Txn> {
        self.impl_start_txn_on(self.config.db.clone(), Operation::Write, &[])
            .await
    }

    /// Starts a new transaction on the configured database specifying the desired operation.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn start_txn_as(
        &self,
        operation: Operation,
        bookmarks: Option<Vec<String>>,
    ) -> Result<Txn> {
        self.impl_start_txn_on(
            self.config.db.clone(),
            operation,
            bookmarks.as_deref().unwrap_or_default(),
        )
        .await
    }

    /// Starts a new transaction on the provided database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    pub async fn start_txn_on(&self, db: impl Into<Database>) -> Result<Txn> {
        self.impl_start_txn_on(Some(db.into()), Operation::Write, &[])
            .await
    }

    #[allow(unused_variables)]
    async fn impl_start_txn_on(
        &self,
        db: Option<Database>,
        operation: Operation,
        bookmarks: &[String],
    ) -> Result<Txn> {
        let connection = self.pool.get(Some(operation.clone())).await?;
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        {
            Txn::new(db, self.config.fetch_size, connection, operation, bookmarks).await
        }
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            Txn::new(db, self.config.fetch_size, connection, operation).await
        }
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
    pub async fn run(&self, q: impl Into<Query>) -> Result<RunResult> {
        self.impl_run_on(self.config.db.clone(), q.into(), Operation::Write)
            .await
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
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn run_on(
        &self,
        db: impl Into<Database>,
        q: impl Into<Query>,
        operation: Operation,
    ) -> Result<ResultSummary> {
        self.impl_run_on(Some(db.into()), q.into(), operation).await
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub async fn run_on(&self, db: impl Into<Database>, q: impl Into<Query>) -> Result<()> {
        self.impl_run_on(Some(db.into()), q.into(), Operation::Write)
            .await
    }

    #[allow(unused_variables)]
    async fn impl_run_on(
        &self,
        db: Option<Database>,
        q: Query,
        operation: Operation,
    ) -> Result<RunResult> {
        let is_read = operation.is_read();
        let result = backoff::future::retry_notify(
            self.pool.backoff(),
            || {
                let pool = &self.pool;
                let mut query = q.clone();
                let operation = operation.clone();
                if let Some(db) = db.as_deref() {
                    query = query.extra("db", db);
                }
                query = query.extra("mode", if is_read { "r" } else { "w" });
                async move {
                    let mut connection =
                        pool.get(Some(operation)).await.map_err(Error::Permanent)?; // an error when retrieving a connection is considered permanent
                    query.run_retryable(&mut connection).await
                }
            },
            Self::log_retry,
        )
        .await;

        match result {
            Ok(result) => {
                #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
                {
                    if let Some(bookmark) = result.bookmark.as_deref() {
                        match &self.pool {
                            Routed(routed) => {
                                routed.add_bookmark(bookmark).await;
                            }
                            Direct(_) => {}
                        }
                    } else if is_read {
                        match &self.pool {
                            Routed(routed) => {
                                debug!("No bookmark received after a read operation, discarding all bookmarks");
                                routed.clear_bookmarks().await;
                            }
                            Direct(_) => {}
                        }
                    }
                }
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    /// Executes a READ/WRITE query on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute(&self, q: impl Into<Query>) -> Result<DetachedRowStream> {
        self.impl_execute_on(self.config.db.clone(), q.into(), Operation::Write)
            .await
    }

    /// Executes a query READ on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute_read(&self, q: impl Into<Query>) -> Result<DetachedRowStream> {
        self.impl_execute_on(self.config.db.clone(), q.into(), Operation::Read)
            .await
    }

    /// Executes a query on the provided database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn execute_on(
        &self,
        db: impl Into<Database>,
        q: impl Into<Query>,
        operation: Operation,
    ) -> Result<DetachedRowStream> {
        self.impl_execute_on(Some(db.into()), q.into(), operation)
            .await
    }

    /// Executes a query on the provided database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub async fn execute_on(
        &self,
        db: impl Into<Database>,
        q: impl Into<Query>,
    ) -> Result<DetachedRowStream> {
        self.impl_execute_on(Some(db.into()), q.into(), Operation::Write)
            .await
    }

    #[allow(unused_variables)]
    async fn impl_execute_on(
        &self,
        db: Option<Database>,
        q: Query,
        operation: Operation,
    ) -> Result<DetachedRowStream> {
        backoff::future::retry_notify(
            self.pool.backoff(),
            || {
                let pool = &self.pool;
                let mut query = q.clone();
                let operation = operation.clone();
                let fetch_size = self.config.fetch_size;
                if let Some(db) = db.as_deref() {
                    query = query.extra("db", db);
                }
                let operation = operation.clone();
                query = query.param("mode", if operation.is_read() { "r" } else { "w" });
                async move {
                    let connection = pool.get(Some(operation)).await.map_err(Error::Permanent)?; // an error when retrieving a connection is considered permanent
                    query.execute_retryable(fetch_size, connection).await
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
