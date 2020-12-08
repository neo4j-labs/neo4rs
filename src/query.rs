use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::row::*;
use crate::types::*;
use std::cell::RefCell;

#[derive(Debug)]
pub struct QueryBuilder {
    query: String,
    connections: bb8::Pool<ConnectionManager>,
    params: RefCell<BoltMap>,
}

impl QueryBuilder {
    pub fn new(query: String, connections: bb8::Pool<ConnectionManager>) -> Self {
        QueryBuilder {
            query,
            connections,
            params: RefCell::new(BoltMap::new()),
        }
    }

    pub fn param<T: std::convert::Into<BoltType>>(self, key: &str, value: T) -> Self {
        self.params.borrow_mut().put(key.into(), value.into());
        self
    }

    pub async fn run(self) -> Result<()> {
        //TODO: reset connection
        let run = BoltRequest::run(&self.query, self.params.borrow().clone());
        let mut connection = self.connections.get().await?;
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

    pub async fn execute(self) -> Result<tokio::sync::mpsc::Receiver<Row>> {
        //TODO: reset connection
        let (tx, rx) = tokio::sync::mpsc::channel(100); //TODO: configure buffer size
        let query = self.query.clone();
        let params = self.params.borrow().clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            let mut connection = connections.get().await.unwrap();
            match connection.send_recv(BoltRequest::run(&query, params)).await {
                Ok(BoltResponse::SuccessMessage(success)) => {
                    let qid: i64 = success.get("qid").unwrap_or(-1);
                    let fields: BoltList = success.get("fields").unwrap_or(BoltList::new());
                    let mut has_more = true;
                    while has_more {
                        match connection.send(BoltRequest::pull(qid)).await {
                            Ok(()) => loop {
                                match connection.recv().await {
                                    Ok(BoltResponse::SuccessMessage(s)) => {
                                        has_more = s.get("has_more").unwrap_or(false);
                                        break;
                                    }
                                    Ok(BoltResponse::RecordMessage(record)) => {
                                        let row = Row::new(fields.clone(), record.data);
                                        tx.send(row).await.unwrap(); //TODO: fix unwrap
                                    }
                                    Ok(msg) => {
                                        eprintln!("Got unexpected message: {:?}", msg);
                                        break;
                                    }
                                    Err(msg) => {
                                        eprintln!("Got error while streaming: {:?}", msg);
                                        break;
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("error executing query {:?}", e);
                            }
                        }
                    }
                }
                Ok(BoltResponse::FailureMessage(msg)) => {
                    eprintln!("error executing query {:?}", msg);
                }
                msg => {
                    eprintln!("unexpected message received: {:?}", msg);
                }
            };
        });
        Ok(rx)
    }
}
