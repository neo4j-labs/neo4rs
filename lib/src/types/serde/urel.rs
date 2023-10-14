use crate::{
    types::{
        serde::{
            builder::{BoltUnboundedRelationBuilder, Id},
            element::{ElementDataDeserializer, ElementDataKey},
            BoltKind,
        },
        BoltString, BoltUnboundedRelation,
    },
    DeError, Type, UnboundedRelation,
};

use std::{fmt, result::Result};

use serde::{
    de::{
        value::MapDeserializer, DeserializeSeed, Deserializer, EnumAccess, Error, IntoDeserializer,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

impl<'de> Deserialize<'de> for UnboundedRelation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BoltUnboundedRelation::deserialize(deserializer).map(UnboundedRelation::new)
    }
}

impl BoltUnboundedRelation {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
    }
}

impl<'de> Deserialize<'de> for BoltUnboundedRelation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const ID: &str = "42.<id>";
        const TYP: &str = "42.<type>";

        struct BoltUnboundedRelationVisitor;

        impl<'de> Visitor<'de> for BoltUnboundedRelationVisitor {
            type Value = BoltUnboundedRelation;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BoltUnboundedRelation")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                let mut builder = BoltUnboundedRelationBuilder::default();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        ID => builder.id(|| map.next_value::<Id>().map(|i| i.0))?,
                        TYP => builder.typ(|| map.next_value::<Type<BoltString>>().map(|t| t.0))?,
                        otherwise => builder
                            .insert(|| Ok((BoltString::from(otherwise), map.next_value()?)))?,
                    }
                }

                let node = builder.build()?;
                Ok(node)
            }
        }

        deserializer.deserialize_struct(
            "BoltUnboundedRelation",
            &[ID, TYP],
            BoltUnboundedRelationVisitor,
        )
    }
}

pub struct BoltUnboundedRelationVisitor;

impl<'de> Visitor<'de> for BoltUnboundedRelationVisitor {
    type Value = BoltUnboundedRelation;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct BoltUnboundedRelation")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut builder = BoltUnboundedRelationBuilder::default();

        while let Some(key) = map.next_key::<ElementDataKey>()? {
            match key {
                ElementDataKey::Id => builder.id(|| map.next_value())?,
                ElementDataKey::Type => builder.typ(|| map.next_value())?,
                ElementDataKey::Properties => builder.properties(|| map.next_value())?,
                otherwise => {
                    return Err(Error::unknown_field(
                        otherwise.name(),
                        &["Id", "Type", "Properties"],
                    ))
                }
            }
        }

        let node = builder.build()?;
        Ok(node)
    }
}

pub struct BoltUnboundedRelationDeserializer<'de>(&'de BoltUnboundedRelation);

impl<'de> BoltUnboundedRelationDeserializer<'de> {
    fn new(node: &'de BoltUnboundedRelation) -> Self {
        Self(node)
    }
}

impl<'de> Deserializer<'de> for BoltUnboundedRelationDeserializer<'de> {
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

