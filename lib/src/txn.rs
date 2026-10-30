#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
use crate::messages::{BoltRequest, BoltResponse};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::bolt::{Begin, Commit, Summary},
    crate::bookmarks::Bookmark,
    log::debug,
};

use crate::config::ImpersonateUser;
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
use crate::{BoltMap, BoltType};
use crate::{
    Operation, RunResult,
    config::Database,
    errors::Result,
    pool::{ConnectionManager, DiscardOnAbandon, ManagedConnection},
    query::Query,
    stream::RowStream,
};
use std::time::Duration;

/// Options sent to Neo4j when an explicit transaction is started.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransactionConfig {
    tx_timeout: Option<Duration>,
    tx_metadata: Option<Vec<(String, String)>>,
}

impl TransactionConfig {
    /// Sets the server-side transaction timeout.
    ///
    /// A timeout of zero disables the transaction timeout.
    /// Positive sub-millisecond timeouts are rounded up to one millisecond.
    /// Timeouts larger than the Bolt protocol supports return an error when the transaction starts.
    pub fn with_tx_timeout(mut self, tx_timeout: Duration) -> Self {
        self.tx_timeout = Some(tx_timeout);
        self
    }

    /// Sets metadata attached to the transaction as string key/value pairs.
    pub fn with_tx_metadata(mut self, tx_metadata: Vec<(String, String)>) -> Self {
        self.tx_metadata = Some(tx_metadata);
        self
    }

