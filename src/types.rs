pub mod boolean;
pub mod integer;
pub mod list;
pub mod map;
pub mod node;
pub mod null;
pub mod string;
pub use boolean::BoltBoolean;
pub use integer::BoltInteger;
pub use list::BoltList;
pub use map::BoltMap;
pub use node::BoltNode;
pub use null::BoltNull;
pub use string::BoltString;

use crate::errors::*;
use bytes::Bytes;
use core::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BoltType {
    String(BoltString),
    Boolean(BoltBoolean),
    Map(BoltMap),
    Null(BoltNull),
    Integer(BoltInteger),
    List(BoltList),
    Node(BoltNode),
}

impl Hash for BoltType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BoltType::String(t) => t.hash(state),
            BoltType::Boolean(t) => t.hash(state),
            BoltType::Null(t) => t.hash(state),
            BoltType::Integer(t) => t.hash(state),
            BoltType::List(t) => t.hash(state),
            BoltType::Node(_) => panic!("node not hashed"),
            BoltType::Map(_) => panic!("map not hashed"),
        }
    }
}

impl From<&str> for BoltType {
    fn from(v: &str) -> Self {
        BoltType::String(v.into())
    }
}

impl From<String> for BoltType {
    fn from(v: String) -> Self {
        BoltType::String(v.into())
    }
}

impl From<i64> for BoltType {
    fn from(v: i64) -> Self {
        BoltType::Integer(v.into())
    }
}

impl Into<String> for BoltType {
    fn into(self) -> String {
        match self {
            BoltType::String(t) => t.value,
            _ => "".to_owned(),
        }
    }
}

impl TryInto<Bytes> for BoltType {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        match self {
            BoltType::Null(t) => t.try_into(),
            BoltType::Boolean(t) => t.try_into(),
            BoltType::Integer(t) => t.try_into(),
            BoltType::String(t) => t.try_into(),
            BoltType::List(t) => t.try_into(),
            BoltType::Map(t) => t.try_into(),
            BoltType::Node(t) => t.try_into(),
        }
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltType {
    type Error = Error;
    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltType> {
        let bolt_type = match input {
            input if integer::is_present(input.clone()) => BoltType::Integer(input.try_into()?),
            input if string::is_present(input.clone()) => BoltType::String(input.try_into()?),
            input if list::is_present(input.clone()) => BoltType::List(input.try_into()?),
            input if map::is_present(input.clone()) => BoltType::Map(input.try_into()?),
            input if node::is_present(input.clone()) => BoltType::Node(input.try_into()?),
            _ => return Err(Error::UnknownTypeMarker),
        };
        Ok(bolt_type)
    }
}
