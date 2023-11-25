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
pub(crate) mod serde;
pub mod string;
pub mod time;
pub use self::time::{BoltLocalTime, BoltTime};
mod wire;
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
pub(crate) use wire::BoltWireFormat;

use crate::{
    errors::{Error, Result},
    version::Version,
};
use bytes::{Bytes, BytesMut};
use std::fmt::Display;

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
        match self {
            BoltType::String(s) => f.write_str(&s.value),
            _ => f.write_str("to_string not implemented"),
        }
    }
}

impl BoltType {
    fn write_into(&self, version: Version, bytes: &mut BytesMut) -> Result<()> {
        match self {
            BoltType::Null(t) => t.write_into(version, bytes),
            BoltType::Boolean(t) => t.write_into(version, bytes),
            BoltType::Integer(t) => t.write_into(version, bytes),
            BoltType::Float(t) => t.write_into(version, bytes),
            BoltType::String(t) => t.write_into(version, bytes),
            BoltType::List(t) => t.write_into(version, bytes),
            BoltType::Point2D(t) => t.write_into(version, bytes),
            BoltType::Point3D(t) => t.write_into(version, bytes),
            BoltType::Map(t) => t.write_into(version, bytes),
            BoltType::Node(t) => t.write_into(version, bytes),
            BoltType::Path(t) => t.write_into(version, bytes),
            BoltType::Relation(t) => t.write_into(version, bytes),
            BoltType::UnboundedRelation(t) => t.write_into(version, bytes),
            BoltType::Bytes(t) => t.write_into(version, bytes),
            BoltType::Duration(t) => t.write_into(version, bytes),
            BoltType::Date(t) => t.write_into(version, bytes),
            BoltType::Time(t) => t.write_into(version, bytes),
            BoltType::LocalTime(t) => t.write_into(version, bytes),
            BoltType::DateTime(t) => t.write_into(version, bytes),
            BoltType::LocalDateTime(t) => t.write_into(version, bytes),
            BoltType::DateTimeZoneId(t) => t.write_into(version, bytes),
        }
    }

    fn parse(version: Version, input: &mut Bytes) -> Result<BoltType> {
        let bolt_type = match input {
            input if BoltInteger::can_parse(version, input) => {
                BoltType::Integer(BoltInteger::parse(version, input)?)
            }
            input if BoltFloat::can_parse(version, input) => {
                BoltType::Float(BoltFloat::parse(version, input)?)
            }
            input if BoltString::can_parse(version, input) => {
                BoltType::String(BoltString::parse(version, input)?)
            }
            input if BoltList::can_parse(version, input) => {
                BoltType::List(BoltList::parse(version, input)?)
            }
            input if BoltMap::can_parse(version, input) => {
                BoltType::Map(BoltMap::parse(version, input)?)
            }
            input if BoltNode::can_parse(version, input) => {
                BoltType::Node(BoltNode::parse(version, input)?)
            }
            input if BoltBoolean::can_parse(version, input) => {
                BoltType::Boolean(BoltBoolean::parse(version, input)?)
            }
            input if BoltNull::can_parse(version, input) => {
                BoltType::Null(BoltNull::parse(version, input)?)
            }
            input if BoltPoint2D::can_parse(version, input) => {
                BoltType::Point2D(BoltPoint2D::parse(version, input)?)
            }
            input if BoltPoint3D::can_parse(version, input) => {
                BoltType::Point3D(BoltPoint3D::parse(version, input)?)
            }
            input if BoltBytes::can_parse(version, input) => {
                BoltType::Bytes(BoltBytes::parse(version, input)?)
            }
            input if BoltPath::can_parse(version, input) => {
                BoltType::Path(BoltPath::parse(version, input)?)
            }
            input if BoltDuration::can_parse(version, input) => {
                BoltType::Duration(BoltDuration::parse(version, input)?)
            }
            input if BoltDate::can_parse(version, input) => {
                BoltType::Date(BoltDate::parse(version, input)?)
            }
            input if BoltTime::can_parse(version, input) => {
                BoltType::Time(BoltTime::parse(version, input)?)
            }
            input if BoltLocalTime::can_parse(version, input) => {
                BoltType::LocalTime(BoltLocalTime::parse(version, input)?)
            }
            input if BoltDateTime::can_parse(version, input) => {
                BoltType::DateTime(BoltDateTime::parse(version, input)?)
            }
            input if BoltLocalDateTime::can_parse(version, input) => {
                BoltType::LocalDateTime(BoltLocalDateTime::parse(version, input)?)
            }
            input if BoltDateTimeZoneId::can_parse(version, input) => {
                BoltType::DateTimeZoneId(BoltDateTimeZoneId::parse(version, input)?)
            }
            input if BoltUnboundedRelation::can_parse(version, input) => {
                BoltType::UnboundedRelation(BoltUnboundedRelation::parse(version, input)?)
            }
            input if BoltRelation::can_parse(version, input) => {
                BoltType::Relation(BoltRelation::parse(version, input)?)
            }
            _ => return Err(Error::UnknownType(format!("{:#04X?}", input))),
        };
        Ok(bolt_type)
    }
}
