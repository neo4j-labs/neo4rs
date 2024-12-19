#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::connection::{Connection, ConnectionInfo, Routing},
    crate::graph::ConnectionPoolManager::Routed,
    crate::routing::{RoundRobinStrategy, RouteBuilder, RoutedConnectionManager},
    log::info,
    std::sync::Arc,
};

use crate::graph::ConnectionPoolManager::Normal;
use crate::pool::ManagedConnection;
use crate::{
    config::{Config, ConfigBuilder, Database, LiveConfig},
    errors::Result,
    pool::{create_pool, ConnectionPool},
    query::Query,
    stream::DetachedRowStream,
    txn::Txn,
    Operation,
};
use backoff::ExponentialBackoff;
use std::time::Duration;

#[derive(Clone)]
enum ConnectionPoolManager {
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    Routed(RoutedConnectionManager),
    Normal(ConnectionPool),
}

impl ConnectionPoolManager {
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    async fn get(&self, operation: Option<Operation>) -> Result<ManagedConnection> {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            Routed(manager) => manager.get(operation).await,
            Normal(pool) => pool.get().await.map_err(crate::Error::from),
        }
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    async fn get(&self) -> Result<ManagedConnection> {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            Routed(manager) => manager.get(operation).await,
            Normal(pool) => pool.get().await.map_err(crate::Error::from),
        }
    }

    fn backoff(&self) -> ExponentialBackoff {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            Routed(manager) => manager.backoff(),
            Normal(pool) => pool.manager().backoff(),
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
    pub async fn connect(config: Config) -> Result<Self> {
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        {
            let info = ConnectionInfo::new(
                &config.uri,
                &config.user,
                &config.password,
                &config.tls_config,
            )?;
            if matches!(info.routing, Routing::Yes(_)) {
                let mut connection = Connection::new(&info).await?;
                let mut builder = RouteBuilder::new(info.routing, vec![]);
                if let Some(db) = config.db.clone() {
                    builder = builder.with_db(db);
                }
                let rt = connection
                    .route(builder.build(connection.version()))
                    .await?;
                connection.reset().await?;
                info!("Connected to routing server, routing table: {:?}", rt);
                let pool = Routed(
                    RoutedConnectionManager::new(
                        &config,
                        Arc::new(rt.clone()),
                        Arc::new(RoundRobinStrategy::new(rt)),
                    )
                    .await?,
                );
                Ok(Graph {
                    config: config.into_live_config(),
                    pool,
                })
            } else {
                let pool = Normal(create_pool(&config).await?);
                Ok(Graph {
                    config: config.into_live_config(),
                    pool,
                })
            }
        }
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            let pool = Normal(create_pool(&config).await?);
            Ok(Graph {
                config: config.into_live_config(),
                pool,
            })
        }
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
        self.impl_start_txn_on(self.config.db.clone(), Operation::Write)
            .await
    }

    /// Starts a new transaction on the configured database specifying the desired operation.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn start_txn_as(&self, operation: Operation) -> Result<Txn> {
        self.impl_start_txn_on(self.config.db.clone(), operation)
            .await
    }

    /// Starts a new transaction on the provided database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    pub async fn start_txn_on(&self, db: impl Into<Database>) -> Result<Txn> {
        self.impl_start_txn_on(Some(db.into()), Operation::Write)
            .await
    }

    #[allow(unused_variables)]
    async fn impl_start_txn_on(&self, db: Option<Database>, operation: Operation) -> Result<Txn> {
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        let connection = self.pool.get(Some(operation)).await?;
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
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
        self.impl_run_on(self.config.db.clone(), q, Operation::Write)
            .await
    }

    /// Runs a READ ONLY query on the configured database using a connection from the connection pool,
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
    pub async fn run_read(&self, q: Query) -> Result<()> {
        self.impl_run_on(self.config.db.clone(), q, Operation::Read)
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
        q: Query,
        operation: Operation,
    ) -> Result<()> {
        self.impl_run_on(Some(db.into()), q, operation).await
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub async fn run_on(&self, db: impl Into<Database>, q: Query) -> Result<()> {
        self.impl_run_on(Some(db.into()), q, Operation::Write).await
    }

    #[allow(unused_variables)]
    async fn impl_run_on(
        &self,
        db: Option<Database>,
        q: Query,
        operation: Operation,
    ) -> Result<()> {
        backoff::future::retry_notify(
            self.pool.backoff(),
            || {
                let pool = &self.pool;
                let query = &q;
                let db = db.as_deref();
                let operation = operation.clone();
                async move {
                    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
                    let mut connection = pool.get(Some(operation)).await?;
                    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
                    let mut connection = pool.get().await?;
                    query.run_retryable(db, &mut connection).await
                }
            },
            Self::log_retry,
        )
        .await
    }

    /// Executes a READ/WRITE query on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute(&self, q: Query) -> Result<DetachedRowStream> {
        self.impl_execute_on(self.config.db.clone(), q, Operation::Write)
            .await
    }

    /// Executes a query READ on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute_read(&self, q: Query) -> Result<DetachedRowStream> {
        self.impl_execute_on(self.config.db.clone(), q, Operation::Read)
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
        q: Query,
        operation: Operation,
    ) -> Result<DetachedRowStream> {
        self.impl_execute_on(Some(db.into()), q, operation).await
    }

    /// Executes a query on the provided database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub async fn execute_on(&self, db: impl Into<Database>, q: Query) -> Result<DetachedRowStream> {
        self.impl_execute_on(Some(db.into()), q, Operation::Write)
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
                let fetch_size = self.config.fetch_size;
                let query = &q;
                let db = db.as_deref();
                #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
                let operation = operation.clone();
                async move {
                    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
                    let connection = pool.get(Some(operation)).await?;
                    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
                    let connection = pool.get().await?;
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
