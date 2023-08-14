use crate::types::{
    serde::DeError, BoltList, BoltMap, BoltNode, BoltPath, BoltPoint2D, BoltPoint3D, BoltRelation,
    BoltType, BoltUnboundedRelation,
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
        self.attributes.to::<T>()
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

    /// Get the attributes of the node
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
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

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
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

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
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

    use crate::types::BoltString;

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
