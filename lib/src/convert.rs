use crate::errors::*;
use crate::row::*;
use crate::types::*;
use std::convert::{TryFrom, TryInto};

impl<A: TryFrom<BoltType, Error = Error>> TryFrom<BoltType> for Vec<A> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Vec<A>> {
        match input {
            BoltType::List(l) => l.value.iter().map(|x| A::try_from(x.clone())).collect(),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for f64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<f64> {
        match input {
            BoltType::Float(t) => Ok(t.value),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for i64 {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<i64> {
        match input {
            BoltType::Integer(t) => Ok(t.value),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for bool {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<bool> {
        match input {
            BoltType::Boolean(t) => Ok(t.value),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for Point2D {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Point2D> {
        match input {
            BoltType::Point2D(p) => Ok(Point2D::new(p)),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for std::time::Duration {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<std::time::Duration> {
        match input {
            BoltType::Duration(d) => Ok(d.into()),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for chrono::NaiveDate {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<chrono::NaiveDate> {
        match input {
            BoltType::Date(d) => d.try_into(),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for chrono::DateTime<chrono::FixedOffset> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<chrono::DateTime<chrono::FixedOffset>> {
        match input {
            BoltType::DateTime(d) => d.try_into(),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for chrono::NaiveDateTime {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<chrono::NaiveDateTime> {
        match input {
            BoltType::LocalDateTime(d) => d.try_into(),
            _ => Err(Error::ConversionError),
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
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for (chrono::NaiveDateTime, String) {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<(chrono::NaiveDateTime, String)> {
        match input {
            BoltType::DateTimeZoneId(date_time_zone_id) => date_time_zone_id.try_into(),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for Vec<u8> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Vec<u8>> {
        match input {
            BoltType::Bytes(b) => Ok(b.value.to_vec()),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for Point3D {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Point3D> {
        match input {
            BoltType::Point3D(p) => Ok(Point3D::new(p)),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for Node {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Node> {
        match input {
            BoltType::Node(n) => Ok(Node::new(n)),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for Path {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Path> {
        match input {
            BoltType::Path(n) => Ok(Path::new(n)),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for Relation {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Relation> {
        match input {
            BoltType::Relation(r) => Ok(Relation::new(r)),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for UnboundedRelation {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<UnboundedRelation> {
        match input {
            BoltType::UnboundedRelation(r) => Ok(UnboundedRelation::new(r)),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for BoltList {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<BoltList> {
        match input {
            BoltType::List(l) => Ok(l),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for BoltString {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<BoltString> {
        match input {
            BoltType::String(s) => Ok(s),
            _ => Err(Error::ConversionError),
        }
    }
}

impl TryFrom<BoltType> for String {
    type Error = Error;
    fn try_from(input: BoltType) -> Result<String> {
        match input {
            BoltType::String(t) => Ok(t.value),
            _ => Err(Error::ConversionError),
        }
    }
}

impl Into<BoltType> for std::time::Duration {
    fn into(self) -> BoltType {
        BoltType::Duration(self.into())
    }
}

impl Into<BoltType> for chrono::NaiveDate {
    fn into(self) -> BoltType {
        BoltType::Date(self.into())
    }
}

impl Into<BoltType> for chrono::NaiveTime {
    fn into(self) -> BoltType {
        BoltType::LocalTime(self.into())
    }
}

impl Into<BoltType> for chrono::NaiveDateTime {
    fn into(self) -> BoltType {
        BoltType::LocalDateTime(self.into())
    }
}

impl Into<BoltType> for chrono::DateTime<chrono::FixedOffset> {
    fn into(self) -> BoltType {
        BoltType::DateTime(self.into())
    }
}

impl Into<BoltType> for (chrono::NaiveTime, chrono::FixedOffset) {
    fn into(self) -> BoltType {
        BoltType::Time(self.into())
    }
}

impl Into<BoltType> for (chrono::NaiveDateTime, &str) {
    fn into(self) -> BoltType {
        BoltType::DateTimeZoneId(self.into())
    }
}

impl<A: Into<BoltType> + Clone> Into<BoltType> for Vec<A> {
    fn into(self) -> BoltType {
        BoltType::List(BoltList {
            value: self.iter().map(|v| v.clone().into()).collect(),
        })
    }
}

impl<A: Into<BoltType> + Clone> Into<BoltType> for &[A] {
    fn into(self) -> BoltType {
        BoltType::List(BoltList {
            value: self.iter().map(|v| v.clone().into()).collect(),
        })
    }
}

impl Into<BoltType> for Vec<u8> {
    fn into(self) -> BoltType {
        BoltType::Bytes(BoltBytes::new(self.into()))
    }
}

impl Into<BoltType> for f64 {
    fn into(self) -> BoltType {
        BoltType::Float(BoltFloat::new(self))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_into_vec() {
        let value = BoltType::List(BoltList {
            value: vec![
                BoltType::Integer(BoltInteger::new(42)),
                BoltType::Integer(BoltInteger::new(1337)),
            ],
        });
        let value = Vec::<i64>::try_from(value).unwrap();
        assert_eq!(value, vec![42, 1337]);
    }

    #[test]
    fn convert_propagates_error() {
        let value = BoltType::List(BoltList {
            value: vec![
                BoltType::Integer(BoltInteger::new(42)),
                BoltType::Float(BoltFloat::new(13.37)),
            ],
        });
        let value = Vec::<i64>::try_from(value).unwrap_err();
        assert!(matches!(value, Error::ConversionError));
    }

    #[test]
    fn convert_from_vec() {
        let value: Vec<i64> = vec![42, 1337];
        let value: BoltType = value.into();
        assert_eq!(
            value,
            BoltType::List(BoltList {
                value: vec![
                    BoltType::Integer(BoltInteger::new(42)),
                    BoltType::Integer(BoltInteger::new(1337)),
                ],
            })
        );
    }

    #[test]
    fn convert_from_slice() {
        let value: Vec<i64> = vec![42, 1337];
        let value: BoltType = value.as_slice().into();
        assert_eq!(
            value,
            BoltType::List(BoltList {
                value: vec![
                    BoltType::Integer(BoltInteger::new(42)),
                    BoltType::Integer(BoltInteger::new(1337)),
                ],
            })
        );
    }
}
