mod connection;
mod error;
mod messages;
mod query;
mod row;
mod types;
mod version;
use crate::connection::*;
pub use crate::error::*;
use crate::messages::*;
use crate::query::*;
pub use crate::types::*;
pub use crate::version::Version;
use std::cell::RefCell;
use std::rc::Rc;

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
    pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Self> {
        let (mut connection, version) = Connection::new(uri.to_owned()).await?;
        let hello = BoltRequest::hello("neo4rs", user.to_owned(), password.to_owned());
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
