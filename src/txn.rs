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
    pub async fn new(connection: Rc<RefCell<Connection>>) -> Result<Self> {
        let begin = BoltRequest::begin();
        match connection.borrow_mut().send_recv(begin).await? {
            BoltResponse::SuccessMessage(_) => Ok(Txn {
                connection: connection.clone(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn commit(&self) -> Result<()> {
        let connection = self.connection.borrow();
        match connection.send_recv(BoltRequest::commit()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn rollback(&self) -> Result<()> {
        let connection = self.connection.borrow();
        match connection.send_recv(BoltRequest::rollback()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
