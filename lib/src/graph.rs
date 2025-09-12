#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::connection::{ConnectionInfo, Routing},
    crate::routing::{ClusterRoutingTableProvider, RoutedConnectionManager},
    crate::summary::ResultSummary,
    log::debug,
    std::sync::Arc,
};

use crate::config::ImpersonateUser;
use crate::pool::ManagedConnection;
use crate::query::RetryableQuery;
use crate::retry::Retry;
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
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{
    session::{Session, SessionConfig},
    Error,
};
use backon::{ExponentialBuilder, RetryableWithContext};
use std::time::Duration;

#[derive(Clone)]
pub(crate) enum ConnectionPoolManager {
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    Routed(RoutedConnectionManager),
    Direct(ConnectionPool),
}

impl ConnectionPoolManager {
    #[allow(unused_variables)]
    pub(crate) async fn get(
        &self,
        operation: Option<Operation>,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<ManagedConnection> {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            ConnectionPoolManager::Routed(manager) => {
                manager.get(operation, db, imp_user, bookmarks).await
            }
            ConnectionPoolManager::Direct(pool) => pool.get().await.map_err(crate::Error::from),
        }
    }

    fn backoff(&self) -> ExponentialBuilder {
        match self {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            ConnectionPoolManager::Routed(manager) => manager.backoff(),
            ConnectionPoolManager::Direct(pool) => pool.manager().backoff(),
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
                let pool = ConnectionPoolManager::Routed(RoutedConnectionManager::new(
                    &config,
                    Arc::new(ClusterRoutingTableProvider::new(config.clone())),
                )?);
                Ok(Graph {
                    config: config.into_live_config(),
                    pool,
                })
            } else {
                let pool = ConnectionPoolManager::Direct(create_pool(&config)?);
                Ok(Graph {
                    config: config.into_live_config(),
                    pool,
                })
            }
        }
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            let pool = ConnectionPoolManager::Direct(create_pool(&config)?);
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
        self.impl_start_txn_on(
            self.config.db.clone(),
            Operation::Write,
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
        )
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
            self.config.imp_user.clone(),
            bookmarks.as_deref().unwrap_or_default(),
            Some(self.config.fetch_size),
        )
        .await
    }

    /// Starts a new transaction on the provided database.
    /// All queries that needs to be run/executed within the transaction
    /// should be executed using either [`Txn::run`] or [`Txn::execute`]
    ///
    /// Transactions will not be automatically retried on any failure.
    pub async fn start_txn_on(&self, db: impl Into<Database>) -> Result<Txn> {
        self.impl_start_txn_on(
            Some(db.into()),
            Operation::Write,
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
        )
        .await
    }

    #[allow(unused_variables)]
    pub(crate) async fn impl_start_txn_on(
        &self,
        db: Option<Database>,
        operation: Operation,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
        fetch_size: Option<usize>,
    ) -> Result<Txn> {
        let connection = self
            .pool
            .get(Some(operation), db.clone(), imp_user.clone(), bookmarks)
            .await?;
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        {
            Txn::new(
                db,
                fetch_size.unwrap_or(self.config.fetch_size),
                connection,
                operation,
                imp_user,
                bookmarks,
            )
            .await
        }
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            Txn::new(db, self.config.fetch_size, connection, operation, imp_user).await
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
        self.impl_run_on(
            self.config.db.clone(),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
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
    ) -> Result<ResultSummary> {
        self.impl_run_on(
            Some(db.into()),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
        .await
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub async fn run_on(&self, db: impl Into<Database>, q: impl Into<Query>) -> Result<()> {
        self.impl_run_on(
            Some(db.into()),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
        .await
    }

    #[allow(unused_variables)]
    pub(crate) async fn impl_run_on(
        &self,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
        fetch_size: Option<usize>,
        query: Query,
    ) -> Result<RunResult> {
        let query = query.into_retryable(
            db,
            imp_user,
            Operation::Write,
            &self.pool,
            fetch_size.or(Some(self.config.fetch_size)),
            bookmarks,
        );

        let (query, result) = RetryableQuery::retry_run
            .retry(self.pool.backoff())
            .sleep(tokio::time::sleep)
            .context(query)
            .when(|e| matches!(e, Retry::Yes(_)))
            .notify(Self::log_retry)
            .await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(e.into_inner()),
        }
    }

    /// Executes a READ|WRITE query on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute(&self, q: impl Into<Query>) -> Result<DetachedRowStream> {
        self.impl_execute_on(
            Operation::Write,
            self.config.db.clone(),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
        .await
    }

    /// Executes a query READ on the configured database and returns a [`DetachedRowStream`]
    ///
    /// This operation retires the query on certain failures.
    /// All errors with the `Transient` error class as well as a few other error classes are considered retryable.
    /// This includes errors during a leader election or when the transaction resources on the server (memory, handles, ...) are exhausted.
    /// Retries happen with an exponential backoff until a retry delay exceeds 60s, at which point the query fails with the last error as it would without any retry.
    pub async fn execute_read(&self, q: impl Into<Query>) -> Result<DetachedRowStream> {
        self.impl_execute_on(
            Operation::Read,
            self.config.db.clone(),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
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
        operation: Operation,
        db: impl Into<Database>,
        q: impl Into<Query>,
    ) -> Result<DetachedRowStream> {
        self.impl_execute_on(
            operation,
            Some(db.into()),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
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
        self.impl_execute_on(
            Operation::Write,
            Some(db.into()),
            self.config.imp_user.clone(),
            &[],
            Some(self.config.fetch_size),
            q.into(),
        )
        .await
    }

    #[allow(unused_variables)]
    pub(crate) async fn impl_execute_on(
        &self,
        operation: Operation,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
        fetch_size: Option<usize>,
        query: Query,
    ) -> Result<DetachedRowStream> {
        let query = query.into_retryable(
            db,
            imp_user,
            operation,
            &self.pool,
            fetch_size.or(Some(self.config.fetch_size)),
            bookmarks,
        );

        let (query, result) = RetryableQuery::retry_execute
            .retry(self.pool.backoff())
            .sleep(tokio::time::sleep)
            .context(query)
            .when(|e| matches!(e, Retry::Yes(_)))
            .notify(Self::log_retry)
            .await;

        result.map_err(Retry::into_inner)
    }

    fn log_retry(e: &Retry<crate::Error>, delay: Duration) {
        let level = match delay.as_millis() {
            0..=499 => log::Level::Debug,
            500..=4999 => log::Level::Info,
            _ => log::Level::Warn,
        };
        log::log!(level, "Retrying query in {delay:?} due to error: {e}");
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub fn with_session(&self, config: Option<SessionConfig>) -> Session {
        Session::new(config.unwrap_or_default(), Arc::new(self.clone()))
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn get_default_db(
        &self,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<Option<Database>, Error> {
        match &self.pool {
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            ConnectionPoolManager::Routed(routed) => {
                routed.get_default_db(imp_user.clone(), bookmarks).await
            }
            ConnectionPoolManager::Direct(_) => self
                .config
                .db
                .clone()
                .map_or(Ok(Some("".into())), |db| Ok(Some(db))),
        }
    }
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync>() {}
    assert_send_sync::<Graph>();
};
