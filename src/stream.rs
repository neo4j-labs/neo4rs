use crate::connection::*;
use crate::errors::*;
use crate::messages::*;
use crate::row::*;
use crate::types::BoltList;
use futures::stream::Stream;
use futures::Future;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct RowStream {
    fields: BoltList,
    connection: Rc<RefCell<Connection>>,
}

impl RowStream {
    pub async fn new(fields: BoltList, connection: Rc<RefCell<Connection>>) -> Result<RowStream> {
        let pull = BoltRequest::pull();
        connection.borrow_mut().send(pull).await?;
        Ok(RowStream { fields, connection })
    }
}

impl Stream for RowStream {
    type Item = Row;
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Row>> {
        let mut connection = self.connection.borrow_mut();
        let mut future = Box::pin(connection.recv());
        match future.as_mut().poll(context) {
            Poll::Ready(Ok(BoltResponse::SuccessMessage(_))) => Poll::Ready(None),
            Poll::Ready(Ok(BoltResponse::RecordMessage(record))) => {
                Poll::Ready(Some(Row::new(self.fields.clone(), record.data)))
            }
            Poll::Ready(m) => panic!("unexpected message {:?}", m),
            Poll::Pending => Poll::Pending,
        }
    }
}
