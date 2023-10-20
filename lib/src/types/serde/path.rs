use crate::{
    cenum,
    types::{
        serde::{
            builder::BoltPathBuilder,
            element::{ElementDataDeserializer, ElementDataKey},
            BoltKind,
        },
        BoltList, BoltPath, BoltType,
    },
    DeError, Indices, Nodes, Path, Relationships,
};

use std::{fmt, result::Result};

use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        DeserializeSeed, Deserializer, EnumAccess, Error, IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BoltPath::deserialize(deserializer).map(Path::new)
    }
}

impl BoltPath {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
    }
}

cenum!(Fields {
    Nodes,
    Relationships,
    Indices,
});

impl<'de> Deserialize<'de> for BoltPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["nodes", "relationships", "indices"];

        struct BoltPathVisitor;

        impl<'de> Visitor<'de> for BoltPathVisitor {
            type Value = BoltPath;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BoltPath")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                let mut path = BoltPathBuilder::default();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "nodes" => path.nodes(|| {
                            Ok(BoltList {
                                value: map.next_value::<Nodes<BoltType>>()?.0,
                            })
                        })?,
                        "relationships" => path.relations(|| {
                            Ok(BoltList {
                                value: map.next_value::<Relationships<BoltType>>()?.0,
                            })
                        })?,
                        "indices" => path.indices(|| {
                            Ok(BoltList {
                                value: map.next_value::<Indices<BoltType>>()?.0,
                            })
                        })?,
                        otherwise => return Err(Error::unknown_field(otherwise, FIELDS)),
                    }
                }

                path.build()
            }
        }

        deserializer.deserialize_struct("BoltPath", FIELDS, BoltPathVisitor)
    }
}

pub struct BoltPathVisitor;

impl<'de> Visitor<'de> for BoltPathVisitor {
    type Value = BoltPath;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct BoltPath")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut path = BoltPathBuilder::default();

        while let Some(key) = map.next_key::<ElementDataKey>()? {
            match key {
                ElementDataKey::Nodes => path.nodes(|| map.next_value())?,
                ElementDataKey::Relationships => path.relations(|| map.next_value())?,
                ElementDataKey::Indices => path.indices(|| map.next_value())?,
                otherwise => {
                    return Err(Error::unknown_field(
                        otherwise.name(),
                        &["nodes", "relationships", "indices"],
                    ))
                }
            }
        }

        path.build()
    }
}

pub struct BoltPathDeserializer<'de>(&'de BoltPath);

impl<'de> BoltPathDeserializer<'de> {
    fn new(node: &'de BoltPath) -> Self {
        Self(node)
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltList {
    type Deserializer = SeqDeserializer<std::slice::Iter<'de, BoltType>, DeError>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.value.iter())
    }
}

impl<'de> Deserializer<'de> for BoltPathDeserializer<'de> {
    type Error = DeError;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(
            [
                ("nodes", &self.0.nodes),
                ("relationships", &self.0.rels),
                ("indices", &self.0.indices),
            ]
            .into_iter(),
        ))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        ElementDataDeserializer::new(self.0).deserialize_outer_struct(fields, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        ElementDataDeserializer::new(self.0).deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple tuple_struct identifier
    }
}

impl<'de> EnumAccess<'de> for BoltPathDeserializer<'de> {
    type Error = DeError;

    type Variant = ElementDataDeserializer<'de, &'de BoltPath>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let kind = BoltKind::Path;
        let val = seed.deserialize(kind.into_deserializer())?;
        Ok((val, ElementDataDeserializer::new(self.0)))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltPath {
    type Deserializer = BoltPathDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltPathDeserializer::new(self)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::{
        types::{BoltInteger, BoltNode, BoltString, BoltType, BoltUnboundedRelation},
        Node, UnboundedRelation,
    };

    use super::*;

    fn test_path() -> BoltPath {
        let alice = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into()), ("age".into(), 42.into())]
                .into_iter()
                .collect(),
        );
        let bob = BoltNode::new(
            BoltInteger::new(1337),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Bob".into()), ("age".into(), 84.into())]
                .into_iter()
                .collect(),
        );
        let rel = BoltUnboundedRelation::new(
            BoltInteger::new(84),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );
        BoltPath {
            nodes: BoltList::from(vec![BoltType::Node(alice), BoltType::Node(bob)]),
            rels: BoltList::from(vec![BoltType::UnboundedRelation(rel)]),
            indices: BoltList::from(vec![BoltType::from(1), BoltType::from(1)]),
        }
    }

    #[test]
    fn path_nodes() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        test_extract_path(Nodes(vec![
            Person {
                name: "Alice".into(),
                age: 42,
            },
            Person {
                name: "Bob".into(),
                age: 84,
            },
        ]));
    }

    #[test]
    fn path_rels() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Knows {
            since: u16,
        }

        test_extract_path(Relationships(vec![Knows { since: 2017 }]));
    }

    #[test]
    fn path_indices() {
        test_extract_path(Indices(vec![1, 1]));
    }

    #[test]
    fn path_all() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Knows {
            since: u16,
        }

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Path {
            nodes: Nodes<Person>,
            rels: Relationships<Knows>,
            indices: Indices,
        }

        test_extract_path(Path {
            nodes: Nodes(vec![
                Person {
                    name: "Alice".into(),
                    age: 42,
                },
                Person {
                    name: "Bob".into(),
                    age: 84,
                },
            ]),
            rels: Relationships(vec![Knows { since: 2017 }]),
            indices: Indices(vec![1, 1]),
        });
    }

    fn test_extract_path<T: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: T) {
        let path = test_path();
        let actual = path.to::<T>().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn path_to_bolt_type() {
        let path = test_path();
        let actual = path.to::<BoltType>().unwrap();
        assert_eq!(actual, BoltType::Path(path));
    }

    #[test]
    fn path_to_bolt_path() {
        let path = test_path();
        let actual = path.to::<BoltPath>().unwrap();
        assert_eq!(actual, path);
    }

    #[test]
    fn path_to_path() {
        let path = test_path();
        let actual = path.to::<Path>().unwrap();

        let nodes = path.nodes().into_iter().map(Node::new).collect::<Vec<_>>();
        let relationships = path
            .rels()
            .into_iter()
            .map(UnboundedRelation::new)
            .collect::<Vec<_>>();

        assert_eq!(actual.nodes(), nodes);
        assert_eq!(actual.rels(), relationships);
        assert_eq!(actual.indices(), vec![1, 1]);
    }
}
