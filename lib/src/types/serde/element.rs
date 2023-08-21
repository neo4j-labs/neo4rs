use super::DeError;
use crate::types::{
    serde::deser::BoltTypeDeserializer, BoltInteger, BoltList, BoltMap, BoltNode, BoltRelation,
    BoltString, BoltType, BoltUnboundedRelation,
};

use std::{iter, marker::PhantomData, result::Result};

use delegate::delegate;
use serde::{
    de::{
        value::{BorrowedStrDeserializer, I64Deserializer, MapDeserializer, SeqDeserializer},
        DeserializeSeed, Deserializer, Error, IntoDeserializer, MapAccess, Unexpected,
        VariantAccess, Visitor,
    },
    forward_to_deserialize_any,
};

pub trait ElementData<'de> {
    fn id(self) -> &'de BoltInteger;

    fn start_node_id(self) -> Option<&'de BoltInteger>;

    fn end_node_id(self) -> Option<&'de BoltInteger>;

    type Labels: Iterator<Item = &'de BoltType>;

    fn labels(self) -> Option<Self::Labels>;

    fn typ(self) -> Option<&'de BoltString>;

    fn properties(self) -> &'de BoltMap;

    type Items: IntoIterator<Item = (ElementDataKey, ElementDataValue<'de>)>;

    fn items(self) -> Self::Items;
}

crate::cenum!(ElementDataKey {
    Id,
    StartNodeId,
    EndNodeId,
    Type,
    Labels,
    Properties,
} element_data_key_tests);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ElementDataValue<'de> {
    Int(&'de BoltInteger),
    Str(&'de BoltString),
    Lst(&'de BoltList),
    Map(&'de BoltMap),
}

pub struct ElementDataDeserializer<'de, T> {
    data: T,
    _lifetime: PhantomData<&'de ()>,
}

impl<'de, T: ElementData<'de>> ElementDataDeserializer<'de, T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            _lifetime: PhantomData,
        }
    }

    pub fn deserialize_outer_struct<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeError>
    where
        V: Visitor<'de>,
        T: Copy,
    {
        let properties = &self.data.properties().value;
        let additional_fields = fields
            .iter()
            .copied()
            .filter(|f| !properties.contains_key(*f))
            .map(|f| (f, AdditionalData::Element(self.data)));
        let property_fields = properties
            .iter()
            .map(|(k, v)| (k.value.as_str(), AdditionalData::Property(v)));
        let node_fields = property_fields
            .chain(additional_fields)
            .map(|(k, v)| (BorrowedStr(k), v));

        visitor.visit_map(MapDeserializer::new(node_fields))
    }

    fn deserialize_any_struct<V>(
        self,
        name: &str,
        visitor: V,
        visitation: Visitation,
    ) -> Result<V::Value, DeError>
    where
        V: Visitor<'de>,
    {
        struct IterDeserializer<I>(I);

        impl<'de, I, T> IntoDeserializer<'de, DeError> for IterDeserializer<I>
        where
            T: 'de,
            I: Iterator<Item = &'de T>,
            &'de T: IntoDeserializer<'de, DeError>,
        {
            type Deserializer = SeqDeserializer<I, DeError>;

            fn into_deserializer(self) -> Self::Deserializer {
                SeqDeserializer::new(self.0)
            }
        }

        match name {
            "Id" => {
                let id = self.data.id().value;
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(I64Deserializer::new(id)),
                    Visitation::Tuple => {
                        Ok(visitor.visit_seq(SeqDeserializer::new(iter::once(id))))?
                    }
                    Visitation::Struct(field) => {
                        Ok(visitor.visit_map(MapDeserializer::new(iter::once((field, id))))?)
                    }
                }
            }
            "StartNodeId" => {
                let id = self
                    .data
                    .start_node_id()
                    .ok_or_else(|| DeError::missing_field("start_node_id"))?
                    .value;
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(I64Deserializer::new(id)),
                    Visitation::Tuple => {
                        Ok(visitor.visit_seq(SeqDeserializer::new(iter::once(id))))?
                    }
                    Visitation::Struct(field) => {
                        Ok(visitor.visit_map(MapDeserializer::new(iter::once((field, id))))?)
                    }
                }
            }
            "EndNodeId" => {
                let id = self
                    .data
                    .end_node_id()
                    .ok_or_else(|| DeError::missing_field("end_node_id"))?
                    .value;
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(I64Deserializer::new(id)),
                    Visitation::Tuple => {
                        Ok(visitor.visit_seq(SeqDeserializer::new(iter::once(id))))?
                    }
                    Visitation::Struct(field) => {
                        Ok(visitor.visit_map(MapDeserializer::new(iter::once((field, id))))?)
                    }
                }
            }
            "Labels" => {
                let labels = self
                    .data
                    .labels()
                    .ok_or_else(|| DeError::missing_field("labels"))?;
                match visitation {
                    Visitation::Newtype => {
                        visitor.visit_newtype_struct(SeqDeserializer::new(labels))
                    }
                    Visitation::Tuple => Ok(visitor
                        .visit_seq(SeqDeserializer::new(iter::once(IterDeserializer(labels)))))?,
                    Visitation::Struct(field) => Ok(visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(labels))),
                    ))?),
                }
            }
            "Type" => {
                let typ = self
                    .data
                    .typ()
                    .ok_or_else(|| DeError::missing_field("type"))?
                    .value
                    .as_str();
                match visitation {
                    Visitation::Newtype => {
                        visitor.visit_newtype_struct(BorrowedStrDeserializer::new(typ))
                    }
                    Visitation::Tuple => {
                        Ok(visitor.visit_seq(SeqDeserializer::new(iter::once(typ))))?
                    }
                    Visitation::Struct(field) => {
                        Ok(visitor.visit_map(MapDeserializer::new(iter::once((field, typ))))?)
                    }
                }
            }
            "Keys" => {
                let keys = self.data.properties().value.keys();
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(SeqDeserializer::new(keys)),
                    Visitation::Tuple => {
                        Ok(visitor
                            .visit_seq(SeqDeserializer::new(iter::once(IterDeserializer(keys)))))?
                    }
                    Visitation::Struct(field) => Ok(visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(keys))),
                    ))?),
                }
            }
            _ => Err(DeError::invalid_type(
                Unexpected::Other(&format!("struct `{}`", name)),
                &"one of `Id`, `Labels`, `Type`, `StartNodeId`, `EndNodeId`, or `Keys`",
            )),
        }
    }
}

