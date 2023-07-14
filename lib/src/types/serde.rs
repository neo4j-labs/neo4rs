use ::serde::{
    de::{self, value::StrDeserializer},
    forward_to_deserialize_any, Deserialize,
};
use serde::de::value::{MapDeserializer, SeqDeserializer};

use crate::types::{BoltMap, BoltString, BoltType};

impl BoltMap {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(MapDeserializer::new(self.value.iter()))
    }
}

impl BoltType {
    #[allow(unused)]
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(BoltTypeDeserializer::new(self))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeError {
    #[error("{0}")]
    Error(String),

    #[error("Could not convert the integer `{1}` to the target type {2}")]
    IntegerOutOfBounds(#[source] std::num::TryFromIntError, i64, &'static str),
}

impl de::Error for DeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Error(msg.to_string())
    }
}

pub struct BoltTypeDeserializer<'de> {
    value: &'de BoltType,
}

impl<'de> BoltTypeDeserializer<'de> {
    fn new(value: &'de BoltType) -> Self {
        Self { value }
    }
}

impl<'de> de::IntoDeserializer<'de, DeError> for &'de BoltType {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(self)
    }
}

impl<'de> de::IntoDeserializer<'de, DeError> for &'de BoltString {
    type Deserializer = StrDeserializer<'de, DeError>;

    fn into_deserializer(self) -> Self::Deserializer {
        StrDeserializer::new(&self.value)
    }
}

