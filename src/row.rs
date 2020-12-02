use crate::connection::*;
use crate::error::*;
use crate::messages::*;
use crate::types::*;
use futures::stream::Stream;
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

#[derive(Debug)]
pub struct Node {
    data: BoltNode,
}

impl Row {
    pub fn new(fields: Vec<String>, data: BoltList) -> Self {
        let mut attributes = HashMap::with_capacity(fields.len());
        for (field, value) in fields.into_iter().zip(data.into_iter()) {
            attributes.insert(field, value);
        }
        Row { attributes }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.attributes.get(key) {
            Some(BoltType::String(s)) => Some(s.value.clone()),
            _ => None,
        }
    }

    pub fn get_node(&self, key: &str) -> Option<Node> {
        match self.attributes.get(key) {
            Some(BoltType::Node(n)) => Some(Node { data: n.clone() }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct RowStream {
    fields: Vec<String>,
    connection: Rc<RefCell<Connection>>,
}

impl RowStream {
    pub async fn new(
        fields: Vec<String>,
        connection: Rc<RefCell<Connection>>,
    ) -> Result<RowStream> {
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
