use crate::errors::*;
use crate::row::*;
use crate::types::*;
use std::convert::{TryFrom, TryInto};

impl TryFrom<BoltType> for f64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<f64> {
        match input {
            BoltType::Float(t) => Ok(t.value),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for i64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<i64> {
        match input {
            BoltType::Integer(t) => Ok(t.value),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for bool {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<bool> {
        match input {
            BoltType::Boolean(t) => Ok(t.value),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for Point2D {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Point2D> {
        match input {
            BoltType::Point2D(p) => Ok(Point2D::new(p)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for std::time::Duration {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<std::time::Duration> {
        match input {
            BoltType::Duration(d) => Ok(d.into()),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for chrono::NaiveDate {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<chrono::NaiveDate> {
        match input {
            BoltType::Date(d) => d.try_into(),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for chrono::DateTime<chrono::FixedOffset> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<chrono::DateTime<chrono::FixedOffset>> {
        match input {
            BoltType::DateTime(d) => d.try_into(),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for chrono::NaiveDateTime {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<chrono::NaiveDateTime> {
        match input {
            BoltType::LocalDateTime(d) => d.try_into(),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for (chrono::NaiveTime, Option<chrono::FixedOffset>) {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<(chrono::NaiveTime, Option<chrono::FixedOffset>)> {
        match input {
            BoltType::Time(bolt_time) => {
                let (time, offset) = bolt_time.into();
                if offset.local_minus_utc() == 0 {
                    Ok((time, None))
                } else {
                    Ok((time, Some(offset)))
                }
            }
            BoltType::LocalTime(d) => Ok((d.into(), None)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for (chrono::NaiveDateTime, String) {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<(chrono::NaiveDateTime, String)> {
        match input {
            BoltType::DateTimeZoneId(date_time_zone_id) => date_time_zone_id.try_into(),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for Vec<u8> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Vec<u8>> {
        match input {
            BoltType::Bytes(b) => Ok(b.value.to_vec()),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for Point3D {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Point3D> {
        match input {
            BoltType::Point3D(p) => Ok(Point3D::new(p)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for Node {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Node> {
        match input {
            BoltType::Node(n) => Ok(Node::new(n)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for Path {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Path> {
        match input {
            BoltType::Path(n) => Ok(Path::new(n)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for Relation {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Relation> {
        match input {
            BoltType::Relation(r) => Ok(Relation::new(r)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for UnboundedRelation {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<UnboundedRelation> {
        match input {
            BoltType::UnboundedRelation(r) => Ok(UnboundedRelation::new(r)),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for BoltList {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<BoltList> {
        match input {
            BoltType::List(l) => Ok(l),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for BoltString {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<BoltString> {
        match input {
            BoltType::String(s) => Ok(s),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl TryFrom<BoltType> for String {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<String> {
        match input {
            BoltType::String(t) => Ok(t.value),
            _ => Err(Error::ConvertError(input)),
        }
    }
}

impl From<std::time::Duration> for BoltType {
    fn from(val: std::time::Duration) -> Self {
        BoltType::Duration(val.into())
    }
}

impl From<chrono::NaiveDate> for BoltType {
    fn from(val: chrono::NaiveDate) -> Self {
        BoltType::Date(val.into())
    }
}

impl From<chrono::NaiveTime> for BoltType {
    fn from(val: chrono::NaiveTime) -> Self {
        BoltType::LocalTime(val.into())
    }
}

impl From<chrono::NaiveDateTime> for BoltType {
    fn from(val: chrono::NaiveDateTime) -> Self {
        BoltType::LocalDateTime(val.into())
    }
}

impl From<chrono::DateTime<chrono::FixedOffset>> for BoltType {
    fn from(val: chrono::DateTime<chrono::FixedOffset>) -> Self {
        BoltType::DateTime(val.into())
    }
}

impl From<(chrono::NaiveTime, chrono::FixedOffset)> for BoltType {
    fn from(val: (chrono::NaiveTime, chrono::FixedOffset)) -> Self {
        BoltType::Time(val.into())
    }
}

impl From<(chrono::NaiveDateTime, &str)> for BoltType {
    fn from(val: (chrono::NaiveDateTime, &str)) -> Self {
        BoltType::DateTimeZoneId(val.into())
    }
}

impl From<Vec<u8>> for BoltType {
    fn from(val: Vec<u8>) -> Self {
        BoltType::Bytes(BoltBytes::new(val.into()))
    }
}

impl From<i64> for BoltType {
    fn from(val: i64) -> Self {
        BoltType::Integer(BoltInteger::new(val))
    }
}

impl From<String> for BoltType {
    fn from(val: String) -> Self {
        BoltType::String(val.into())
    }
}

impl From<&str> for BoltType {
    fn from(val: &str) -> Self {
        BoltType::String(val.into())
    }
}
