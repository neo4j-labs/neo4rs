use serde::{de::value::StrDeserializer, forward_to_deserialize_any, Deserialize};

use crate::types::*;
use std::{collections::HashMap, convert::TryInto};

/// Represents a row returned as a result of executing a query.
///
/// A row is very similar to a `HashMap`, you can get the attributes using [`Row::get`] method.
#[derive(Debug)]
pub struct Row {
    attributes: BoltMap,
}

/// Snapshot of a node within a graph database
#[derive(Debug)]
pub struct Node {
    inner: BoltNode,
}

/// Alternating sequence of nodes and relationships
#[derive(Debug)]
pub struct Path {
    inner: BoltPath,
}

/// Snapshot of a relationship within a graph database
#[derive(Debug)]
pub struct Relation {
    inner: BoltRelation,
}

/// Relationship detail without start or end node information
#[derive(Debug)]
pub struct UnboundedRelation {
    inner: BoltUnboundedRelation,
}

/// Represents a single location in 2-dimensional space
pub struct Point2D {
    inner: BoltPoint2D,
}

/// Represents a single location in 3-dimensional space
pub struct Point3D {
    inner: BoltPoint3D,
}

impl Path {
    pub fn new(inner: BoltPath) -> Self {
        Path { inner }
    }

    pub fn ids(&self) -> Vec<i64> {
        let bolt_ids = self.inner.ids();
        bolt_ids.into_iter().map(|id| id.value).collect()
    }

    pub fn nodes(&self) -> Vec<Node> {
        let nodes = self.inner.nodes();
        nodes.into_iter().map(Node::new).collect()
    }

    pub fn rels(&self) -> Vec<UnboundedRelation> {
        let rels = self.inner.rels();
        rels.into_iter().map(UnboundedRelation::new).collect()
    }
}

impl Point2D {
    pub fn new(inner: BoltPoint2D) -> Self {
        Point2D { inner }
    }

    /// Spatial refrerence system identifier, see <https://en.wikipedia.org/wiki/Spatial_reference_system#Identifier>
    pub fn sr_id(&self) -> i64 {
        self.inner.sr_id.value
    }

    pub fn x(&self) -> f64 {
        self.inner.x.value
    }

    pub fn y(&self) -> f64 {
        self.inner.y.value
    }
}

impl Point3D {
    pub fn new(inner: BoltPoint3D) -> Self {
        Point3D { inner }
    }

    /// Spatial refrerence system identifier, see <https://en.wikipedia.org/wiki/Spatial_reference_system#Identifier>
    pub fn sr_id(&self) -> i64 {
        self.inner.sr_id.value
    }

    pub fn x(&self) -> f64 {
        self.inner.x.value
    }

    pub fn y(&self) -> f64 {
        self.inner.y.value
    }

    pub fn z(&self) -> f64 {
        self.inner.z.value
    }
}

impl Row {
    pub fn new(fields: BoltList, data: BoltList) -> Self {
        let mut attributes = BoltMap::with_capacity(fields.len());
        for (field, value) in fields.into_iter().zip(data.into_iter()) {
            if let Ok(key) = field.try_into() {
                attributes.put(key, value);
            }
        }
        Row { attributes }
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.attributes.get(key)
    }

    pub fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        let deserializer = BoltMapDeserializer::new(&self.attributes);
        let t = T::deserialize(deserializer)?;
        Ok(t)
    }
}

impl Node {
    pub fn new(inner: BoltNode) -> Self {
        Node { inner }
    }

    /// Id of the node
    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    /// various labels attached to this node
    pub fn labels(&self) -> Vec<String> {
        self.inner.labels.iter().map(|l| l.to_string()).collect()
    }

    /// Get the attributes of the node
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }
}

impl Relation {
    pub fn new(inner: BoltRelation) -> Self {
        Relation { inner }
    }

    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    pub fn start_node_id(&self) -> i64 {
        self.inner.start_node_id.value
    }

