use crate::errors::*;
use crate::row::*;
use crate::types::*;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

impl<A: TryFrom<BoltType, Error = Error>> TryFrom<BoltType> for Vec<A> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<Vec<A>> {
        match input {
            BoltType::List(l) => l.value.iter().map(|x| A::try_from(x.clone())).collect(),
            _ => Err(Error::ConversionError),
        }
    }
}

impl<A: From<BoltString> + Eq + PartialEq + Hash, B: TryFrom<BoltType, Error = Error>> TryFrom<BoltType> for HashMap<A, B> {
    type Error = Error;

    fn try_from(input: BoltType) -> Result<HashMap<A, B>> {
        match input {
            BoltType::Map(l) => {
                let mut map = HashMap::new();
                for (key, val) in l.value {
                    if let BoltType::Null(_) = val {
                        ()
                    } else {
                        map.insert(A::from(key.clone()), B::try_from(val.clone())?);
                    }
                }
                Ok(map)
            },
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

impl From<std::time::Duration> for BoltType {
    fn from(value: std::time::Duration) -> BoltType {
        BoltType::Duration(value.into())
    }
}

impl From<chrono::NaiveDate> for BoltType {
    fn from(value: chrono::NaiveDate) -> BoltType {
        BoltType::Date(value.into())
    }
}

impl From<chrono::NaiveTime> for BoltType {
    fn from(value: chrono::NaiveTime) -> BoltType {
        BoltType::LocalTime(value.into())
    }
}

impl From<chrono::NaiveDateTime> for BoltType {
    fn from(value: chrono::NaiveDateTime) -> BoltType {
        BoltType::LocalDateTime(value.into())
    }
}

impl From<chrono::DateTime<chrono::FixedOffset>> for BoltType {
    fn from(value: chrono::DateTime<chrono::FixedOffset>) -> Self {
        BoltType::DateTime(value.into())
    }
}

impl From<(chrono::NaiveTime, chrono::FixedOffset)> for BoltType {
    fn from(value: (chrono::NaiveTime, chrono::FixedOffset)) -> Self {
        BoltType::Time(value.into())
    }
}

impl From<(chrono::NaiveDateTime, &str)> for BoltType {
    fn from(value: (chrono::NaiveDateTime, &str)) -> Self {
        BoltType::DateTimeZoneId(value.into())
    }
}

impl<A: Into<BoltType> + Clone> From<Vec<A>> for BoltType {
    fn from(value: Vec<A>) -> BoltType {
        BoltType::List(BoltList {
            value: value.iter().map(|v| v.clone().into()).collect(),
        })
    }
}

impl<A: Into<BoltType> + Clone> From<&[A]> for BoltType {
    fn from(value: &[A]) -> Self {
        BoltType::List(BoltList {
            value: value.iter().map(|v| v.clone().into()).collect(),
        })
    }
}

impl<A: Into<BoltString> + Clone, B: Into<BoltType> + Clone> From<HashMap<A, B>> for BoltType {
    fn from(value: HashMap<A, B>) -> Self {
        BoltType::Map(BoltMap { value: value.iter().fold(HashMap::new(), |mut accum, (k, v)| {
            accum.insert(k.clone().into(), v.clone().into());
            accum
        })})
    }
}

impl From<Vec<u8>> for BoltType {
    fn from(value: Vec<u8>) -> Self {
        BoltType::Bytes(BoltBytes::new(value.into()))
    }
}

impl From<&[u8]> for BoltType {
    fn from(value: &[u8]) -> Self {
        Self::from(value.to_vec())
    }
}

impl From<f64> for BoltType {
    fn from(val: f64) -> Self {
        BoltType::Float(BoltFloat::new(val))
    }
}

impl From<f32> for BoltType {
    fn from(val: f32) -> Self {
        Self::from(f64::from(val))
    }
}

impl From<bool> for BoltType {
    fn from(val: bool) -> Self {
        BoltType::Boolean(BoltBoolean::new(val))
    }
}

impl From<i64> for BoltType {
    fn from(value: i64) -> BoltType {
        BoltType::Integer(BoltInteger::new(value))
    }
}

macro_rules! int_impl {
    ($($ty:ty),+) => {
        $(
            impl From<$ty> for BoltType {
                fn from(val: $ty) -> Self {
                    Self::from(i64::from(val))
                }
            }
        )+
    };

    (try $($ty:ty),+) => {
        $(
            impl TryFrom<$ty> for BoltType {
                type Error = ::std::num::TryFromIntError;

                fn try_from(val: $ty) -> ::std::result::Result<Self, Self::Error> {
                    match i64::try_from(val) {
                        Ok(v) => Ok(Self::from(v)),
                        Err(e) => Err(e),
                    }
                }
            }
        )+
    };
}

// no impl for u8 as it produces a
// conflict of From impls for Vec<A> and Vec<u8>
int_impl!(i8, i16, i32, u16, u32);
int_impl!(try isize, i128, usize, u64, u128);

impl From<String> for BoltType {
    fn from(value: String) -> Self {
        BoltType::String(value.into())
    }
}

impl From<&str> for BoltType {
    fn from(value: &str) -> Self {
        BoltType::String(value.into())
    }
}

impl<T: Into<BoltType>> From<Option<T>> for BoltType {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => BoltType::Null(BoltNull),
        }
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
    fn convert_into_map() {
        let map = HashMap::from([(BoltString::new("42"), BoltType::Null(BoltNull {})), 
            (BoltString::new("1337"), BoltType::Integer(BoltInteger::new(1337)))]);
        let value = BoltType::Map(BoltMap {
            value: map,
        });
        let value = HashMap::<String, i64>::try_from(value).unwrap();
        assert_eq!(value, HashMap::from([("1337".to_owned(), 1337_i64)]));
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
    fn convert_from_map() {
        let map = HashMap::from([("1337".to_owned(), 1337_i64)]);
        let value: BoltType = map.into();
        assert_eq!(
            value,
            BoltType::Map(BoltMap {
                value: HashMap::from([(BoltString::new("1337"), BoltType::Integer(BoltInteger::new(1337)))]),
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

    #[test]
    fn convert_from_option() {
        let value: Option<Vec<i64>> = Some(vec![42, 1337]);
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
        let value: Option<Vec<i64>> = None;
        let value: BoltType = value.into();
        assert_eq!(value, BoltType::Null(BoltNull));
    }
}
