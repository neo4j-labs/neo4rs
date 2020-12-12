use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::row::*;
use crate::types::*;
use tokio::sync::mpsc;

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

    pub async fn run(self, connection: &mut ManagedConnection) -> Result<()> {
        //TODO: reset connection
        let run = BoltRequest::run(&self.query, self.params.clone());
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

    pub async fn execute(self, mut connection: ManagedConnection) -> Result<mpsc::Receiver<Row>> {
        let (sender, receiver) = mpsc::channel(100); //TODO: configure buffer size

        tokio::spawn(async move {
            let run = BoltRequest::run(&self.query, self.params);
            match connection.send_recv(run).await {
                Ok(BoltResponse::SuccessMessage(success)) => {
                    let mut has_more_records = true;
                    let qid: i64 = success.get("qid").unwrap_or(-1);
                    let fields: BoltList = success.get("fields").unwrap_or(BoltList::new());
                    while has_more_records {
                        let pull = BoltRequest::pull(qid);
                        match connection.send(pull).await {
                            Ok(()) => loop {
                                match connection.recv().await {
                                    Ok(BoltResponse::SuccessMessage(s)) => {
                                        has_more_records = s.get("has_more").unwrap_or(false);
                                        break;
                                    }
                                    Ok(BoltResponse::RecordMessage(record)) => {
                                        let row = Row::new(fields.clone(), record.data);
                                        sender.send(row).await.unwrap(); //TODO: fix unwrap
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
        Ok(receiver)
    }
}
