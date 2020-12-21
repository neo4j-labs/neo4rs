use crate::errors::*;
use crate::row::*;
use crate::types::*;
use std::convert::TryFrom;

impl TryFrom<BoltType> for f64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<f64> {
        match input {
            BoltType::Float(t) => Ok(t.value),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for i64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<i64> {
        match input {
            BoltType::Integer(t) => Ok(t.value),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for bool {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<bool> {
        match input {
            BoltType::Boolean(t) => Ok(t.value),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for Point2D {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Point2D> {
        match input {
            BoltType::Point2D(p) => Ok(Point2D::new(p)),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for Vec<u8> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Vec<u8>> {
        match input {
            BoltType::Bytes(b) => Ok(b.value.to_vec()),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for Point3D {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Point3D> {
        match input {
            BoltType::Point3D(p) => Ok(Point3D::new(p)),
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

impl TryFrom<BoltType> for Path {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Path> {
        match input {
            BoltType::Path(n) => Ok(Path::new(n)),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for Relation {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Relation> {
        match input {
            BoltType::Relation(r) => Ok(Relation::new(r)),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for UnboundedRelation {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<UnboundedRelation> {
        match input {
            BoltType::UnboundedRelation(r) => Ok(UnboundedRelation::new(r)),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for BoltList {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<BoltList> {
        match input {
            BoltType::List(l) => Ok(l),
            _ => Err(Error::ConverstionError),
        }
    }
}

impl TryFrom<BoltType> for BoltString {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<BoltString> {
        match input {
            BoltType::String(s) => Ok(s),
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

impl Into<BoltType> for Vec<u8> {
    fn into(self) -> BoltType {
        BoltType::Bytes(BoltBytes::new(self.into()))
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