    pub fn end_node_id(&self) -> i64 {
        self.inner.end_node_id.value
    }

    pub fn typ(&self) -> String {
        self.inner.typ.value.clone()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }
}

impl UnboundedRelation {
    pub fn new(inner: BoltUnboundedRelation) -> Self {
        UnboundedRelation { inner }
    }

    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    pub fn typ(&self) -> String {
        self.inner.typ.value.clone()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeError {
    #[error("{0}")]
    Error(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl serde::de::Error for DeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Error(msg.to_string().into())
    }
}

struct BoltMapDeserializer<'de> {
    map: <&'de HashMap<BoltString, BoltType> as IntoIterator>::IntoIter,
    value: Option<&'de BoltType>,
}

impl<'de> BoltMapDeserializer<'de> {
    fn new(input: &'de BoltMap) -> Self {
        Self {
            map: input.value.iter(),
            value: None,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for BoltMapDeserializer<'de> {
    type Error = DeError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.map.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(StrDeserializer::new(&key.value)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => Ok(seed.deserialize(BoltTypeDeserializer::new(value))?),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.map.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::de::Deserializer<'de> for BoltMapDeserializer<'de> {
    type Error = DeError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
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

impl<'de> serde::de::Deserializer<'de> for BoltTypeDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
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
            BoltType::Bytes(v) => visitor.visit_bytes(&v.value),
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

    use super::*;

    #[test]
    fn test_person() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }
        let row = {
            let name_field = BoltType::from("name");
            let age_field = BoltType::from("age");

            let fields = BoltList::from(vec![name_field, age_field]);

            let name = BoltType::from("Alice");
            let age = BoltType::from(42);

            let data = BoltList::from(vec![name, age]);
            Row::new(fields, data)
        };

        let actual = row.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice".into(),
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_borrowed_person() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<'a> {
            #[serde(borrow)]
            name: &'a str,
            age: u8,
        }

        let row = {
            let name_field = BoltType::from("name");
            let age_field = BoltType::from("age");

            let fields = BoltList::from(vec![name_field, age_field]);

            let name = BoltType::from("Alice");
            let age = BoltType::from(42);

            let data = BoltList::from(vec![name, age]);
            Row::new(fields, data)
        };

        let actual = row.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice",
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_more_types() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Bag<'a> {
            #[serde(borrow)]
            borrowed: &'a str,

            owned: String,

            float: f64,
            int: i32,
            long: i64,

            boolean: bool,

            unit: (),
        }

        let row = {
            let fields = BoltList::from(vec![
                BoltType::from("borrowed"),
                BoltType::from("owned"),
                BoltType::from("float"),
                BoltType::from("int"),
                BoltType::from("long"),
                BoltType::from("boolean"),
                BoltType::from("unit"),
            ]);

            let data = BoltList::from(vec![
                BoltType::from("I am borrowed"),
                BoltType::from("I am cloned and owned"),
                BoltType::from(13.37),
                BoltType::from(42_i32),
                BoltType::from(1337_i64),
                BoltType::from(true),
                BoltType::Null(BoltNull::default()),
            ]);
            Row::new(fields, data)
        };

        let actual = row.to::<Bag>().unwrap();
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
    fn test_nested_structs() {
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

        let row = {
            let fields = BoltList::from(vec![BoltType::from("p0"), BoltType::from("p1")]);

            let data = BoltList::from(vec![
                BoltType::Map(
                    [
                        (BoltString::from("name"), BoltType::from("Alice")),
                        (BoltString::from("age"), BoltType::from(42)),
                    ]
                    .into_iter()
                    .collect(),
                ),
                BoltType::Map(
                    [
                        (BoltString::from("name"), BoltType::from("Bob")),
                        (BoltString::from("age"), BoltType::from(1337)),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]);
            Row::new(fields, data)
        };

        let actual = row.to::<Couple>().unwrap();
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
