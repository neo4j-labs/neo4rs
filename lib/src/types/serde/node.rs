use crate::{
    types::{
        serde::{
            builder::{BoltNodeBuilder, Id},
            element::{ElementDataDeserializer, ElementDataKey},
            BoltKind,
        },
        BoltList, BoltNode, BoltString,
    },
    DeError, Labels, Node,
};

use std::{fmt, result::Result};

use serde::{
    de::{
        value::MapDeserializer, DeserializeSeed, Deserializer, EnumAccess, Error, IntoDeserializer,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BoltNode::deserialize(deserializer).map(Node::new)
    }
}

impl BoltNode {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
    }
}

impl<'de> Deserialize<'de> for BoltNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const ID: &str = "42.<id>";
        const LABELS: &str = "42.<labels>";

        struct BoltNodeVisitor;

        impl<'de> Visitor<'de> for BoltNodeVisitor {
            type Value = BoltNode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BoltNode")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                let mut builder = BoltNodeBuilder::default();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        ID => builder.id(|| map.next_value::<Id>().map(|i| i.0))?,
                        LABELS => {
                            builder.labels(|| map.next_value::<Labels<BoltList>>().map(|l| l.0))?
                        }
                        otherwise => builder
                            .insert(|| Ok((BoltString::from(otherwise), map.next_value()?)))?,
                    }
                }

                let node = builder.build()?;
                Ok(node)
            }
        }

        deserializer.deserialize_struct("BoltNode", &[ID, LABELS], BoltNodeVisitor)
    }
}

pub struct BoltNodeVisitor;

impl<'de> Visitor<'de> for BoltNodeVisitor {
    type Value = BoltNode;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct BoltNode")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut builder = BoltNodeBuilder::default();

        while let Some(key) = map.next_key::<ElementDataKey>()? {
            match key {
                ElementDataKey::Id => builder.id(|| map.next_value())?,
                ElementDataKey::Labels => builder.labels(|| map.next_value())?,
                ElementDataKey::Properties => builder.properties(|| map.next_value())?,
                otherwise => {
                    return Err(Error::unknown_field(
                        otherwise.name(),
                        &["Id", "Labels", "Properties"],
                    ))
                }
            }
        }

        let node = builder.build()?;
        Ok(node)
    }
}

pub struct BoltNodeDeserializer<'de>(&'de BoltNode);

impl<'de> BoltNodeDeserializer<'de> {
    fn new(node: &'de BoltNode) -> Self {
        Self(node)
    }
}

impl<'de> Deserializer<'de> for BoltNodeDeserializer<'de> {
    type Error = DeError;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(self.0.properties.value.iter()))
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

