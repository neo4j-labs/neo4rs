use ::serde::{
    de::{self, value::StrDeserializer},
    forward_to_deserialize_any, Deserialize,
};

use crate::types::{BoltMap, BoltString, BoltType};
use std::collections::HashMap;

impl BoltMap {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        let deserializer = BoltMapDeserializer::new(self);
        let t = T::deserialize(deserializer)?;
        Ok(t)
    }
}


#[derive(Debug, thiserror::Error)]
pub enum DeError {
    #[error("{0}")]
    Error(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl de::Error for DeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Error(msg.to_string().into())
    }
}

struct BoltMapDeserializer<'de> {
    entries: <&'de HashMap<BoltString, BoltType> as IntoIterator>::IntoIter,
    value: Option<&'de BoltType>,
}

impl<'de> BoltMapDeserializer<'de> {
    fn new(input: &'de BoltMap) -> Self {
        Self {
            entries: input.value.iter(),
            value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for BoltMapDeserializer<'de> {
    type Error = DeError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.entries.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(StrDeserializer::new(&key.value)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(BoltTypeDeserializer::new(value)),
            None => Err(de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.entries.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> de::Deserializer<'de> for BoltMapDeserializer<'de> {
    type Error = DeError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct BoltTypeDeserializer<'de> {
    value: &'de BoltType,
}

impl<'de> BoltTypeDeserializer<'de> {
    fn new(value: &'de BoltType) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for BoltTypeDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        #[allow(unused)]
        match self.value {
            BoltType::String(v) => visitor.visit_borrowed_str(&v.value),
            BoltType::Boolean(v) => visitor.visit_bool(v.value),
            BoltType::Map(v) => visitor.visit_map(BoltMapDeserializer::new(v)),
            BoltType::Null(v) => visitor.visit_unit(),
            BoltType::Integer(v) => visitor.visit_i64(v.value),
            BoltType::Float(v) => visitor.visit_f64(v.value),
            BoltType::List(v) => todo!(),
            BoltType::Node(v) => todo!(),
            BoltType::Relation(v) => todo!(),
            BoltType::UnboundedRelation(v) => todo!(),
            BoltType::Point2D(v) => todo!(),
            BoltType::Point3D(v) => todo!(),
            BoltType::Bytes(v) => todo!(),
            BoltType::Path(v) => todo!(),
            BoltType::Duration(v) => todo!(),
            BoltType::Date(v) => todo!(),
            BoltType::Time(v) => todo!(),
            BoltType::LocalTime(v) => todo!(),
            BoltType::DateTime(v) => todo!(),
            BoltType::LocalDateTime(v) => todo!(),
            BoltType::DateTimeZoneId(v) => todo!(),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::borrow::Cow;

    use super::*;
    use crate::types::BoltNull;

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
}
