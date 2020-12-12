use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::query::*;
use crate::stream::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Txn {
    connection: Arc<Mutex<ManagedConnection>>,
}

impl Txn {
    pub async fn new(mut connection: ManagedConnection) -> Result<Self> {
        let begin = BoltRequest::begin();
        match connection.send_recv(begin).await? {
            BoltResponse::SuccessMessage(_) => Ok(Txn {
                connection: Arc::new(Mutex::new(connection)),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn run_queries(&self, queries: Vec<Query>) -> Result<()> {
        for query in queries.into_iter() {
            self.run(query).await?;
        }
        Ok(())
    }

    pub async fn run(&self, q: Query) -> Result<()> {
        q.run(self.connection.clone()).await
    }

    pub async fn execute(&self, q: Query) -> Result<RowStream> {
        q.execute(self.connection.clone()).await
    }

    pub async fn commit(self) -> Result<()> {
        let commit = BoltRequest::commit();
        match self.connection.lock().await.send_recv(commit).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            msg => {
                eprintln!("unexpected message {:?}", msg);
                Err(Error::UnexpectedMessage)
            }
        }
    }

    pub async fn rollback(self) -> Result<()> {
        let rollback = BoltRequest::rollback();
        match self.connection.lock().await.send_recv(rollback).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