    fn tx_timeout_millis(timeout: Duration) -> Result<i64> {
        let millis = timeout.as_millis();
        let millis = millis + u128::from(timeout.subsec_nanos() % 1_000_000 != 0);
        i64::try_from(millis)
            .map_err(|error| crate::errors::Error::IntegerOverflow("tx_timeout", error))
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    fn into_bolt_map(self) -> Result<BoltMap> {
        let mut extra = BoltMap::new();
        if let Some(timeout) = self.tx_timeout {
            extra.put(
                "tx_timeout".into(),
                Self::tx_timeout_millis(timeout)?.into(),
            );
        }
        if let Some(tx_metadata) = self.tx_metadata {
            let metadata = tx_metadata
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect();
            extra.put("tx_metadata".into(), BoltType::Map(metadata));
        }
        Ok(extra)
    }
}

/// A handle which is used to control a transaction, created as a result of [`crate::Graph::start_txn`]
///
/// When a transaction is started, a dedicated connection is reserved and moved into the handle.
/// If the handle is dropped before the transaction is committed or rolled back, the connection is
/// discarded so the server can roll back the transaction. Completed transactions return their
/// connections to the pool when the handle is dropped.
pub struct Txn {
    db: Option<Database>,
    fetch_size: usize,
    connection: DiscardOnAbandon<ConnectionManager>,
    operation: Operation,
    imp_user: Option<ImpersonateUser>,
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    bookmark: Option<String>,
}

impl Txn {
    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub(crate) async fn new(
        db: Option<Database>,
        fetch_size: usize,
        mut connection: ManagedConnection,
        operation: Operation,
        imp_user: Option<ImpersonateUser>,
        config: TransactionConfig,
    ) -> Result<Self> {
        let begin = BoltRequest::begin(db.as_deref(), config.into_bolt_map()?);
        match connection.send_recv(begin).await? {
            BoltResponse::Success(_) => {
                let connection = DiscardOnAbandon::new(connection);
                Ok(Txn {
                    db,
                    fetch_size,
                    connection,
                    operation,
                    imp_user,
                })
            }
            msg => Err(msg.into_error("BEGIN")),
        }
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub(crate) async fn new(
        db: Option<Database>,
        fetch_size: usize,
        mut connection: ManagedConnection,
        operation: Operation,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
        config: TransactionConfig,
    ) -> Result<Self> {
        debug!("Starting transaction with bookmarks: {:?}", bookmarks);
        let mut builder = Begin::builder(db.as_deref()).with_bookmarks(bookmarks.to_vec());
        if let Some(timeout) = config.tx_timeout {
            builder = builder.with_tx_timeout(TransactionConfig::tx_timeout_millis(timeout)?);
        }
        if let Some(tx_metadata) = config.tx_metadata {
            builder = builder.with_tx_metadata(tx_metadata);
        }
        let begin = builder.build(connection.version());
        match connection.send_recv_as(begin).await? {
            Summary::Success(response) => {
                let connection = DiscardOnAbandon::new(connection);
                Ok(Txn {
                    db: response.metadata.db.or(db),
                    fetch_size,
                    connection,
                    operation,
                    bookmark: None,
                    imp_user,
                })
            }
            Summary::Ignored => Err(crate::errors::Error::Ignored("Failed to start transaction")),
            Summary::Failure(failure) => Err(failure.into_error()),
        }
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    /// Runs multiple queries one after the other in the same connection,
    /// merging all counters from each result summary.
    pub async fn run_queries<Q: Into<Query>>(
        &mut self,
        queries: impl IntoIterator<Item = Q>,
    ) -> Result<crate::summary::Counters> {
        let mut counters = crate::summary::Counters::default();
        for query in queries {
            let q = query.into().imp_user(self.imp_user.clone());
            let summary = self.run(q).await?;
            counters += summary.stats();
            self.save_bookmark_state(&summary);
        }
        Ok(counters)
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    /// Runs multiple queries one after the other in the same connection
    pub async fn run_queries<Q: Into<Query>>(
        &mut self,
        queries: impl IntoIterator<Item = Q>,
    ) -> Result<()> {
        for query in queries {
            let q = query.into().imp_user(self.imp_user.clone());
            self.run(q).await?;
        }
        Ok(())
    }

    /// Runs a single query and discards the stream.
    pub async fn run(&mut self, q: impl Into<Query>) -> Result<RunResult> {
        let mut query = q.into();
        if let Some(db) = self.db.as_ref() {
            query = query.extra("db", db.to_string());
        }
        query = query.extra(
            "mode",
            match self.operation {
                Operation::Read => "r",
                Operation::Write => "w",
            },
        );
        match query.run(&mut self.connection).await {
            Ok(result) => {
                #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
                self.save_bookmark_state(&result);
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    /// Executes a query and returns a [`RowStream`]
    pub async fn execute(&mut self, q: impl Into<Query>) -> Result<RowStream> {
        let mut query = q.into();
        if let Some(db) = self.db.as_ref() {
            query = query.extra("db", db.to_string());
        }
        query = query.extra(
            "mode",
            match self.operation {
                Operation::Read => "r",
                Operation::Write => "w",
            },
        );
        query
            .execute_mut(self.fetch_size, &mut self.connection)
            .await
    }

    /// Commits the transaction in progress
    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    pub async fn commit(mut self) -> Result<()> {
        let commit = BoltRequest::commit();
        match self.connection.send_recv(commit).await? {
            BoltResponse::Success(_) => {
                self.connection.close_transaction();
                Ok(())
            }
            msg => Err(msg.into_error("COMMIT")),
        }
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn commit(mut self) -> Result<Option<String>> {
        match self.connection.send_recv_as(Commit).await? {
            Summary::Success(resp) => {
                self.save_bookmark_state(&resp.metadata);
                self.connection.close_transaction();
                Ok(self.bookmark)
            }
            msg => Err(msg.into_error("COMMIT")),
        }
    }

    /// Rolls back the current transaction.
    ///
    /// If the server ignores the rollback because the transaction is in a failed state,
    /// the connection is reset before it can be reused.
    pub async fn rollback(mut self) -> Result<()> {
        self.connection.rollback_transaction().await?;
        self.connection.close_transaction();
        Ok(())
    }

    pub fn handle(&mut self) -> &mut impl TransactionHandle {
        self
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub fn last_bookmark(&self) -> Option<&str> {
        self.bookmark.as_deref()
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    fn save_bookmark_state(&mut self, summary: &impl Bookmark) {
        if let Some(bookmark) = summary.get_bookmark() {
            self.bookmark = Some(bookmark.to_string());
        } else {
            self.bookmark = None;
        }
    }
}

const _: () = {
    const fn assert_send_sync<T: ?Sized + Send + Sync>() {}
    assert_send_sync::<Txn>();
};

pub trait TransactionHandle: private::Handle {}

impl TransactionHandle for Txn {}
impl TransactionHandle for ManagedConnection {}
impl<T: TransactionHandle> TransactionHandle for &mut T {}

pub(crate) mod private {
    use crate::{Txn, pool::ManagedConnection};

    pub trait Handle {
        fn connection(&mut self) -> &mut ManagedConnection;
    }

    impl Handle for Txn {
        fn connection(&mut self) -> &mut ManagedConnection {
            &mut self.connection
        }
    }

    impl Handle for ManagedConnection {
        fn connection(&mut self) -> &mut ManagedConnection {
            self
        }
    }

    impl<T: Handle> Handle for &mut T {
        fn connection(&mut self) -> &mut ManagedConnection {
            (**self).connection()
        }
    }
}

#[cfg(test)]
mod transaction_config_tests {
    use super::TransactionConfig;
    use crate::errors::Error;
    use std::time::Duration;

    #[test]
    fn options_are_optional_and_preserve_values() {
        let default = TransactionConfig::default();
        assert_eq!(default.tx_timeout, None);
        assert_eq!(default.tx_metadata, None);

        let configured = TransactionConfig::default()
            .with_tx_timeout(Duration::from_secs(15))
            .with_tx_metadata(vec![
                ("operation".into(), "probe".into()),
                ("request_id".into(), "request-1".into()),
            ]);
        assert_eq!(configured.tx_timeout, Some(Duration::from_secs(15)));
        assert_eq!(
            configured.tx_metadata,
            Some(vec![
                ("operation".into(), "probe".into()),
                ("request_id".into(), "request-1".into()),
            ])
        );
    }

    #[test]
    fn timeout_is_rounded_up_to_milliseconds() {
        assert_eq!(
            TransactionConfig::tx_timeout_millis(Duration::ZERO).unwrap(),
            0
        );
        assert_eq!(
            TransactionConfig::tx_timeout_millis(Duration::from_nanos(1)).unwrap(),
            1
        );
        assert_eq!(
            TransactionConfig::tx_timeout_millis(Duration::from_micros(1_500)).unwrap(),
            2
        );
        assert_eq!(
            TransactionConfig::tx_timeout_millis(Duration::from_millis(15_000)).unwrap(),
            15_000
        );
    }

    #[test]
    fn timeout_outside_bolt_integer_range_returns_an_error() {
        let timeout = Duration::from_millis(i64::MAX as u64)
            .checked_add(Duration::from_millis(1))
            .unwrap();
        assert!(matches!(
            TransactionConfig::tx_timeout_millis(timeout),
            Err(Error::IntegerOverflow("tx_timeout", _))
        ));
    }
}
