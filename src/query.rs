use crate::connection::*;
use crate::errors::*;
use crate::messages::*;
use crate::row::*;
use crate::stream::*;
use crate::types::*;
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

    pub fn param<T: std::convert::Into<BoltType>>(&self, key: &str, value: T) -> &Self {
        self.params.borrow_mut().put(key.into(), value.into());
        &self
    }

    pub async fn run(&self) -> Result<()> {
        let run = BoltRequest::run(&self.query, self.params.borrow().clone());
        let mut connection = self.connection.borrow_mut();
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

    pub async fn execute(&self) -> Result<impl Stream<Item = Row>> {
        let run = BoltRequest::run(&self.query, self.params.borrow().clone());
        let response = self.connection.borrow_mut().send_recv(run).await?;
        match response {
            BoltResponse::SuccessMessage(success) => {
                Ok(RowStream::new(success.fields(), self.connection.clone()).await?)
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
