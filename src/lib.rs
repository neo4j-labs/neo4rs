mod connection;
mod convert;
mod errors;
mod messages;
mod query;
mod row;
mod stream;
mod txn;
mod types;
mod version;
use crate::connection::*;
pub use crate::errors::*;
use crate::messages::*;
use crate::query::*;
pub use crate::row::{Node, Relation, Row};
pub use crate::txn::Txn;
pub use crate::version::Version;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Graph {
    pub version: Version,
    state: State,
    connection: Rc<RefCell<Connection>>,
}

#[derive(Debug, PartialEq)]
enum State {
    Ready { id: String, server: String },
}

impl Graph {
    pub async fn begin_txn(&self) -> Result<Txn> {
        Ok(Txn::new(self.connection.clone()).await?)
    }

    pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Self> {
        let (connection, version) = Connection::new(uri.to_owned()).await?;
        let hello = BoltRequest::hello("neo4rs", user.to_owned(), password.to_owned());
        match connection.send_recv(hello).await? {
            BoltResponse::SuccessMessage(msg) => Ok(Graph {
                version,
                state: State::Ready {
                    id: msg.get("connection_id").unwrap(),
                    server: msg.get("server").unwrap(),
                },
                connection: Rc::new(RefCell::new(connection)),
            }),
            BoltResponse::FailureMessage(msg) => Err(Error::AuthenticationError {
                detail: msg.get("message").unwrap(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub fn query(&self, q: &str) -> QueryBuilder {
        QueryBuilder::new(q.to_owned(), self.connection.clone())
    }
}
