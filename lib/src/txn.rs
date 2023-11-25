use crate::{
    config::Database,
    errors::{unexpected, Result},
    messages::{BoltRequest, BoltResponse},
    pool::ManagedConnection,
    query::Query,
    stream::RowStream,
};

/// A handle which is used to control a transaction, created as a result of [`crate::Graph::start_txn`]
///
/// When a transation is started, a dedicated connection is resered and moved into the handle which
/// will be released to the connection pool when the [`Txn`] handle is dropped.
pub struct Txn {
    db: Database,
    fetch_size: usize,
    connection: ManagedConnection,
}

impl Txn {
    pub(crate) async fn new(
        db: Database,
        fetch_size: usize,
        mut connection: ManagedConnection,
    ) -> Result<Self> {
        let begin = BoltRequest::begin(&db);
        match connection.send_recv(begin).await? {
            BoltResponse::Success(_) => Ok(Txn {
                db,
                fetch_size,
                connection,
            }),
            msg => Err(unexpected(msg, "BEGIN")),
        }
    }

    /// Runs multiple queries one after the other in the same connection
    pub async fn run_queries<Q: Into<Query>>(
        &mut self,
        queries: impl IntoIterator<Item = Q>,
    ) -> Result<()> {
        for query in queries {
            self.run(query.into()).await?;
        }
        Ok(())
    }

    /// Runs a single query and discards the stream.
    pub async fn run(&mut self, q: Query) -> Result<()> {
        q.run(&self.db, &mut self.connection).await
    }

    /// Executes a query and returns a [`RowStream`]
    pub async fn execute(&mut self, q: Query) -> Result<RowStream> {
        q.execute_mut(&self.db, self.fetch_size, &mut self.connection)
            .await
    }

    /// Commits the transaction in progress
    pub async fn commit(mut self) -> Result<()> {
        let commit = BoltRequest::commit();
        match self.connection.send_recv(commit).await? {
            BoltResponse::Success(_) => Ok(()),
            msg => Err(unexpected(msg, "COMMIT")),
        }
    }

    /// rollback/abort the current transaction
    pub async fn rollback(mut self) -> Result<()> {
        let rollback = BoltRequest::rollback();
        match self.connection.send_recv(rollback).await? {
            BoltResponse::Success(_) => Ok(()),
            msg => Err(unexpected(msg, "ROLLBACK")),
        }
    }

    pub fn handle(&mut self) -> &mut impl TransactionHandle {
        self
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
