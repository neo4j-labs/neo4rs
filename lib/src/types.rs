pub mod binary;
pub mod boolean;
pub mod date;
pub mod date_time;
pub mod duration;
pub mod float;
pub mod integer;
pub mod list;
pub mod map;
pub mod node;
pub mod null;
pub mod path;
pub mod point;
pub mod relation;
pub mod string;
pub mod time;
pub use binary::BoltBytes;
pub use boolean::BoltBoolean;
pub use date::BoltDate;
pub use date_time::{BoltDateTime, BoltDateTimeZoneId, BoltLocalDateTime};
pub use duration::BoltDuration;
pub use float::BoltFloat;
pub use integer::BoltInteger;
pub use list::BoltList;
pub use map::BoltMap;
pub use node::BoltNode;
pub use null::BoltNull;
pub use path::BoltPath;
pub use point::{BoltPoint2D, BoltPoint3D};
pub use relation::{BoltRelation, BoltUnboundedRelation};
pub use string::BoltString;
pub use time::{BoltLocalTime, BoltTime};

use crate::errors::*;
use crate::version::Version;
use bytes::Bytes;
use core::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum BoltType {
    String(BoltString),
    Boolean(BoltBoolean),
    Map(BoltMap),
    Null(BoltNull),
    Integer(BoltInteger),
    Float(BoltFloat),
    List(BoltList),
    Node(BoltNode),
    Relation(BoltRelation),
    UnboundedRelation(BoltUnboundedRelation),
    Point2D(BoltPoint2D),
    Point3D(BoltPoint3D),
    Bytes(BoltBytes),
    Path(BoltPath),
    Duration(BoltDuration),
    Date(BoltDate),
    Time(BoltTime),
    LocalTime(BoltLocalTime),
    DateTime(BoltDateTime),
    LocalDateTime(BoltLocalDateTime),
    DateTimeZoneId(BoltDateTimeZoneId),
}

impl Display for BoltType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            BoltType::String(s) => s.to_string(),
            _ => "to_string not implemented".to_owned(),
        };
        write!(f, "{}", value)
    }
}

impl Hash for BoltType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BoltType::String(t) => t.hash(state),
            BoltType::Boolean(t) => t.hash(state),
            BoltType::Null(t) => t.hash(state),
            BoltType::Integer(t) => t.hash(state),
            BoltType::List(t) => t.hash(state),
            BoltType::Duration(t) => t.hash(state),
            BoltType::Date(t) => t.hash(state),
            BoltType::Time(t) => t.hash(state),
            BoltType::LocalTime(t) => t.hash(state),
            BoltType::DateTime(t) => t.hash(state),
            BoltType::LocalDateTime(t) => t.hash(state),
            BoltType::DateTimeZoneId(t) => t.hash(state),
            //The below types cannot be hashed
            BoltType::Path(_) => panic!("path not hashed"),
            BoltType::Bytes(_) => panic!("bytes not hashed"),
            BoltType::Float(_) => panic!("float not hashed"),
            BoltType::Point2D(_) => panic!("point2d not hashed"),
            BoltType::Point3D(_) => panic!("point3d not hashed"),
            BoltType::Node(_) => panic!("node not hashed"),
            BoltType::Map(_) => panic!("map not hashed"),
            BoltType::Relation(_) => panic!("relation not hashed"),
            BoltType::UnboundedRelation(_) => panic!("relation not hashed"),
        }
    }
}

