#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
use crate::messages::{BoltRequest, BoltResponse};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::bolt::{Begin, Commit, Rollback, Summary},
    crate::bookmarks::Bookmark,
    log::debug,
};

use crate::config::ImpersonateUser;
use crate::{
    config::Database, errors::Result, pool::ManagedConnection, query::Query, stream::RowStream,
    Operation, RunResult,
};

/// A handle which is used to control a transaction, created as a result of [`crate::Graph::start_txn`]
///
/// When a transaction is started, a dedicated connection is reserved and moved into the handle which
/// will be released to the connection pool when the [`Txn`] handle is dropped.
pub struct Txn {
    db: Option<Database>,
    fetch_size: usize,
    connection: ManagedConnection,
    operation: Operation,
    imp_user: Option<ImpersonateUser>,
    #[allow(dead_code)]
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
    ) -> Result<Self> {
        let begin = BoltRequest::begin(db.as_deref());
        match connection.send_recv(begin).await? {
            BoltResponse::Success(_) => Ok(Txn {
                db,
                fetch_size,
                connection,
                operation,
                bookmark: None,
                imp_user,
            }),
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
    ) -> Result<Self> {
        debug!("Starting transaction with bookmarks: {:?}", bookmarks);
        let begin = Begin::builder(db.as_deref())
            .with_bookmarks(bookmarks.to_vec())
            .build(connection.version());
        match connection.send_recv_as(begin).await? {
            Summary::Success(response) => Ok(Txn {
                db: response.metadata.db.or(db),
                fetch_size,
                connection,
                operation,
                bookmark: None,
                imp_user,
            }),
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
            BoltResponse::Success(_) => Ok(()),
            msg => Err(msg.into_error("COMMIT")),
        }
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    pub async fn commit(mut self) -> Result<Option<String>> {
        match self.connection.send_recv_as(Commit).await? {
            Summary::Success(resp) => {
                self.save_bookmark_state(&resp.metadata);
                Ok(self.bookmark)
            }
            msg => Err(msg.into_error("COMMIT")),
        }
    }

    /// rollback/abort the current transaction
    pub async fn rollback(mut self) -> Result<()> {
        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            let rollback = BoltRequest::rollback();
            match self.connection.send_recv(rollback).await? {
                BoltResponse::Success(_) => Ok(()),
                msg => Err(msg.into_error("ROLLBACK")),
            }
        }

        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        {
            match self.connection.send_recv_as(Rollback).await? {
                Summary::Success(_) => Ok(()),
                msg => Err(msg.into_error("ROLLBACK")),
            }
        }
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
    use crate::{pool::ManagedConnection, Txn};

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
