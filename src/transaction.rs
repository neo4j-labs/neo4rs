use crate::connection::*;
use crate::errors::*;
use crate::messages::*;
use crate::result::*;
use crate::types::*;
use futures::stream::Stream;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct TxnHandle {
    connection: Rc<RefCell<Connection>>,
}

impl TxnHandle {
    pub fn new(connection: Rc<RefCell<Connection>>) -> Self {
        TxnHandle { connection }
    }

    pub async fn commit(&self) -> Result<()> {
        Ok(())
    }
}