impl<'de, T: ElementData<'de>> Deserializer<'de> for ElementDataDeserializer<'de, T> {
    type Error = DeError;

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any_struct(name, visitor, Visitation::Newtype)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if len == 1 {
            self.deserialize_any_struct(name, visitor, Visitation::Tuple)
        } else {
            Err(DeError::invalid_length(
                len,
                &format!("tuple struct {} with 1 element", name).as_str(),
            ))
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match fields {
            [field] => self.deserialize_any_struct(name, visitor, Visitation::Struct(field)),
            _ => Err(DeError::invalid_length(fields.len(), &"1")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option seq tuple map enum identifier
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::custom(
            "deserializing additional data requires a struct",
        ))
    }
}

enum Visitation {
    Newtype,
    Tuple,
    Struct(&'static str),
}

impl<'de, T: ElementData<'de>> VariantAccess<'de> for ElementDataDeserializer<'de, T> {
    type Error = DeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(DeError::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn newtype_variant_seed<S>(self, _seed: S) -> Result<S::Value, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        Err(DeError::invalid_type(
            Unexpected::TupleVariant,
            &"tuple variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(ElementMapAccess::new(self.data.items()))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::invalid_type(
            Unexpected::StructVariant,
            &"tuple variant",
        ))
    }
}

pub struct ElementMapAccess<'de, I> {
    elements: I,
    value: Option<ElementDataValue<'de>>,
}

impl<'de, I> ElementMapAccess<'de, I>
where
    I: Iterator<Item = (ElementDataKey, ElementDataValue<'de>)>,
{
    pub fn new<II>(elements: II) -> Self
    where
        II: IntoIterator<IntoIter = I>,
    {
        Self {
            elements: elements.into_iter(),
            value: None,
        }
    }
}

impl<'de, I> MapAccess<'de> for ElementMapAccess<'de, I>
where
    I: Iterator<Item = (ElementDataKey, ElementDataValue<'de>)>,
{
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.elements
            .next()
            .map(|(key, value)| {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer())
            })
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .expect("next_value_seed called before next_key_seed");
        match value {
            ElementDataValue::Int(value) => seed.deserialize(I64Deserializer::new(value.value)),
            ElementDataValue::Str(value) => {
                seed.deserialize(NoEnumBorrowedStrDeserializer(&value.value))
            }
            ElementDataValue::Lst(value) => {
                seed.deserialize(SeqDeserializer::new(value.value.iter()))
            }
            ElementDataValue::Map(value) => {
                seed.deserialize(MapDeserializer::new(value.value.iter()))
            }
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.elements.size_hint().1
    }
}

// Like serdes BorrowedStrDeserializer, but without the different behavior
// for deserialize_enum. The serde type would return the str as a variant
// instead of the value of the variant.
struct NoEnumBorrowedStrDeserializer<'de>(&'de str);

impl<'de> Deserializer<'de> for NoEnumBorrowedStrDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.0)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any enum
    }
}

