use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::stream::*;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Query {
    query: String,
    params: BoltMap,
}

impl Query {
    pub fn new(query: String) -> Self {
        Query {
            query,
            params: BoltMap::new(),
        }
    }

    pub fn param<T: std::convert::Into<BoltType>>(mut self, key: &str, value: T) -> Self {
        self.params.put(key.into(), value.into());
        self
    }

    pub async fn run(self, connection: Arc<Mutex<ManagedConnection>>) -> Result<()> {
        let run = BoltRequest::run(&self.query, self.params.clone());
        let mut connection = connection.lock().await;
        match connection.send_recv(run).await? {
            BoltResponse::SuccessMessage(_) => {
                match connection.send_recv(BoltRequest::discard()).await? {
                    BoltResponse::SuccessMessage(_) => Ok(()),
                    _ => Err(Error::UnexpectedMessage),
                }
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn execute(self, connection: Arc<Mutex<ManagedConnection>>) -> Result<RowStream> {
        let run = BoltRequest::run(&self.query, self.params);
        match connection.lock().await.send_recv(run).await {
            Ok(BoltResponse::SuccessMessage(success)) => {
                let fields: BoltList = success.get("fields").unwrap_or(BoltList::new());
                let qid: i64 = success.get("qid").unwrap_or(-1);
                Ok(RowStream::new(qid, fields, connection.clone()))
            }
            msg => {
                eprintln!("unexpected message received: {:?}", msg);
                Err(Error::QueryError)
            }
        }
    }
}
