use crate::errors::*;
use crate::row::*;
use crate::types::*;
use std::convert::TryFrom;

impl TryFrom<BoltType> for i64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<i64> {
        match input {
            BoltType::Integer(t) => Ok(t.value),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for Node {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Node> {
        match input {
            BoltType::Node(n) => Ok(Node::new(n)),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for String {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<String> {
        match input {
            BoltType::String(t) => Ok(t.value),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl Into<BoltType> for i64 {
    fn into(self) -> BoltType {
        BoltType::Integer(BoltInteger::new(self))
    }
}

impl Into<BoltType> for String {
    fn into(self) -> BoltType {
        BoltType::String(self.into())
    }
}

impl Into<BoltType> for &str {
    fn into(self) -> BoltType {
        BoltType::String(self.into())
    }
}
