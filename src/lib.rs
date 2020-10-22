mod connection;
mod error;
mod messages;
mod types;
mod version;
use crate::connection::*;
pub use crate::error::*;
use crate::messages::*;
pub use crate::types::*;
pub use crate::version::Version;
use futures::stream::Stream;
use futures::stream::TryStream;
use futures::Future;
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct Row {
    attributes: HashMap<String, BoltType>,
}

impl Row {
    fn new(fields: Vec<String>, values: Vec<BoltType>) -> Self {
        let mut attributes = HashMap::with_capacity(fields.len());
        for (field, value) in fields.into_iter().zip(values.into_iter()) {
            attributes.insert(field, value);
        }
        Row { attributes }
    }
}

#[derive(Debug)]
struct RowStream {
    fields: Vec<String>,
    connection: Rc<RefCell<Connection>>,
}

impl RowStream {
    async fn new(fields: Vec<String>, connection: Rc<RefCell<Connection>>) -> Result<RowStream> {
        Ok(RowStream { fields, connection })
    }
}

impl Stream for RowStream {
    type Item = Row;
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Row>> {
        let mut connection = self.connection.borrow_mut();
        let mut future = Box::pin(connection.recv());
        match future.as_mut().poll(context) {
            Poll::Ready(Ok(BoltResponse::SuccessMessage(success))) => Poll::Ready(None),
            Poll::Ready(Ok(BoltResponse::RecordMessage(record))) => {
                let row = Row::new(self.fields.clone(), record.into());
                Poll::Ready(Some(row))
            }
            Poll::Ready(_) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct QueryBuilder {
    query: String,
    connection: Rc<RefCell<Connection>>,
}

impl QueryBuilder {
    pub fn new(query: String, connection: Rc<RefCell<Connection>>) -> Self {
        QueryBuilder { query, connection }
    }

    pub async fn execute(&self) -> Result<impl Stream<Item = Row>> {
        let run = BoltRequest::run(&self.query, BoltMap::new());
        let response = self.connection.borrow_mut().request(run).await?;
        match response {
            BoltResponse::SuccessMessage(success) => {
                let pull = BoltRequest::pull();
                self.connection.borrow_mut().send(pull).await?;
                Ok(RowStream::new(success.fields(), self.connection.clone()).await?)
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    pub version: Version,
    pub state: State,
    connection: Rc<RefCell<Connection>>,
}

#[derive(Debug, PartialEq)]
pub enum State {
    Ready { id: String, server: String },
}

impl Graph {
    pub async fn connect(uri: String, user: String, password: String) -> Result<Self> {
        let (mut connection, version) = Connection::new(uri).await?;
        let hello = BoltRequest::hello("neo4rs", user, password);
        match connection.request(hello).await? {
            BoltResponse::SuccessMessage(msg) => Ok(Graph {
                version,
                state: State::Ready {
                    id: msg.connection_id(),
                    server: msg.server(),
                },
                connection: Rc::new(RefCell::new(connection)),
            }),
            BoltResponse::FailureMessage(msg) => Err(Error::AuthenticationError {
                detail: msg.message(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub fn query(&mut self, q: &str) -> QueryBuilder {
        QueryBuilder::new(q.to_owned(), self.connection.clone())
    }
}
