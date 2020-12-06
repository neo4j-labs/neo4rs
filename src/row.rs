use crate::connection::*;
use crate::errors::*;
use crate::messages::*;
use crate::types::*;
use futures::stream::Stream;
use futures::Future;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
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

impl Node {
    pub fn new(data: BoltNode) -> Self {
        Node { data }
    }

    pub fn id(&self) -> i64 {
        self.data.id.value
    }

    pub fn labels(&self) -> Vec<String> {
        self.data.labels.iter().map(|l| l.to_string()).collect()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        match self.data.properties.get(key) {
            Some(bolt_type) => {
                if let Ok(value) = TryInto::<T>::try_into(bolt_type.clone()) {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Row {
    pub fn new(fields: Vec<String>, data: BoltList) -> Self {
        let mut attributes = HashMap::with_capacity(fields.len());
        for (field, value) in fields.into_iter().zip(data.into_iter()) {
            attributes.insert(field, value);
        }
        Row { attributes }
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        match self.attributes.get(key) {
            Some(bolt_type) => {
                if let Ok(value) = TryInto::<T>::try_into(bolt_type.clone()) {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
