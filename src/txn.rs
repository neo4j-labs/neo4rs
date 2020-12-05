use crate::connection::*;
use crate::errors::*;
use crate::messages::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Txn {
    connection: Rc<RefCell<Connection>>,
}

impl Txn {
    pub fn new(connection: Rc<RefCell<Connection>>) -> Self {
        Txn { connection }
    }

    pub async fn commit(&self) -> Result<()> {
        let mut connection = self.connection.borrow_mut();
        match connection.request(BoltRequest::commit()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn rollback(&self) -> Result<()> {
        let mut connection = self.connection.borrow_mut();
        match connection.request(BoltRequest::rollback()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