impl<'de> EnumAccess<'de> for BoltUnboundedRelationDeserializer<'de> {
    type Error = DeError;

    type Variant = ElementDataDeserializer<'de, &'de BoltUnboundedRelation>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let kind = BoltKind::UnboundedRelation;
        let val = seed.deserialize(kind.into_deserializer())?;
        Ok((val, ElementDataDeserializer::new(self.0)))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltUnboundedRelation {
    type Deserializer = BoltUnboundedRelationDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltUnboundedRelationDeserializer::new(self)
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

    fn test_unbounded_relation() -> BoltUnboundedRelation {
        let id = BoltInteger::new(1337);
        let typ = "Person".into();
        let properties = vec![
            ("name".into(), "Alice".into()),
            ("age".into(), 42_u16.into()),
        ]
        .into_iter()
        .collect();

        BoltUnboundedRelation {
            id,
            properties,
            typ,
        }
    }

    #[test]
    fn unbounded_relation() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        test_extract_unbounded_relation(Person {
            name: "Alice".into(),
            age: 42,
        });
    }

    #[test]
    fn extract_unbounded_relation_with_unit_types() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<T> {
            name: String,
            age: u8,
            _t: PhantomData<T>,
            _u: (),
        }

        test_extract_unbounded_relation(Person {
            name: "Alice".to_owned(),
            age: 42,
            _t: PhantomData::<i32>,
            _u: (),
        });
    }

    #[test]
    fn extract_unbounded_relation_id() {
        test_extract_unbounded_relation_extra(Id(1337));
    }

    #[test]
    fn extract_unbounded_relation_id_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Id(i16);

        test_extract_unbounded_relation_extra(Id(1337));
    }

    #[test]
    fn extract_unbounded_relation_id_with_custom_struct() {
        #[derive(Debug, Deserialize)]
        struct Id {
            id: AtomicU32,
        }

        impl PartialEq for Id {
            fn eq(&self, other: &Self) -> bool {
                self.id.load(Ordering::SeqCst) == other.id.load(Ordering::SeqCst)
            }
        }

        test_extract_unbounded_relation_extra(Id { id: 1337.into() });
    }

    #[test]
    fn extract_unbounded_relation_type() {
        test_extract_unbounded_relation_extra(Type("Person".to_owned()));
    }

    #[test]
    fn extract_unbounded_relation_type_custom_inner() {
        test_extract_unbounded_relation_extra(Type::<Box<str>>("Person".into()));
    }

    #[test]
    fn extract_unbounded_relation_type_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Type(String);

        test_extract_unbounded_relation_extra(Type("Person".to_owned()));
    }

    #[test]
    fn extract_unbounded_relation_type_with_custom_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Type {
            types: String,
        }

        test_extract_unbounded_relation_extra(Type {
            types: "Person".to_owned(),
        });
    }

    #[test]
    fn extract_unbounded_relation_type_borrowed() {
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

        let unbounded_relation = test_unbounded_relation();

        let actual = unbounded_relation.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn extract_unbounded_relation_property_keys() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            keys: Keys,
        }

        let expected = Person {
            keys: Keys(["name".to_owned(), "age".to_owned()].into()),
        };

        test_extract_unbounded_relation(expected);
    }

    #[test]
    fn extract_unbounded_relation_property_keys_custom_vec() {
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

        test_extract_unbounded_relation(expected);
    }

    #[test]
    fn extract_unbounded_relation_property_keys_custom_struct() {
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

        test_extract_unbounded_relation(expected);
    }

    #[test]
    fn extract_unbounded_relation_property_keys_borrowed() {
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

        let unbounded_relation = test_unbounded_relation();
        let actual = unbounded_relation.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    fn test_extract_unbounded_relation_extra<T: Debug + PartialEq + for<'a> Deserialize<'a>>(
        expected: T,
    ) {
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

        test_extract_unbounded_relation(expected);
    }

    fn test_extract_unbounded_relation<Person: Debug + PartialEq + for<'a> Deserialize<'a>>(
        expected: Person,
    ) {
        let unbounded_relation = test_unbounded_relation();
        let actual = unbounded_relation.to::<Person>().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_just_extract_unbounded_relation_extra() {
        let unbounded_relation = test_unbounded_relation();

        let id = unbounded_relation.to::<Id>().unwrap();
        let typ = unbounded_relation.to::<Type>().unwrap();
        let keys = unbounded_relation.to::<Keys>().unwrap();

        assert_eq!(id, Id(1337));
        assert_eq!(typ, Type("Person".to_owned()));
        assert_eq!(keys, Keys(["name".to_owned(), "age".to_owned()].into()));
    }

    #[test]
    fn unbounded_relation_to_bolt_type() {
        let unbounded_relation = test_unbounded_relation();
        let actual = unbounded_relation.to::<BoltType>().unwrap();
        assert_eq!(actual, BoltType::UnboundedRelation(unbounded_relation));
    }

    #[test]
    fn unbounded_relation_to_bolt_unbounded_relation() {
        let unbounded_relation = test_unbounded_relation();
        let actual = unbounded_relation.to::<BoltUnboundedRelation>().unwrap();
        assert_eq!(actual, unbounded_relation);
    }

    #[test]
    fn unbounded_relation_to_unbounded_relation() {
        let unbounded_relation = test_unbounded_relation();
        let actual = unbounded_relation.to::<UnboundedRelation>().unwrap();

        assert_eq!(actual.id(), unbounded_relation.id.value);
        assert_eq!(actual.typ(), unbounded_relation.typ.value);
        assert_eq!(
            actual.keys().tap_mut(|v| v.sort_unstable()),
            unbounded_relation
                .properties
                .value
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .tap_mut(|v| v.sort_unstable())
        );
    }
}
