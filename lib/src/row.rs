use crate::{
    types::{
        serde::DeError, BoltList, BoltMap, BoltNode, BoltPath, BoltPoint2D, BoltPoint3D,
        BoltRelation, BoltUnboundedRelation,
    },
    BoltType,
};

use serde::Deserialize;
use std::convert::TryInto;

/// Represents a row returned as a result of executing a query.
///
/// A row is very similar to a `HashMap`, you can get the attributes using [`Row::get`] method.
#[derive(Debug)]
pub struct Row {
    attributes: BoltMap,
}

/// Snapshot of a node within a graph database
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    inner: BoltNode,
}

/// Alternating sequence of nodes and relationships
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    inner: BoltPath,
}

/// Snapshot of a relationship within a graph database
#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    inner: BoltRelation,
}

/// Relationship detail without start or end node information
#[derive(Debug, Clone, PartialEq)]
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

    #[deprecated(since = "0.7.0", note = "Please use `indices` instead.")]
    pub fn ids(&self) -> Vec<i64> {
        self.indices()
    }

    pub fn indices(&self) -> Vec<i64> {
        self.indices_as().unwrap()
    }

    pub fn nodes(&self) -> Vec<Node> {
        self.nodes_as().unwrap()
    }

    pub fn rels(&self) -> Vec<UnboundedRelation> {
        self.relationships_as().unwrap()
    }

    /// Deserialize the path into a custom type that implements [`serde::Deserialize`]
    pub fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.to::<T>()
    }

    /// Deserialize the nodes of this path into custom type that implements [`serde::Deserialize`]
    pub fn nodes_as<'this, T>(&'this self) -> Result<Vec<T>, DeError>
    where
        T: Deserialize<'this>,
    {
        Ok(self.to::<crate::Nodes<T>>()?.0)
    }

    /// Deserialize the relationships of this path into custom type that implements [`serde::Deserialize`]
    pub fn relationships_as<'this, T>(&'this self) -> Result<Vec<T>, DeError>
    where
        T: Deserialize<'this>,
    {
        Ok(self.to::<crate::Relationships<T>>()?.0)
    }

    /// Deserialize the indices of this path into a custom type that implements [`serde::Deserialize`]
    pub fn indices_as<'this, T>(&'this self) -> Result<Vec<T>, DeError>
    where
        T: Deserialize<'this>,
    {
        Ok(self.to::<crate::Indices<T>>()?.0)
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

    /// Get an attribute of this relationship and deserialize it into custom type that implements [`serde::Deserialize`]
    pub fn get<'this, T>(&'this self, key: &str) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.attributes.get::<T>(key)
    }

    pub fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.to_strict::<T>().or_else(|e| match self.single() {
            Some(single) => single.to::<T>(),
            None => Err(e),
        })
    }

    pub fn to_strict<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.attributes.to::<T>()
    }

    fn single(&self) -> Option<&BoltType> {
        let mut values = self.attributes.value.values();
        let first = values.next()?;
        if values.next().is_some() {
            return None;
        }
        Some(first)
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
    pub fn labels(&self) -> Vec<&str> {
        self.to::<crate::Labels<_>>().unwrap().0
    }

    /// Get the names of the attributes of this node
    pub fn keys(&self) -> Vec<&str> {
        self.to::<crate::Keys<_>>().unwrap().0
    }

    /// Get an attribute of this node and deserialize it into custom type that implements [`serde::Deserialize`]
    pub fn get<'this, T>(&'this self, key: &str) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.properties.get::<T>(key)
    }

    /// Deserialize the node into custom type that implements [`serde::Deserialize`]
    pub fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.to::<T>()
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

    pub fn typ(&self) -> &str {
        self.to::<crate::Type<_>>().unwrap().0
    }

    /// Get the names of the attributes of this relationship
    pub fn keys(&self) -> Vec<&str> {
        self.to::<crate::Keys<_>>().unwrap().0
    }

    /// Get an attribute of this relationship and deserialize it into custom type that implements [`serde::Deserialize`]
    pub fn get<'this, T>(&'this self, key: &str) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.properties.get::<T>(key)
    }

    /// Deserialize the relationship into custom type that implements [`serde::Deserialize`]
    pub fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.to::<T>()
    }
}

impl UnboundedRelation {
    pub fn new(inner: BoltUnboundedRelation) -> Self {
        UnboundedRelation { inner }
    }

    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    pub fn typ(&self) -> &str {
        self.to::<crate::Type<_>>().unwrap().0
    }

    /// Get the names of the attributes of this relationship
    pub fn keys(&self) -> Vec<&str> {
        self.to::<crate::Keys<_>>().unwrap().0
    }

    /// Get an attribute of this relationship and deserialize it into custom type that implements [`serde::Deserialize`]
    pub fn get<'this, T>(&'this self, key: &str) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.properties.get::<T>(key)
    }

    /// Deserialize the relationship into custom type that implements [`serde::Deserialize`]
    pub fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        self.inner.to::<T>()
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::types::{BoltString, BoltType};

    use super::*;

    #[test]
    fn row_serializes_from_fields() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Person0 {
            name: String,
            age: i32,
            score: f64,
            awesome: bool,
            #[serde(with = "serde_bytes")]
            data: Vec<u8>,
        }

        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Person1<'a> {
            name: &'a str,
            age: i32,
            score: f64,
            awesome: bool,
            #[serde(with = "serde_bytes")]
            data: &'a [u8],
        }

        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Couple<'a> {
            p0: Person0,
            #[serde(borrow)]
            p1: Person1<'a>,
        }

        let row = {
            let fields = BoltList::from(vec![BoltType::from("p0"), BoltType::from("p1")]);

            let data = BoltList::from(vec![
                BoltType::Map(
                    [
                        (BoltString::from("name"), BoltType::from("Alice")),
                        (BoltString::from("age"), BoltType::from(42)),
                        (BoltString::from("score"), BoltType::from(4.2)),
                        (BoltString::from("awesome"), BoltType::from(true)),
                        (BoltString::from("data"), BoltType::from(vec![4_u8, 2])),
                    ]
                    .into_iter()
                    .collect(),
                ),
                BoltType::Map(
                    [
                        (BoltString::from("name"), BoltType::from("Bob")),
                        (BoltString::from("age"), BoltType::from(1337)),
                        (BoltString::from("score"), BoltType::from(13.37)),
                        (BoltString::from("awesome"), BoltType::from(false)),
                        (
                            BoltString::from("data"),
                            BoltType::from(vec![1_u8, 3, 3, 7]),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ]);
            Row::new(fields, data)
        };

        let actual = row.to::<Couple>().unwrap();
        let expected = Couple {
            p0: Person0 {
                name: "Alice".to_owned(),
                age: 42,
                score: 4.2,
                awesome: true,
                data: vec![4, 2],
            },
            p1: Person1 {
                name: "Bob",
                age: 1337,
                score: 13.37,
                awesome: false, // poor Bob
                data: &[1, 3, 3, 7],
            },
        };

        assert_eq!(actual, expected);
    }
}
