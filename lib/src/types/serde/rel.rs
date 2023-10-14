use crate::{
    types::{
        serde::{
            builder::{BoltRelationBuilder, EndNodeId, Id, StartNodeId},
            element::{ElementDataDeserializer, ElementDataKey},
            BoltKind,
        },
        BoltRelation, BoltString,
    },
    DeError, Relation, Type,
};

use std::{fmt, result::Result};

use serde::{
    de::{
        value::MapDeserializer, DeserializeSeed, Deserializer, EnumAccess, Error, IntoDeserializer,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

impl<'de> Deserialize<'de> for Relation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BoltRelation::deserialize(deserializer).map(Relation::new)
    }
}

impl BoltRelation {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
    }
}

impl<'de> Deserialize<'de> for BoltRelation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const ID: &str = "42.<id>";
        const SID: &str = "42.<start_node_id>";
        const EID: &str = "42.<end_node_id>";
        const TYP: &str = "42.<type>";

        struct BoltRelationVisitor;

        impl<'de> Visitor<'de> for BoltRelationVisitor {
            type Value = BoltRelation;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BoltRelation")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                let mut builder = BoltRelationBuilder::default();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        ID => builder.id(|| map.next_value::<Id>().map(|i| i.0))?,
                        SID => builder
                            .start_node_id(|| map.next_value::<StartNodeId>().map(|i| i.0))?,
                        EID => {
                            builder.end_node_id(|| map.next_value::<EndNodeId>().map(|i| i.0))?
                        }
                        TYP => builder.typ(|| map.next_value::<Type<BoltString>>().map(|t| t.0))?,
                        otherwise => builder
                            .insert(|| Ok((BoltString::from(otherwise), map.next_value()?)))?,
                    }
                }

                let node = builder.build()?;
                Ok(node)
            }
        }

        deserializer.deserialize_struct("BoltRelation", &[ID, SID, EID, TYP], BoltRelationVisitor)
    }
}

pub struct BoltRelationVisitor;

impl<'de> Visitor<'de> for BoltRelationVisitor {
    type Value = BoltRelation;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct BoltRelation")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut builder = BoltRelationBuilder::default();

        while let Some(key) = map.next_key::<ElementDataKey>()? {
            match key {
                ElementDataKey::Id => builder.id(|| map.next_value())?,
                ElementDataKey::StartNodeId => builder.start_node_id(|| map.next_value())?,
                ElementDataKey::EndNodeId => builder.end_node_id(|| map.next_value())?,
                ElementDataKey::Type => builder.typ(|| map.next_value())?,
                ElementDataKey::Properties => builder.properties(|| map.next_value())?,
                otherwise => {
                    return Err(Error::unknown_field(
                        otherwise.name(),
                        &["Id", "StartNodeId", "EndNodeId", "Type", "Properties"],
                    ))
                }
            }
        }

        let node = builder.build()?;
        Ok(node)
    }
}

pub struct BoltRelationDeserializer<'de>(&'de BoltRelation);

impl<'de> BoltRelationDeserializer<'de> {
    fn new(node: &'de BoltRelation) -> Self {
        Self(node)
    }
}

impl<'de> Deserializer<'de> for BoltRelationDeserializer<'de> {
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