impl<'de> EnumAccess<'de> for BoltNodeDeserializer<'de> {
    type Error = DeError;

    type Variant = ElementDataDeserializer<'de, &'de BoltNode>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let kind = BoltKind::Node;
        let val = seed.deserialize(kind.into_deserializer())?;
        Ok((val, ElementDataDeserializer::new(self.0)))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltNode {
    type Deserializer = BoltNodeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltNodeDeserializer::new(self)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fmt::Debug,
        marker::PhantomData,
        sync::atomic::{AtomicU32, Ordering},
    };

    use crate::{
        types::{BoltInteger, BoltType},
        Id, Keys,
    };

    use super::*;

    use tap::Tap;

    fn test_node() -> BoltNode {
        let id = BoltInteger::new(1337);
        let labels = vec!["Person".into()].into();
        let properties = vec![
            ("name".into(), "Alice".into()),
            ("age".into(), 42_u16.into()),
        ]
        .into_iter()
        .collect();

        BoltNode {
            id,
            labels,
            properties,
        }
    }

    #[test]
    fn node() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        test_extract_node(Person {
            name: "Alice".into(),
            age: 42,
        });
    }

    #[test]
    fn extract_node_with_unit_types() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<T> {
            name: String,
            age: u8,
            _t: PhantomData<T>,
            _u: (),
        }

        test_extract_node(Person {
            name: "Alice".to_owned(),
            age: 42,
            _t: PhantomData::<i32>,
            _u: (),
        });
    }

    #[test]
    fn extract_missing_properties_with_option() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            favorite_rust_crate: Option<String>,
        }

        test_extract_node(Person {
            favorite_rust_crate: None,
        });
    }

    #[test]
    fn extract_missing_properties_with_default() {
        fn favorite_rust_crate() -> String {
            "graph".to_owned()
        }

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            #[serde(default = "favorite_rust_crate")]
            favorite_rust_crate: String,
        }

        let node = test_node();
        let actual = node.to::<Person>().unwrap_err();
        assert!(matches!(actual, DeError::PropertyMissingButRequired));
    }

    #[test]
    fn extract_node_id() {
        test_extract_node_extra(Id(1337));
    }

    #[test]
    fn extract_node_id_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Id(i16);

        test_extract_node_extra(Id(1337));
    }

    #[test]
    fn extract_node_id_with_custom_struct() {
        #[derive(Debug, Deserialize)]
        struct Id {
            id: AtomicU32,
        }

        impl PartialEq for Id {
            fn eq(&self, other: &Self) -> bool {
                self.id.load(Ordering::SeqCst) == other.id.load(Ordering::SeqCst)
            }
        }

        test_extract_node_extra(Id { id: 1337.into() });
    }

    #[test]
    fn extract_node_labels() {
        test_extract_node_extra(Labels(vec!["Person".to_owned()]));
    }

    #[test]
    fn extract_node_property_custom_labels_collection() {
        test_extract_node_extra(Labels([String::from("Person")]));
    }

    #[test]
    fn extract_node_labels_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Labels([String; 1]);

        test_extract_node_extra(Labels(["Person".to_owned()]));
    }

    #[test]
    fn extract_node_labels_with_custom_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Labels {
            labels: Vec<String>,
        }

        test_extract_node_extra(Labels {
            labels: vec!["Person".to_owned()],
        });
    }

    #[test]
    fn extract_node_labels_borrowed() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Labels<'a>(#[serde(borrow)] Vec<&'a str>);

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<'a> {
            #[serde(borrow)]
            labels: Labels<'a>,
            name: String,
            age: u8,
        }

        let expected = Person {
            labels: Labels(vec!["Person"]),
            name: "Alice".to_owned(),
            age: 42,
        };

        let node = test_node();
        let actual = node.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_node_property_keys() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            keys: Keys,
        }

        let expected = Person {
            keys: Keys(["name".to_owned(), "age".to_owned()].into()),
        };

        test_extract_node(expected);
    }

    #[test]
    fn extract_node_property_keys_custom_vec() {
        #[derive(Clone, Debug, Eq, Deserialize)]
        #[serde(transparent)]
        struct UnorderedVec(Vec<String>);

        impl PartialEq for UnorderedVec {
            fn eq(&self, other: &Self) -> bool {
                // compare on sorted vectors to ignore
                // order on comparison
                let mut lhs = self.0.clone();
                lhs.sort();

                let mut rhs = other.0.clone();
                rhs.sort();

                lhs == rhs
            }
        }

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            keys: Keys<UnorderedVec>,
        }

        let expected = Person {
            keys: Keys(UnorderedVec(vec!["name".to_owned(), "age".to_owned()])),
        };

        test_extract_node(expected);
    }

    #[test]
    fn extract_node_property_keys_custom_struct() {
        #[derive(Clone, Debug, Eq, Deserialize)]
        struct Keys {
            keys: Vec<String>,
        }

        impl PartialEq for Keys {
            fn eq(&self, other: &Self) -> bool {
                // since we cannot gurantee the order of the keys
                // we have to sort them before comparing
                let mut lhs = self.keys.clone();
                lhs.sort();

                let mut rhs = other.keys.clone();
                rhs.sort();

                lhs == rhs
            }
        }

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            property_keys: Keys,
        }

        let expected = Person {
            property_keys: Keys {
                keys: vec!["name".to_owned(), "age".to_owned()],
            },
        };

        test_extract_node(expected);
    }

    #[test]
    fn extract_node_property_keys_borrowed() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Keys<'a>(#[serde(borrow)] HashSet<&'a str>);

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<'a> {
            #[serde(borrow)]
            keys: Keys<'a>,
        }

        let expected = Person {
            keys: Keys(["age", "name"].into()),
        };

        let node = test_node();
        let actual = node.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    fn test_extract_node_extra<T: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: T) {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<T> {
            extra: T,
            name: String,
            age: u8,
        }

        let expected = Person {
            extra: expected,
            name: "Alice".to_owned(),
            age: 42,
        };

        test_extract_node(expected);
    }

    fn test_extract_node<Person: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: Person) {
        let node = test_node();
        let actual = node.to::<Person>().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_just_extract_node_extra() {
        let node = test_node();

        let id = node.to::<Id>().unwrap();
        let labels = node.to::<Labels>().unwrap();
        let keys = node.to::<Keys>().unwrap();

        assert_eq!(id, Id(1337));
        assert_eq!(labels, Labels(vec!["Person".to_owned()]));
        assert_eq!(keys, Keys(["name".to_owned(), "age".to_owned()].into()));
    }

    #[test]
    fn node_to_bolt_type() {
        let node = test_node();
        let actual = node.to::<BoltType>().unwrap();
        assert_eq!(actual, BoltType::Node(node));
    }

    #[test]
    fn node_to_bolt_node() {
        let node = test_node();
        let actual = node.to::<BoltNode>().unwrap();
        assert_eq!(actual, node);
    }

    #[test]
    fn node_to_node() {
        let node = test_node();
        let actual = node.to::<Node>().unwrap();
        assert_eq!(actual.id(), node.id.value);
        assert_eq!(
            actual.labels().tap_mut(|v| v.sort_unstable()),
            node.labels
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .tap_mut(|v| v.sort_unstable())
        );
        assert_eq!(
            actual.keys().tap_mut(|v| v.sort_unstable()),
            node.properties
                .value
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .tap_mut(|v| v.sort_unstable())
        );
    }
}
