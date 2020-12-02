use crate::connection::*;
use crate::error::*;
use crate::messages::*;
use crate::row::*;
use crate::types::*;
use futures::stream::Stream;
use std::cell::RefCell;
use std::rc::Rc;

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