impl<'de> EnumAccess<'de> for BoltRelationDeserializer<'de> {
    type Error = DeError;

    type Variant = ElementDataDeserializer<'de, &'de BoltRelation>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let kind = BoltKind::Relation;
        let val = seed.deserialize(kind.into_deserializer())?;
        Ok((val, ElementDataDeserializer::new(self.0)))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltRelation {
    type Deserializer = BoltRelationDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltRelationDeserializer::new(self)
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
        EndNodeId, Id, Keys, StartNodeId,
    };

    use super::*;

    use tap::Tap;

    fn test_relation() -> BoltRelation {
        let id = BoltInteger::new(1337);
        let start_node_id = BoltInteger::new(21);
        let end_node_id = BoltInteger::new(84);
        let typ = "Person".into();
        let properties = vec![
            ("name".into(), "Alice".into()),
            ("age".into(), 42_u16.into()),
        ]
        .into_iter()
        .collect();

        BoltRelation {
            id,
            start_node_id,
            end_node_id,
            properties,
            typ,
        }
    }

    #[test]
    fn relation() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        test_extract_relation(Person {
            name: "Alice".into(),
            age: 42,
        });
    }

    #[test]
    fn extract_relation_with_unit_types() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<T> {
            name: String,
            age: u8,
            _t: PhantomData<T>,
            _u: (),
        }

        test_extract_relation(Person {
            name: "Alice".to_owned(),
            age: 42,
            _t: PhantomData::<i32>,
            _u: (),
        });
    }

    #[test]
    fn extract_relation_id() {
        test_extract_relation_extra(Id(1337));
    }

    #[test]
    fn extract_relation_id_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Id(i16);

        test_extract_relation_extra(Id(1337));
    }

    #[test]
    fn extract_relation_id_with_custom_struct() {
        #[derive(Debug, Deserialize)]
        struct Id {
            id: AtomicU32,
        }

        impl PartialEq for Id {
            fn eq(&self, other: &Self) -> bool {
                self.id.load(Ordering::SeqCst) == other.id.load(Ordering::SeqCst)
            }
        }

        test_extract_relation_extra(Id { id: 1337.into() });
    }

    #[test]
    fn extract_relation_start_node_id() {
        test_extract_relation_extra(StartNodeId(21));
    }

    #[test]
    fn extract_relation_start_node_id_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct StartNodeId(i16);

        test_extract_relation_extra(StartNodeId(21));
    }

    #[test]
    fn extract_relation_start_node_id_with_custom_struct() {
        #[derive(Debug, Deserialize)]
        struct StartNodeId {
            id: AtomicU32,
        }

        impl PartialEq for StartNodeId {
            fn eq(&self, other: &Self) -> bool {
                self.id.load(Ordering::SeqCst) == other.id.load(Ordering::SeqCst)
            }
        }

        test_extract_relation_extra(StartNodeId { id: 21.into() });
    }

    #[test]
    fn extract_relation_end_node_id() {
        test_extract_relation_extra(EndNodeId(84));
    }

    #[test]
    fn extract_relation_end_node_id_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct EndNodeId(i16);

        test_extract_relation_extra(EndNodeId(84));
    }

    #[test]
    fn extract_relation_end_node_id_with_custom_struct() {
        #[derive(Debug, Deserialize)]
        struct EndNodeId {
            id: AtomicU32,
        }

        impl PartialEq for EndNodeId {
            fn eq(&self, other: &Self) -> bool {
                self.id.load(Ordering::SeqCst) == other.id.load(Ordering::SeqCst)
            }
        }

        test_extract_relation_extra(EndNodeId { id: 84.into() });
    }

    #[test]
    fn extract_relation_type() {
        test_extract_relation_extra(Type("Person".to_owned()));
    }

    #[test]
    fn extract_relation_type_custom_inner() {
        test_extract_relation_extra(Type::<Box<str>>("Person".into()));
    }

    #[test]
    fn extract_relation_type_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Type(String);

        test_extract_relation_extra(Type("Person".to_owned()));
    }

    #[test]
    fn extract_relation_type_with_custom_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Type {
            types: String,
        }

        test_extract_relation_extra(Type {
            types: "Person".to_owned(),
        });
    }

    #[test]
    fn extract_relation_type_borrowed() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Type<'a>(#[serde(borrow)] &'a str);

        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<'a> {
            #[serde(borrow)]
            type_: Type<'a>,
            name: String,
            age: u8,
        }

        let expected = Person {
            type_: Type("Person"),
            name: "Alice".to_owned(),
            age: 42,
        };

        let relation = test_relation();
        let actual = relation.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_relation_property_keys() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            keys: Keys,
        }

        let expected = Person {
            keys: Keys(["name".to_owned(), "age".to_owned()].into()),
        };

        test_extract_relation(expected);
    }

    #[test]
    fn extract_relation_property_keys_custom_vec() {
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

        test_extract_relation(expected);
    }

    #[test]
    fn extract_relation_property_keys_custom_struct() {
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

        test_extract_relation(expected);
    }

    #[test]
    fn extract_relation_property_keys_borrowed() {
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

        let relation = test_relation();
        let actual = relation.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    fn test_extract_relation_extra<T: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: T) {
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

        test_extract_relation(expected);
    }

    fn test_extract_relation<Person: Debug + PartialEq + for<'a> Deserialize<'a>>(
        expected: Person,
    ) {
        let relation = test_relation();
        let actual = relation.to::<Person>().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_just_extract_relation_extra() {
        let relation = test_relation();

        let id = relation.to::<Id>().unwrap();
        let start_node_id = relation.to::<StartNodeId>().unwrap();
        let end_node_id = relation.to::<EndNodeId>().unwrap();
        let typ = relation.to::<Type>().unwrap();
        let keys = relation.to::<Keys>().unwrap();

        assert_eq!(id, Id(1337));
        assert_eq!(start_node_id, StartNodeId(21));
        assert_eq!(end_node_id, EndNodeId(84));
        assert_eq!(typ, Type("Person".to_owned()));
        assert_eq!(keys, Keys(["name".to_owned(), "age".to_owned()].into()));
    }

    #[test]
    fn relation_to_bolt_type() {
        let relation = test_relation();
        let actual = relation.to::<BoltType>().unwrap();
        assert_eq!(actual, BoltType::Relation(relation));
    }

    #[test]
    fn relation_to_bolt_relation() {
        let relation = test_relation();
        let actual = relation.to::<BoltRelation>().unwrap();
        assert_eq!(actual, relation);
    }

    #[test]
    fn relation_to_relation() {
        let relation = test_relation();
        let actual = relation.to::<Relation>().unwrap();

        assert_eq!(actual.id(), relation.id.value);
        assert_eq!(actual.start_node_id(), relation.start_node_id.value);
        assert_eq!(actual.end_node_id(), relation.end_node_id.value);
        assert_eq!(actual.typ(), relation.typ.value);
        assert_eq!(
            actual.keys().tap_mut(|v| v.sort_unstable()),
            relation
                .properties
                .value
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .tap_mut(|v| v.sort_unstable())
        );
    }
}