struct BorrowedStr<'de>(&'de str);

impl<'de> IntoDeserializer<'de, DeError> for BorrowedStr<'de> {
    type Deserializer = BorrowedStrDeserializer<'de, DeError>;

    fn into_deserializer(self) -> Self::Deserializer {
        BorrowedStrDeserializer::new(self.0)
    }
}

enum AdditionalData<'de, T> {
    Property(&'de BoltType),
    Element(T),
}

enum AdditionalDataDeserializer<'de, T> {
    Property(BoltTypeDeserializer<'de>),
    Element(ElementDataDeserializer<'de, T>),
}

impl<'de, T: ElementData<'de>> Deserializer<'de> for AdditionalDataDeserializer<'de, T> {
    type Error = DeError;

    delegate! {
        to match self { Self::Property(v) => v, Self::Element(v) => v } {
            fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_unit_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_tuple_struct<V: Visitor<'de>>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_struct<V: Visitor<'de>>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_enum<V: Visitor<'de>>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
            fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;
        }
    }
}

impl<'de, T: ElementData<'de>> IntoDeserializer<'de, DeError> for AdditionalData<'de, T> {
    type Deserializer = AdditionalDataDeserializer<'de, T>;

    fn into_deserializer(self) -> Self::Deserializer {
        match self {
            AdditionalData::Property(v) => {
                AdditionalDataDeserializer::Property(v.into_deserializer())
            }
            AdditionalData::Element(v) => {
                AdditionalDataDeserializer::Element(ElementDataDeserializer::new(v))
            }
        }
    }
}

impl<'de> ElementData<'de> for &'de BoltNode {
    fn id(self) -> &'de BoltInteger {
        &self.id
    }

    fn start_node_id(self) -> Option<&'de BoltInteger> {
        None
    }

    fn end_node_id(self) -> Option<&'de BoltInteger> {
        None
    }

    type Labels = std::slice::Iter<'de, BoltType>;

    fn labels(self) -> Option<Self::Labels> {
        Some(self.labels.value.iter())
    }

    fn typ(self) -> Option<&'de BoltString> {
        None
    }

    fn properties(self) -> &'de BoltMap {
        &self.properties
    }

    type Items = [(ElementDataKey, ElementDataValue<'de>); 3];

    fn items(self) -> Self::Items {
        [
            (ElementDataKey::Id, ElementDataValue::Int(&self.id)),
            (ElementDataKey::Labels, ElementDataValue::Lst(&self.labels)),
            (
                ElementDataKey::Properties,
                ElementDataValue::Map(&self.properties),
            ),
        ]
    }
}

impl<'de> ElementData<'de> for &'de BoltRelation {
    fn id(self) -> &'de BoltInteger {
        &self.id
    }

    fn start_node_id(self) -> Option<&'de BoltInteger> {
        Some(&self.start_node_id)
    }

    fn end_node_id(self) -> Option<&'de BoltInteger> {
        Some(&self.end_node_id)
    }

    type Labels = std::iter::Empty<&'de BoltType>;

    fn labels(self) -> Option<Self::Labels> {
        None
    }

    fn typ(self) -> Option<&'de BoltString> {
        Some(&self.typ)
    }

    fn properties(self) -> &'de BoltMap {
        &self.properties
    }

    type Items = [(ElementDataKey, ElementDataValue<'de>); 5];

    fn items(self) -> Self::Items {
        [
            (ElementDataKey::Id, ElementDataValue::Int(&self.id)),
            (
                ElementDataKey::StartNodeId,
                ElementDataValue::Int(&self.start_node_id),
            ),
            (
                ElementDataKey::EndNodeId,
                ElementDataValue::Int(&self.end_node_id),
            ),
            (ElementDataKey::Type, ElementDataValue::Str(&self.typ)),
            (
                ElementDataKey::Properties,
                ElementDataValue::Map(&self.properties),
            ),
        ]
    }
}

impl<'de> ElementData<'de> for &'de BoltUnboundedRelation {
    fn id(self) -> &'de BoltInteger {
        &self.id
    }

    fn start_node_id(self) -> Option<&'de BoltInteger> {
        None
    }

    fn end_node_id(self) -> Option<&'de BoltInteger> {
        None
    }

    type Labels = std::iter::Empty<&'de BoltType>;

    fn labels(self) -> Option<Self::Labels> {
        None
    }

    fn typ(self) -> Option<&'de BoltString> {
        Some(&self.typ)
    }

    fn properties(self) -> &'de BoltMap {
        &self.properties
    }

    type Items = [(ElementDataKey, ElementDataValue<'de>); 3];

    fn items(self) -> Self::Items {
        [
            (ElementDataKey::Id, ElementDataValue::Int(&self.id)),
            (ElementDataKey::Type, ElementDataValue::Str(&self.typ)),
            (
                ElementDataKey::Properties,
                ElementDataValue::Map(&self.properties),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{de::value::MapAccessDeserializer, Deserialize};
    use std::collections::HashMap;

    use crate::types::{BoltInteger, BoltList, BoltString};
    use crate::{EndNodeId, Id, Keys, Labels, StartNodeId, Type};

    #[test]
    fn node_impl() {
        let node = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );

        assert_eq!(node.id(), &BoltInteger::new(42));
        assert_eq!(node.start_node_id(), None);
        assert_eq!(node.end_node_id(), None);
        assert_eq!(
            node.labels().unwrap().collect::<Vec<_>>(),
            &[&BoltType::from("Person")]
        );
        assert_eq!(node.typ(), None);
        assert_eq!(
            node.properties(),
            &[("name".into(), "Alice".into())].into_iter().collect()
        );

        let mut items = node.items().into_iter();
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Id,
                ElementDataValue::Int(&BoltInteger::new(42))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Labels,
                ElementDataValue::Lst(&BoltList::from(vec![BoltType::from("Person")]))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Properties,
                ElementDataValue::Map(&[("name".into(), "Alice".into())].into_iter().collect())
            ))
        );
        assert_eq!(items.next(), None);
    }

    #[test]
    fn rel_impl() {
        let rel = BoltRelation {
            id: BoltInteger::new(42),
            start_node_id: BoltInteger::new(1),
            end_node_id: BoltInteger::new(2),
            typ: BoltString::from("KNOWS"),
            properties: [("since".into(), 2017.into())].into_iter().collect(),
        };

        assert_eq!(rel.id(), &BoltInteger::new(42));
        assert_eq!(rel.start_node_id(), Some(&BoltInteger::new(1)));
        assert_eq!(rel.end_node_id(), Some(&BoltInteger::new(2)));
        assert!(rel.labels().is_none());
        assert_eq!(rel.typ(), Some(&BoltString::from("KNOWS")));
        assert_eq!(
            rel.properties(),
            &[("since".into(), 2017.into())].into_iter().collect()
        );

        let mut items = rel.items().into_iter();
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Id,
                ElementDataValue::Int(&BoltInteger::new(42))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::StartNodeId,
                ElementDataValue::Int(&BoltInteger::new(1))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::EndNodeId,
                ElementDataValue::Int(&BoltInteger::new(2))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Type,
                ElementDataValue::Str(&BoltString::from("KNOWS"))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Properties,
                ElementDataValue::Map(&[("since".into(), 2017.into())].into_iter().collect())
            ))
        );
        assert_eq!(items.next(), None);
    }

    #[test]
    fn unbounded_rel_impl() {
        let unbounded_rel = BoltUnboundedRelation::new(
            BoltInteger::new(42),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );

        assert_eq!(unbounded_rel.id(), &BoltInteger::new(42));
        assert_eq!(unbounded_rel.start_node_id(), None);
        assert_eq!(unbounded_rel.end_node_id(), None);
        assert!(unbounded_rel.labels().is_none());
        assert_eq!(unbounded_rel.typ(), Some(&BoltString::from("KNOWS")));
        assert_eq!(
            unbounded_rel.properties(),
            &[("since".into(), 2017.into())].into_iter().collect()
        );

        let mut items = unbounded_rel.items().into_iter();
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Id,
                ElementDataValue::Int(&BoltInteger::new(42))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Type,
                ElementDataValue::Str(&BoltString::from("KNOWS"))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Properties,
                ElementDataValue::Map(&[("since".into(), 2017.into())].into_iter().collect())
            ))
        );
        assert_eq!(items.next(), None);
    }

    #[test]
    fn node_deser() {
        let node = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );

        let id = Id::deserialize(ElementDataDeserializer::new(&node)).unwrap();
        assert_eq!(id, Id(42));

        let labels = Labels::deserialize(ElementDataDeserializer::new(&node)).unwrap();
        assert_eq!(labels, Labels(["Person"]));

        let keys = Keys::deserialize(ElementDataDeserializer::new(&node)).unwrap();
        assert_eq!(keys, Keys(["name"]));
    }

    #[test]
    fn rel_deser() {
        let rel = BoltRelation {
            id: BoltInteger::new(42),
            start_node_id: BoltInteger::new(1),
            end_node_id: BoltInteger::new(2),
            typ: BoltString::from("KNOWS"),
            properties: [("since".into(), 2017.into())].into_iter().collect(),
        };

        let id = Id::deserialize(ElementDataDeserializer::new(&rel)).unwrap();
        assert_eq!(id, Id(42));

        let start_node_id = StartNodeId::deserialize(ElementDataDeserializer::new(&rel)).unwrap();
        assert_eq!(start_node_id, StartNodeId(1));

        let end_node_id = EndNodeId::deserialize(ElementDataDeserializer::new(&rel)).unwrap();
        assert_eq!(end_node_id, EndNodeId(2));

        let typ = Type::deserialize(ElementDataDeserializer::new(&rel)).unwrap();
        assert_eq!(typ, Type("KNOWS"));

        let keys = Keys::deserialize(ElementDataDeserializer::new(&rel)).unwrap();
        assert_eq!(keys, Keys(["since"]));
    }

    #[test]
    fn unbounded_deser() {
        let unbounded_rel = BoltUnboundedRelation::new(
            BoltInteger::new(42),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );

        let id = Id::deserialize(ElementDataDeserializer::new(&unbounded_rel)).unwrap();
        assert_eq!(id, Id(42));

        let typ = Type::deserialize(ElementDataDeserializer::new(&unbounded_rel)).unwrap();
        assert_eq!(typ, Type("KNOWS"));

        let keys = Keys::deserialize(ElementDataDeserializer::new(&unbounded_rel)).unwrap();
        assert_eq!(keys, Keys(["since"]));
    }

    #[test]
    fn node_deser_map() {
        let node = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );

        let person = HashMap::<ElementDataKey, BoltType>::deserialize(MapAccessDeserializer::new(
            ElementMapAccess::new(node.items()),
        ))
        .unwrap();

        assert_eq!(person[&ElementDataKey::Id], BoltType::from(42_u32));
        assert_eq!(
            person[&ElementDataKey::Labels],
            BoltType::from(vec!["Person"])
        );
        assert_eq!(
            person[&ElementDataKey::Properties],
            BoltType::Map([("name".into(), "Alice".into())].into_iter().collect())
        );
    }

    #[test]
    fn rel_deser_map() {
        let rel = BoltRelation {
            id: BoltInteger::new(42),
            start_node_id: BoltInteger::new(1),
            end_node_id: BoltInteger::new(2),
            typ: BoltString::from("KNOWS"),
            properties: [("since".into(), 2017.into())].into_iter().collect(),
        };

        let knows = HashMap::<ElementDataKey, BoltType>::deserialize(MapAccessDeserializer::new(
            ElementMapAccess::new(rel.items()),
        ))
        .unwrap();

        assert_eq!(knows[&ElementDataKey::Id], BoltType::from(42_u32));
        assert_eq!(knows[&ElementDataKey::StartNodeId], BoltType::from(1_u32));
        assert_eq!(knows[&ElementDataKey::EndNodeId], BoltType::from(2_u32));
        assert_eq!(knows[&ElementDataKey::Type], BoltType::from("KNOWS"));
        assert_eq!(
            knows[&ElementDataKey::Properties],
            BoltType::Map([("since".into(), 2017.into())].into_iter().collect())
        );
    }

    #[test]
    fn unbounded_deser_map() {
        let unbounded_rel = BoltUnboundedRelation::new(
            BoltInteger::new(42),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );

        let knows = HashMap::<ElementDataKey, BoltType>::deserialize(MapAccessDeserializer::new(
            ElementMapAccess::new(unbounded_rel.items()),
        ))
        .unwrap();

        assert_eq!(knows[&ElementDataKey::Id], BoltType::from(42_u32));
        assert_eq!(knows[&ElementDataKey::Type], BoltType::from("KNOWS"));
        assert_eq!(
            knows[&ElementDataKey::Properties],
            BoltType::Map([("since".into(), 2017.into())].into_iter().collect())
        );
    }
}
