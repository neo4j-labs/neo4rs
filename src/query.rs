use crate::connection::*;
use crate::errors::*;
use crate::messages::*;
use crate::row::*;
use crate::types::*;
use async_stream::stream;
use futures::stream::Stream;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct QueryBuilder {
    query: String,
    connection: Rc<RefCell<Connection>>,
    params: RefCell<BoltMap>,
}

impl QueryBuilder {
    pub fn new(query: String, connection: Rc<RefCell<Connection>>) -> Self {
        QueryBuilder {
            query,
            connection,
            params: RefCell::new(BoltMap::new()),
        }
    }

    pub fn param<T: std::convert::Into<BoltType>>(self, key: &str, value: T) -> Self {
        self.params.borrow_mut().put(key.into(), value.into());
        self
    }

    pub async fn run(self) -> Result<()> {
        let run = BoltRequest::run(&self.query, self.params.borrow().clone());
        let connection = self.connection.borrow_mut();
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

    pub async fn execute(self) -> Result<impl Stream<Item = Row>> {
        let run = BoltRequest::run(&self.query, self.params.borrow().clone());
        let run_response = self.connection.borrow().send_recv(run).await?;

        match run_response {
            BoltResponse::SuccessMessage(success) => {
                let fields: BoltList = success.get("fields").unwrap_or(BoltList::new());
                let connection = self.connection.clone();
                let stream = stream! {
                     let mut has_more = true;
                     while has_more {
                        let pull = BoltRequest::pull();
                        match connection.borrow().send(pull).await {
                            Ok(()) => loop {
                                match self.connection.borrow().recv().await {
                                    Ok(BoltResponse::SuccessMessage(s)) => {
                                        has_more = s.get("has_more").unwrap_or(false);
                                        break;
                                    },
                                    Ok(BoltResponse::RecordMessage(record)) => {
                                        yield Row::new(fields.clone(), record.data);
                                    },
                                    Ok(msg) => {
                                        println!("Got unexpected message: {:?}", msg);
                                        break;
                                    }
                                    Err(msg) => {
                                        println!("Got error while streaming: {:?}", msg);
                                        break;
                                    }
                                }
                            },
                            _ => break,
                        }
                     }
                };
                Ok(Box::pin(stream))
            }
            BoltResponse::FailureMessage(msg) => {
                println!("error executing query {:?}", msg);
                Err(Error::QueryError)
            }
            msg => {
                println!("unexpected message received: {:?}", msg);
                Err(Error::UnexpectedMessage)
            }
        }
    }
}
