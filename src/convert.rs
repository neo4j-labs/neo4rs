use crate::error::*;
use crate::row::*;
use crate::types::*;
use std::convert::{TryFrom, TryInto};

impl TryFrom<BoltType> for i64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<i64> {
        match input {
            BoltType::Integer(t) => Ok(t.value),
            _ => Err(Error::InvalidDataType),
        }
    }
}

impl TryFrom<BoltType> for Node {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Node> {
        match input {
            BoltType::Node(n) => Ok(Node::new(n)),
            _ => Err(Error::InvalidDataType),
        }
    }
}
