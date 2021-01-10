use crate::config::Config;
use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::stream::*;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Abstracts a cypher query that is sent to neo4j server.
#[derive(Clone)]
pub struct Query {
    query: String,
    params: BoltMap,
}

impl Query {
    pub fn new(query: String) -> Self {
        Query {
            query,
            params: BoltMap::default(),
        }
    }

    pub fn param<T: std::convert::Into<BoltType>>(mut self, key: &str, value: T) -> Self {
        self.params.put(key.into(), value.into());
        self
    }

    pub(crate) async fn run(
        self,
        config: &Config,
        connection: Arc<Mutex<ManagedConnection>>,
    ) -> Result<()> {
        let run = BoltRequest::run(&config.db, &self.query, self.params.clone());
        let mut connection = connection.lock().await;
        match connection.send_recv(run).await? {
            BoltResponse::SuccessMessage(_) => {
                match connection.send_recv(BoltRequest::discard()).await? {
                    BoltResponse::SuccessMessage(_) => Ok(()),
                    msg => Err(unexpected(msg, "DISCARD")),
                }
            }
            msg => Err(unexpected(msg, "RUN")),
        }
    }

    pub(crate) async fn execute(
        self,
        config: &Config,
        connection: Arc<Mutex<ManagedConnection>>,
    ) -> Result<RowStream> {
        let run = BoltRequest::run(&config.db, &self.query, self.params);
        match connection.lock().await.send_recv(run).await {
            Ok(BoltResponse::SuccessMessage(success)) => {
                let fields: BoltList = success.get("fields").unwrap_or_else(BoltList::new);
                let qid: i64 = success.get("qid").unwrap_or(-1);
                Ok(RowStream::new(
                    qid,
                    fields,
                    config.fetch_size,
                    connection.clone(),
                ))
            }
            msg => Err(unexpected(msg, "RUN")),
        }
    }
}
