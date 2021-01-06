use crate::config::Config;
use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::query::*;
use crate::stream::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A handle which is used to control a transaction, created as a result of [`Graph::start_txn`]
///
/// When a transation is started, a dedicated connection is resered and moved into the handle which
/// will be released to the connection pool when the [`Txn`] handle is dropped.
pub struct Txn {
    config: Config,
    connection: Arc<Mutex<ManagedConnection>>,
}

impl Txn {
    pub(crate) async fn new(config: Config, mut connection: ManagedConnection) -> Result<Self> {
        let begin = BoltRequest::begin();
        match connection.send_recv(begin).await? {
            BoltResponse::SuccessMessage(_) => Ok(Txn {
                config,
                connection: Arc::new(Mutex::new(connection)),
            }),
            msg => Err(unexpected(msg, "BEGIN")),
        }
    }

    /// Runs multiple queries one after the other in the same connection
    pub async fn run_queries(&self, queries: Vec<Query>) -> Result<()> {
        for query in queries.into_iter() {
            self.run(query).await?;
        }
        Ok(())
    }

    /// Runs a single query and discards the stream.
    pub async fn run(&self, q: Query) -> Result<()> {
        q.run(&self.config, self.connection.clone()).await
    }

    /// Executes a query and returns a [`RowStream`]
    pub async fn execute(&self, q: Query) -> Result<RowStream> {
        q.execute(&self.config, self.connection.clone()).await
    }

    /// Commits the transaction in progress
    pub async fn commit(self) -> Result<()> {
        let commit = BoltRequest::commit();
        match self.connection.lock().await.send_recv(commit).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            msg => Err(unexpected(msg, "COMMIT")),
        }
    }

    /// rollback/abort the current transaction
    pub async fn rollback(self) -> Result<()> {
        let rollback = BoltRequest::rollback();
        match self.connection.lock().await.send_recv(rollback).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            msg => Err(unexpected(msg, "ROLLBACK")),
        }
    }
}
