use crate::connection::Connection;
use crate::errors::*;
use crate::messages::*;
use crate::query::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Txn {
    connection: Arc<Mutex<Connection>>,
}

impl Txn {
    pub async fn new(connection: Arc<Mutex<Connection>>) -> Result<Self> {
        let begin = BoltRequest::begin();
        match connection.lock().await.send_recv(begin).await? {
            BoltResponse::SuccessMessage(_) => Ok(Txn {
                connection: connection.clone(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn commit(&self) -> Result<()> {
        let mut connection = self.connection.lock().await;
        match connection.send_recv(BoltRequest::commit()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn run_queries(&self, queries: Vec<Query>) -> Result<()> {
        for query in queries.into_iter() {
            query.run(self.connection.clone()).await?;
        }
        Ok(())
    }

    pub async fn rollback(&self) -> Result<()> {
        let mut connection = self.connection.lock().await;
        match connection.send_recv(BoltRequest::rollback()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