impl<'de> de::Deserializer<'de> for BoltTypeDeserializer<'de> {
    type Error = DeError;

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::List(v) => visitor.visit_seq(SeqDeserializer::new(v.value.iter())),
            BoltType::Bytes(v) => visitor.visit_seq(SeqDeserializer::new(v.value.iter().copied())),
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::Map(v) => visitor.visit_map(MapDeserializer::new(v.value.iter())),
            BoltType::Node(v) => visitor.visit_map(MapDeserializer::new(v.properties.value.iter())),
            BoltType::Relation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltType::UnboundedRelation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::Map(v) => visitor.visit_map(MapDeserializer::new(v.value.iter())),
            BoltType::Node(v) => visitor.visit_map(MapDeserializer::new(v.properties.value.iter())),
            BoltType::Relation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltType::UnboundedRelation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::List(v) if v.len() == len => {
                visitor.visit_seq(SeqDeserializer::new(v.value.iter()))
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::String(v) = self.value {
            visitor.visit_borrowed_str(&v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::String(v) = self.value {
            visitor.visit_string(v.value.clone())
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Bytes(v) = self.value {
            visitor.visit_borrowed_bytes(&v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Bytes(v) = self.value {
            visitor.visit_byte_buf(v.value.to_vec())
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Boolean(v) = self.value {
            visitor.visit_bool(v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_float(visitor)?;
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_float(visitor)?;
        visitor.visit_f64(v)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Null(_) = self.value {
            visitor.visit_unit()
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Null(_) = self.value {
            visitor.visit_unit()
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.unexpected(visitor)
    }

    forward_to_deserialize_any! {
        char option newtype_struct enum identifier
    }
}

impl<'de> BoltTypeDeserializer<'de> {
    fn read_integer<T, E, V>(self, visitor: V) -> Result<(T, V), DeError>
    where
        V: de::Visitor<'de>,
        i64: TryInto<T, Error = E>,
        E: Into<std::num::TryFromIntError>,
    {
        if let BoltType::Integer(v) = self.value {
            match v.value.try_into() {
                Ok(v) => Ok((v, visitor)),
                Err(e) => Err(DeError::IntegerOutOfBounds(
                    e.into(),
                    v.value,
                    std::any::type_name::<T>(),
                )),
            }
        } else {
            self.unexpected(visitor)
        }
    }

    fn read_float<T, V>(self, visitor: V) -> Result<(T, V), DeError>
    where
        V: de::Visitor<'de>,
        T: FromFloat,
    {
        if let BoltType::Float(v) = self.value {
            Ok((T::from_float(v.value), visitor))
        } else {
            self.unexpected(visitor)
        }
    }

    fn unexpected<V, T>(self, visitor: V) -> Result<T, DeError>
    where
        V: de::Visitor<'de>,
    {
        let typ = match self.value {
            BoltType::String(v) => de::Unexpected::Str(&v.value),
            BoltType::Boolean(v) => de::Unexpected::Bool(v.value),
            BoltType::Map(_) => de::Unexpected::Map,
            BoltType::Null(_) => de::Unexpected::Unit,
            BoltType::Integer(v) => de::Unexpected::Signed(v.value),
            BoltType::Float(v) => de::Unexpected::Float(v.value),
            BoltType::List(_) => de::Unexpected::Seq,
            BoltType::Node(_) => de::Unexpected::Map,
            BoltType::Relation(_) => de::Unexpected::Map,
            BoltType::UnboundedRelation(_) => de::Unexpected::Map,
            BoltType::Point2D(_) => de::Unexpected::Other("Point2D"),
            BoltType::Point3D(_) => de::Unexpected::Other("Point3D"),
            BoltType::Bytes(v) => de::Unexpected::Bytes(&v.value),
            BoltType::Path(_) => de::Unexpected::Other("Path"),
            BoltType::Duration(_) => de::Unexpected::Other("Duration"),
            BoltType::Date(_) => de::Unexpected::Other("Date"),
            BoltType::Time(_) => de::Unexpected::Other("Time"),
            BoltType::LocalTime(_) => de::Unexpected::Other("LocalTime"),
            BoltType::DateTime(_) => de::Unexpected::Other("DateTime"),
            BoltType::LocalDateTime(_) => de::Unexpected::Other("LocalDateTime"),
            BoltType::DateTimeZoneId(_) => de::Unexpected::Other("DateTimeZoneId"),
        };

        Err(de::Error::invalid_type(typ, &visitor))
    }
}

trait FromFloat {
    fn from_float(f: f64) -> Self;
}

impl FromFloat for f32 {
    fn from_float(f: f64) -> Self {
        f as f32
    }
}

impl FromFloat for f64 {
    fn from_float(f: f64) -> Self {
        f
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::borrow::Cow;

    use super::*;
    use crate::types::{BoltInteger, BoltNode, BoltNull, BoltRelation};

    #[test]
    fn map_with_extra_fields() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
            (BoltString::from("bar"), BoltType::from(1337)),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let map = BoltType::Map(map);

        let actual = map.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice".into(),
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn map_with_extra_fields_fails_for_deny_unknown_fields() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Person {
            name: String,
            age: u8,
        }

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
            (BoltString::from("bar"), BoltType::from(1337)),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let map = BoltType::Map(map);

        assert!(map.to::<Person>().is_err());
    }

    #[test]
    fn simple_struct() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice".into(),
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn tuple_struct_from_list() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person(String, u8);

        let list = BoltType::from(vec![BoltType::from("Alice"), BoltType::from(42)]);
        let actual = list.to::<Person>().unwrap();
        let expected = Person("Alice".into(), 42);

        assert_eq!(actual, expected);
    }

    #[test]
    fn tuple_struct_from_map_fails() {
        // We do not support this since maps are unordered and
        // we cannot gurantee that the values are in the same
        // order as the tuple struct fields.
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person(String, u8);

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let map = BoltType::Map(map);

        assert!(map.to::<Person>().is_err());
    }

    #[test]
    fn node() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let id = BoltInteger::new(42);
        let labels = vec!["Person".into()].into();
        let properties = vec![
            ("name".into(), "Alice".into()),
            ("age".into(), 42_u16.into()),
        ]
        .into_iter()
        .collect();

        let node = BoltNode {
            id,
            labels,
            properties,
        };
        let node = BoltType::Node(node);

        let actual = node.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice".to_owned(),
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn relation() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Knows {
            since: u16,
        }

        let id = BoltInteger::new(42);
        let start_node_id = BoltInteger::new(13);
        let end_node_id = BoltInteger::new(37);
        let typ = BoltString::new("REL");
        let properties = vec![("since".into(), 1337_u16.into())]
            .into_iter()
            .collect();

        let relation = BoltRelation {
            id,
            start_node_id,
            end_node_id,
            typ,
            properties,
        };
        let relation = BoltType::Relation(relation);

        let actual = relation.to::<Knows>().unwrap();
        let expected = Knows { since: 1337 };

        assert_eq!(actual, expected);
    }

    #[test]
    fn tuple() {
        let list = BoltType::from(vec![BoltType::from("Alice"), BoltType::from(42)]);
        let actual = list.to::<(String, u8)>().unwrap();
        let expected = ("Alice".into(), 42);

        assert_eq!(actual, expected);
    }

    #[test]
    fn borrowing_struct() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<'a> {
            name: &'a str,
            age: u8,
        }

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice",
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn various_types() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Bag<'a> {
            borrowed: &'a str,
            owned: String,

            float: f64,
            int: i32,
            long: i64,

            boolean: bool,

            unit: (),
        }

        let map = [
            (
                BoltString::from("borrowed"),
                BoltType::from("I am borrowed"),
            ),
            (
                BoltString::from("owned"),
                BoltType::from("I am cloned and owned"),
            ),
            (BoltString::from("float"), BoltType::from(13.37)),
            (BoltString::from("int"), BoltType::from(42_i32)),
            (BoltString::from("long"), BoltType::from(1337_i64)),
            (BoltString::from("boolean"), BoltType::from(true)),
            (
                BoltString::from("unit"),
                BoltType::Null(BoltNull::default()),
            ),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Bag>().unwrap();
        let expected = Bag {
            borrowed: "I am borrowed",
            owned: "I am cloned and owned".to_owned(),
            float: 13.37,
            int: 42,
            long: 1337,
            boolean: true,
            unit: (),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn std_bytes() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Bytes<'a> {
            bytes: Vec<u8>,
            slice: &'a [u8],
        }

        let map = [
            (BoltString::from("bytes"), BoltType::from(vec![4_u8, 2])),
            (
                BoltString::from("slice"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Bytes>().unwrap();
        let expected = Bytes {
            bytes: vec![4, 2],
            slice: &[1, 3, 3, 7],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn serde_bytes_bytes() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Bytes<'a> {
            #[serde(with = "serde_bytes")]
            bytes: Vec<u8>,
            #[serde(with = "serde_bytes")]
            slice: &'a [u8],
        }

        let map = [
            (BoltString::from("bytes"), BoltType::from(vec![4_u8, 2])),
            (
                BoltString::from("slice"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Bytes>().unwrap();
        let expected = Bytes {
            bytes: vec![4, 2],
            slice: &[1, 3, 3, 7],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn serde_with_bytes() {
        use serde_with::{serde_as, Bytes};

        #[serde_as]
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct AsBytes<'a> {
            #[serde_as(as = "Bytes")]
            array: [u8; 4],

            #[serde_as(as = "Bytes")]
            boxed: Box<[u8]>,

            #[serde_as(as = "Bytes")]
            #[serde(borrow)]
            cow: Cow<'a, [u8]>,

            #[serde_as(as = "Bytes")]
            #[serde(borrow)]
            cow_array: Cow<'a, [u8; 2]>,

            #[serde_as(as = "Bytes")]
            bytes: Vec<u8>,

            #[serde_as(as = "Bytes")]
            slice: &'a [u8],
        }

        let map = [
            (
                BoltString::from("array"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
            (BoltString::from("boxed"), BoltType::from(vec![4_u8, 2])),
            (BoltString::from("cow"), BoltType::from(vec![1_u8, 3, 3, 7])),
            (BoltString::from("cow_array"), BoltType::from(vec![4_u8, 2])),
            (
                BoltString::from("bytes"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
            (BoltString::from("slice"), BoltType::from(vec![4_u8, 2])),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<AsBytes>().unwrap();
        let expected = AsBytes {
            array: [1, 3, 3, 7],
            boxed: vec![4, 2].into_boxed_slice(),
            cow: vec![1_u8, 3, 3, 7].into(),
            cow_array: Cow::Owned([4_u8, 2]),
            bytes: vec![1, 3, 3, 7],
            slice: &[4, 2],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn nested_struct() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: u32,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Couple {
            p0: Person,
            p1: Person,
        }

        let map = [
            (
                BoltString::from("p0"),
                BoltType::Map(
                    [
                        (BoltString::from("name"), BoltType::from("Alice")),
                        (BoltString::from("age"), BoltType::from(42)),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ),
            (
                BoltString::from("p1"),
                BoltType::Map(
                    [
                        (BoltString::from("name"), BoltType::from("Bob")),
                        (BoltString::from("age"), BoltType::from(1337)),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Couple>().unwrap();
        let expected = Couple {
            p0: Person {
                name: "Alice".into(),
                age: 42,
            },
            p1: Person {
                name: "Bob".into(),
                age: 1337,
            },
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn list() {
        let list = BoltType::from(vec![42_i64, 1337]);
        let actual = list.to::<Vec<i64>>().unwrap();

        assert_eq!(actual, vec![42_i64, 1337]);
    }

    #[test]
    fn nested_list() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Foo {
            bars: Vec<i64>,
        }

        let data = [(BoltString::from("bars"), BoltType::from(vec![42, 1337]))]
            .into_iter()
            .collect::<BoltMap>();
        let actual = data.to::<Foo>().unwrap();
        let expected = Foo {
            bars: vec![42, 1337],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn type_convert() {
        let i = BoltType::from(42);

        assert_eq!(i.to::<i8>().unwrap(), 42);
    }

    #[test]
    fn type_convert_error() {
        let i = BoltType::from(1337);

        assert_eq!(
            i.to::<i8>().unwrap_err().to_string(),
            "Could not convert the integer `1337` to the target type i8"
        );
    }
}
