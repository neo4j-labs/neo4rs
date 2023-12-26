use super::DeError;
use crate::types::{
    BoltInteger, BoltList, BoltMap, BoltNode, BoltPath, BoltRelation, BoltString, BoltType,
    BoltUnboundedRelation,
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
    fn value(self, key: ElementDataKey) -> Option<ElementDataValue<'de>>;

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
    Nodes,
    Relationships,
    Indices,
});

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
        let properties = self
            .data
            .value(ElementDataKey::Properties)
            .and_then(|v| match v {
                ElementDataValue::Map(map) => Some(&map.value),
                _ => None,
            });
        let additional_fields = fields
            .iter()
            .copied()
            .filter(|f| match properties {
                Some(properties) => !properties.contains_key(*f),
                None => true,
            })
            .map(|f| (f, AdditionalData::Element(self.data)));
        let property_fields = properties
            .into_iter()
            .flatten()
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
                let id = match self.data.value(ElementDataKey::Id) {
                    Some(ElementDataValue::Int(&BoltInteger { value: id })) => id,
                    _ => return Err(DeError::missing_field("id")),
                };
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(I64Deserializer::new(id)),
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(id))),
                    Visitation::Struct(field) => {
                        visitor.visit_map(MapDeserializer::new(iter::once((field, id))))
                    }
                }
            }
            "StartNodeId" => {
                let id = match self.data.value(ElementDataKey::StartNodeId) {
                    Some(ElementDataValue::Int(&BoltInteger { value: id })) => id,
                    _ => return Err(DeError::missing_field("start_node_id")),
                };
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(I64Deserializer::new(id)),
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(id))),
                    Visitation::Struct(field) => {
                        visitor.visit_map(MapDeserializer::new(iter::once((field, id))))
                    }
                }
            }
            "EndNodeId" => {
                let id = match self.data.value(ElementDataKey::EndNodeId) {
                    Some(ElementDataValue::Int(&BoltInteger { value: id })) => id,
                    _ => return Err(DeError::missing_field("end_node_id")),
                };
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(I64Deserializer::new(id)),
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(id))),
                    Visitation::Struct(field) => {
                        visitor.visit_map(MapDeserializer::new(iter::once((field, id))))
                    }
                }
            }
            "Labels" => {
                let labels = match self.data.value(ElementDataKey::Labels) {
                    Some(ElementDataValue::Lst(BoltList { value: labels })) => labels,
                    _ => return Err(DeError::missing_field("labels")),
                };
                match visitation {
                    Visitation::Newtype => {
                        visitor.visit_newtype_struct(SeqDeserializer::new(labels.iter()))
                    }
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(
                        IterDeserializer(labels.iter()),
                    ))),
                    Visitation::Struct(field) => visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(labels.iter()))),
                    )),
                }
            }
            "Type" => {
                let typ = match self.data.value(ElementDataKey::Type) {
                    Some(ElementDataValue::Str(BoltString { value: typ })) => typ,
                    _ => return Err(DeError::missing_field("type")),
                };
                let typ = BorrowedStr(typ);
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(typ.into_deserializer()),
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(typ))),
                    Visitation::Struct(field) => {
                        visitor.visit_map(MapDeserializer::new(iter::once((field, typ))))
                    }
                }
            }
            "Keys" => {
                let properties = match self.data.value(ElementDataKey::Properties) {
                    Some(ElementDataValue::Map(BoltMap { value: properties })) => properties,
                    _ => return Err(DeError::missing_field("properties")),
                };
                let keys = properties.keys();
                match visitation {
                    Visitation::Newtype => visitor.visit_newtype_struct(SeqDeserializer::new(keys)),
                    Visitation::Tuple => {
                        visitor.visit_seq(SeqDeserializer::new(iter::once(IterDeserializer(keys))))
                    }
                    Visitation::Struct(field) => visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(keys))),
                    )),
                }
            }
            "Nodes" => {
                let nodes = match self.data.value(ElementDataKey::Nodes) {
                    Some(ElementDataValue::Lst(BoltList { value: nodes })) => nodes,
                    _ => return Err(DeError::missing_field("nodes")),
                };
                match visitation {
                    Visitation::Newtype => {
                        visitor.visit_newtype_struct(SeqDeserializer::new(nodes.iter()))
                    }
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(
                        IterDeserializer(nodes.iter()),
                    ))),
                    Visitation::Struct(field) => visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(nodes.iter()))),
                    )),
                }
            }
            "Relationships" => {
                let rels = match self.data.value(ElementDataKey::Relationships) {
                    Some(ElementDataValue::Lst(BoltList { value: rels })) => rels,
                    _ => return Err(DeError::missing_field("relationships")),
                };
                match visitation {
                    Visitation::Newtype => {
                        visitor.visit_newtype_struct(SeqDeserializer::new(rels.iter()))
                    }
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(
                        IterDeserializer(rels.iter()),
                    ))),
                    Visitation::Struct(field) => visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(rels.iter()))),
                    )),
                }
            }
            "Indices" => {
                let ids = match self.data.value(ElementDataKey::Indices) {
                    Some(ElementDataValue::Lst(BoltList { value: ids })) => ids,
                    _ => return Err(DeError::missing_field("indices")),
                };
                match visitation {
                    Visitation::Newtype => {
                        visitor.visit_newtype_struct(SeqDeserializer::new(ids.iter()))
                    }
                    Visitation::Tuple => visitor.visit_seq(SeqDeserializer::new(iter::once(
                        IterDeserializer(ids.iter()),
                    ))),
                    Visitation::Struct(field) => visitor.visit_map(MapDeserializer::new(
                        iter::once((field, IterDeserializer(ids.iter()))),
                    )),
                }
            }
            _ => Err(DeError::invalid_type(
                Unexpected::Other(&format!("struct `{}`", name)),
                &concat!(
                    "one of `Id`, `Labels`, `Type`, `StartNodeId`, ",
                    "`EndNodeId`, `Keys`, `Nodes`, `Relationships`, or `Indices`"
                ),
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

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf seq tuple map enum identifier
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::PropertyMissingButRequired)
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

#[derive(Copy, Clone, Debug)]
enum AdditionalData<'de, T> {
    Property(&'de BoltType),
    Element(T),
}

enum AdditionalDataDeserializer<'de, T> {
    Property(<&'de BoltType as IntoDeserializer<'de, DeError>>::Deserializer),
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
    fn value(self, key: ElementDataKey) -> Option<ElementDataValue<'de>> {
        match key {
            ElementDataKey::Id => Some(ElementDataValue::Int(&self.id)),
            ElementDataKey::Labels => Some(ElementDataValue::Lst(&self.labels)),
            ElementDataKey::Properties => Some(ElementDataValue::Map(&self.properties)),
            _ => None,
        }
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
    fn value(self, key: ElementDataKey) -> Option<ElementDataValue<'de>> {
        match key {
            ElementDataKey::Id => Some(ElementDataValue::Int(&self.id)),
            ElementDataKey::StartNodeId => Some(ElementDataValue::Int(&self.start_node_id)),
            ElementDataKey::EndNodeId => Some(ElementDataValue::Int(&self.end_node_id)),
            ElementDataKey::Type => Some(ElementDataValue::Str(&self.typ)),
            ElementDataKey::Properties => Some(ElementDataValue::Map(&self.properties)),
            _ => None,
        }
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
    fn value(self, key: ElementDataKey) -> Option<ElementDataValue<'de>> {
        match key {
            ElementDataKey::Id => Some(ElementDataValue::Int(&self.id)),
            ElementDataKey::Type => Some(ElementDataValue::Str(&self.typ)),
            ElementDataKey::Properties => Some(ElementDataValue::Map(&self.properties)),
            _ => None,
        }
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

impl<'de> ElementData<'de> for &'de BoltPath {
    fn value(self, key: ElementDataKey) -> Option<ElementDataValue<'de>> {
        match key {
            ElementDataKey::Nodes => Some(ElementDataValue::Lst(&self.nodes)),
            ElementDataKey::Relationships => Some(ElementDataValue::Lst(&self.rels)),
            ElementDataKey::Indices => Some(ElementDataValue::Lst(&self.indices)),
            _ => None,
        }
    }

    type Items = [(ElementDataKey, ElementDataValue<'de>); 3];

    fn items(self) -> Self::Items {
        [
            (ElementDataKey::Nodes, ElementDataValue::Lst(&self.nodes)),
            (
                ElementDataKey::Relationships,
                ElementDataValue::Lst(&self.rels),
            ),
            (
                ElementDataKey::Indices,
                ElementDataValue::Lst(&self.indices),
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
    use crate::{EndNodeId, Id, Indices, Keys, Labels, Nodes, Relationships, StartNodeId, Type};

    #[test]
    fn node_impl() {
        let node = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );

        assert_eq!(
            node.value(ElementDataKey::Id),
            Some(ElementDataValue::Int(&BoltInteger::new(42)))
        );
        assert_eq!(node.value(ElementDataKey::StartNodeId), None);
        assert_eq!(node.value(ElementDataKey::EndNodeId), None);
        assert_eq!(
            node.value(ElementDataKey::Labels),
            Some(ElementDataValue::Lst(&BoltList::from(vec![
                BoltType::from("Person")
            ])))
        );
        assert_eq!(node.value(ElementDataKey::Type), None);
        assert_eq!(
            node.value(ElementDataKey::Properties),
            Some(ElementDataValue::Map(
                &[("name".into(), "Alice".into())].into_iter().collect()
            ))
        );
        assert_eq!(node.value(ElementDataKey::Nodes), None);
        assert_eq!(node.value(ElementDataKey::Relationships), None);
        assert_eq!(node.value(ElementDataKey::Indices), None);

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

        assert_eq!(
            rel.value(ElementDataKey::Id),
            Some(ElementDataValue::Int(&BoltInteger::new(42)))
        );
        assert_eq!(
            rel.value(ElementDataKey::StartNodeId),
            Some(ElementDataValue::Int(&BoltInteger::new(1)))
        );
        assert_eq!(
            rel.value(ElementDataKey::EndNodeId),
            Some(ElementDataValue::Int(&BoltInteger::new(2)))
        );
        assert_eq!(rel.value(ElementDataKey::Labels), None);
        assert_eq!(
            rel.value(ElementDataKey::Type),
            Some(ElementDataValue::Str(&BoltString::from("KNOWS")))
        );
        assert_eq!(
            rel.value(ElementDataKey::Properties),
            Some(ElementDataValue::Map(
                &[("since".into(), 2017.into())].into_iter().collect()
            ))
        );
        assert_eq!(rel.value(ElementDataKey::Nodes), None);
        assert_eq!(rel.value(ElementDataKey::Relationships), None);
        assert_eq!(rel.value(ElementDataKey::Indices), None);

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

        assert_eq!(
            unbounded_rel.value(ElementDataKey::Id),
            Some(ElementDataValue::Int(&BoltInteger::new(42)))
        );
        assert_eq!(
            unbounded_rel.value(ElementDataKey::Type),
            Some(ElementDataValue::Str(&BoltString::from("KNOWS")))
        );
        assert_eq!(
            unbounded_rel.value(ElementDataKey::Properties),
            Some(ElementDataValue::Map(
                &[("since".into(), 2017.into())].into_iter().collect()
            ))
        );
        assert_eq!(unbounded_rel.value(ElementDataKey::StartNodeId), None);
        assert_eq!(unbounded_rel.value(ElementDataKey::EndNodeId), None);
        assert_eq!(unbounded_rel.value(ElementDataKey::Labels), None);

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
    fn path_impl() {
        let alice = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );
        let bob = BoltNode::new(
            BoltInteger::new(1337),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Bob".into())].into_iter().collect(),
        );
        let rel = BoltUnboundedRelation::new(
            BoltInteger::new(84),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );
        let path = BoltPath {
            nodes: BoltList::from(vec![
                BoltType::Node(alice.clone()),
                BoltType::Node(bob.clone()),
            ]),
            rels: BoltList::from(vec![BoltType::UnboundedRelation(rel.clone())]),
            indices: BoltList::from(vec![BoltType::from(42), BoltType::from(1337)]),
        };

        assert_eq!(path.value(ElementDataKey::Id), None);
        assert_eq!(path.value(ElementDataKey::StartNodeId), None);
        assert_eq!(path.value(ElementDataKey::EndNodeId), None);
        assert_eq!(path.value(ElementDataKey::Labels), None);
        assert_eq!(path.value(ElementDataKey::Type), None);
        assert_eq!(path.value(ElementDataKey::Properties), None);
        assert_eq!(
            path.value(ElementDataKey::Nodes),
            Some(ElementDataValue::Lst(&BoltList::from(vec![
                BoltType::Node(alice.clone()),
                BoltType::Node(bob.clone())
            ])))
        );
        assert_eq!(
            path.value(ElementDataKey::Relationships),
            Some(ElementDataValue::Lst(&BoltList::from(vec![
                BoltType::UnboundedRelation(rel.clone())
            ])))
        );
        assert_eq!(
            path.value(ElementDataKey::Indices),
            Some(ElementDataValue::Lst(&BoltList::from(vec![
                BoltType::from(42),
                BoltType::from(1337)
            ])))
        );

        let mut items = path.items().into_iter();
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Nodes,
                ElementDataValue::Lst(&BoltList::from(vec![
                    BoltType::Node(alice),
                    BoltType::Node(bob)
                ]))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Relationships,
                ElementDataValue::Lst(&BoltList::from(vec![BoltType::UnboundedRelation(rel)]))
            ))
        );
        assert_eq!(
            items.next(),
            Some((
                ElementDataKey::Indices,
                ElementDataValue::Lst(&BoltList::from(vec![
                    BoltType::from(42),
                    BoltType::from(1337)
                ]))
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
    fn path_deser() {
        let alice = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );
        let bob = BoltNode::new(
            BoltInteger::new(1337),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Bob".into())].into_iter().collect(),
        );
        let rel = BoltUnboundedRelation::new(
            BoltInteger::new(84),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );
        let path = BoltPath {
            nodes: BoltList::from(vec![BoltType::Node(alice), BoltType::Node(bob)]),
            rels: BoltList::from(vec![BoltType::UnboundedRelation(rel)]),
            indices: BoltList::from(vec![BoltType::from(42), BoltType::from(1337)]),
        };

        let nodes = Nodes::<Id>::deserialize(ElementDataDeserializer::new(&path)).unwrap();
        assert_eq!(nodes, Nodes(vec![Id(42), Id(1337)]));

        let rels = Relationships::<Id>::deserialize(ElementDataDeserializer::new(&path)).unwrap();
        assert_eq!(rels, Relationships(vec![Id(84)]));

        let ids = Indices::deserialize(ElementDataDeserializer::new(&path)).unwrap();
        assert_eq!(ids, Indices(vec![42, 1337]));
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

    #[test]
    fn path_deser_map() {
        let alice = BoltNode::new(
            BoltInteger::new(42),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Alice".into())].into_iter().collect(),
        );
        let bob = BoltNode::new(
            BoltInteger::new(1337),
            BoltList::from(vec![BoltType::from("Person")]),
            [("name".into(), "Bob".into())].into_iter().collect(),
        );
        let rel = BoltUnboundedRelation::new(
            BoltInteger::new(84),
            BoltString::from("KNOWS"),
            [("since".into(), 2017.into())].into_iter().collect(),
        );
        let path = BoltPath {
            nodes: BoltList::from(vec![
                BoltType::Node(alice.clone()),
                BoltType::Node(bob.clone()),
            ]),
            rels: BoltList::from(vec![BoltType::UnboundedRelation(rel.clone())]),
            indices: BoltList::from(vec![BoltType::from(42), BoltType::from(1337)]),
        };

        let actual = HashMap::<ElementDataKey, BoltType>::deserialize(MapAccessDeserializer::new(
            ElementMapAccess::new(path.items()),
        ))
        .unwrap();

        assert_eq!(
            actual[&ElementDataKey::Nodes],
            BoltType::from(vec![alice, bob])
        );
        assert_eq!(
            actual[&ElementDataKey::Relationships],
            BoltType::from(vec![rel])
        );
        assert_eq!(
            actual[&ElementDataKey::Indices],
            BoltType::from(vec![42, 1337])
        );
    }
}