impl BoltType {
    pub fn to_bytes(self, version: Version) -> Result<Bytes> {
        match self {
            BoltType::Null(t) => t.to_bytes(version),
            BoltType::Boolean(t) => t.to_bytes(version),
            BoltType::Integer(t) => t.to_bytes(version),
            BoltType::Float(t) => t.to_bytes(version),
            BoltType::String(t) => t.to_bytes(version),
            BoltType::List(t) => t.to_bytes(version),
            BoltType::Point2D(t) => t.to_bytes(version),
            BoltType::Point3D(t) => t.to_bytes(version),
            BoltType::Map(t) => t.to_bytes(version),
            BoltType::Node(t) => t.to_bytes(version),
            BoltType::Path(t) => t.to_bytes(version),
            BoltType::Relation(t) => t.to_bytes(version),
            BoltType::UnboundedRelation(t) => t.to_bytes(version),
            BoltType::Bytes(t) => t.to_bytes(version),
            BoltType::Duration(t) => t.to_bytes(version),
            BoltType::Date(t) => t.to_bytes(version),
            BoltType::Time(t) => t.to_bytes(version),
            BoltType::LocalTime(t) => t.to_bytes(version),
            BoltType::DateTime(t) => t.to_bytes(version),
            BoltType::LocalDateTime(t) => t.to_bytes(version),
            BoltType::DateTimeZoneId(t) => t.to_bytes(version),
        }
    }

    fn parse(version: Version, input: Rc<RefCell<Bytes>>) -> Result<BoltType> {
        let bolt_type = match input {
            input if BoltInteger::can_parse(version, input.clone()) => {
                BoltType::Integer(BoltInteger::parse(version, input)?)
            }
            input if BoltFloat::can_parse(version, input.clone()) => {
                BoltType::Float(BoltFloat::parse(version, input)?)
            }
            input if BoltString::can_parse(version, input.clone()) => {
                BoltType::String(BoltString::parse(version, input)?)
            }
            input if BoltList::can_parse(version, input.clone()) => {
                BoltType::List(BoltList::parse(version, input)?)
            }
            input if BoltMap::can_parse(version, input.clone()) => {
                BoltType::Map(BoltMap::parse(version, input)?)
            }
            input if BoltNode::can_parse(version, input.clone()) => {
                BoltType::Node(BoltNode::parse(version, input)?)
            }
            input if BoltBoolean::can_parse(version, input.clone()) => {
                BoltType::Boolean(BoltBoolean::parse(version, input)?)
            }
            input if BoltPoint2D::can_parse(version, input.clone()) => {
                BoltType::Point2D(BoltPoint2D::parse(version, input)?)
            }
            input if BoltPoint3D::can_parse(version, input.clone()) => {
                BoltType::Point3D(BoltPoint3D::parse(version, input)?)
            }
            input if BoltBytes::can_parse(version, input.clone()) => {
                BoltType::Bytes(BoltBytes::parse(version, input)?)
            }
            input if BoltPath::can_parse(version, input.clone()) => {
                BoltType::Path(BoltPath::parse(version, input)?)
            }
            input if BoltDuration::can_parse(version, input.clone()) => {
                BoltType::Duration(BoltDuration::parse(version, input)?)
            }
            input if BoltDate::can_parse(version, input.clone()) => {
                BoltType::Date(BoltDate::parse(version, input)?)
            }
            input if BoltTime::can_parse(version, input.clone()) => {
                BoltType::Time(BoltTime::parse(version, input)?)
            }
            input if BoltLocalTime::can_parse(version, input.clone()) => {
                BoltType::LocalTime(BoltLocalTime::parse(version, input)?)
            }
            input if BoltDateTime::can_parse(version, input.clone()) => {
                BoltType::DateTime(BoltDateTime::parse(version, input)?)
            }
            input if BoltLocalDateTime::can_parse(version, input.clone()) => {
                BoltType::LocalDateTime(BoltLocalDateTime::parse(version, input)?)
            }
            input if BoltDateTimeZoneId::can_parse(version, input.clone()) => {
                BoltType::DateTimeZoneId(BoltDateTimeZoneId::parse(version, input)?)
            }
            input if BoltUnboundedRelation::can_parse(version, input.clone()) => {
                BoltType::UnboundedRelation(BoltUnboundedRelation::parse(version, input)?)
            }
            input if BoltRelation::can_parse(version, input.clone()) => {
                BoltType::Relation(BoltRelation::parse(version, input)?)
            }
            _ => return Err(Error::UnknownType(format!("{:#04X?}", input.borrow()))),
        };
        Ok(bolt_type)
    }
}
