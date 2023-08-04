use crate::types::{
    BoltInteger, BoltMap, BoltNode, BoltRelation, BoltString, BoltType, BoltUnboundedRelation,
};

use serde::{
    de::{
        value::{BorrowedStrDeserializer, I64Deserializer, MapDeserializer, SeqDeserializer},
        Deserializer, Error, IntoDeserializer, Unexpected, Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};
use std::{collections::HashSet, iter, marker::PhantomData};

/// Newtype to extract the node id or relationship id during deserialization.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct Id(pub u64);

/// Newtype to extract the start node id of a relationship during deserialization.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct StartNodeId(pub u64);

/// Newtype to extract the end node id of a relationship during deserialization.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct EndNodeId(pub u64);

/// Newtype to extract the node labels during deserialization.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct Labels<Coll = Vec<String>>(pub Coll);

/// Newtype to extract the relationship type during deserialization.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct Type<T = String>(pub T);

/// Newtype to extract the node property keys during deserialization.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Keys<Coll = HashSet<String>>(pub Coll);

impl BoltMap {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(MapDeserializer::new(self.value.iter()))
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

impl BoltRelation {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
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

#[cfg(test)]
impl BoltType {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeError {
    #[error("{0}")]
    Error(String),

    #[error("Could not convert the integer `{1}` to the target type {2}")]
    IntegerOutOfBounds(#[source] std::num::TryFromIntError, i64, &'static str),
}

pub struct BoltTypeDeserializer<'de> {
    value: BoltRef<'de>,
}

impl<'de> Deserializer<'de> for BoltTypeDeserializer<'de> {
    type Error = DeError;

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltRef::Type(BoltType::List(v)) => {
                visitor.visit_seq(SeqDeserializer::new(v.value.iter()))
            }
            BoltRef::Type(BoltType::Bytes(v)) => {
                visitor.visit_seq(SeqDeserializer::new(v.value.iter().copied()))
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltRef::Type(BoltType::Map(v)) => {
                visitor.visit_map(MapDeserializer::new(v.value.iter()))
            }
            BoltRef::Type(BoltType::Node(v)) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltRef::Node(v) => visitor.visit_map(MapDeserializer::new(v.properties.value.iter())),
            BoltRef::Type(BoltType::Relation(v)) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltRef::Rel(v) => visitor.visit_map(MapDeserializer::new(v.properties.value.iter())),
            BoltRef::Type(BoltType::UnboundedRelation(v)) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltRef::URel(v) => visitor.visit_map(MapDeserializer::new(v.properties.value.iter())),
            _ => self.unexpected(visitor),
        }
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
        fn struct_with_additional<'de, T, V>(
            fields: &'static [&'static str],
            element: T,
            visitor: V,
        ) -> Result<V::Value, DeError>
        where
            T: Copy + AdditionalData<'de>,
            V: Visitor<'de>,
        {
            let properties = &element.properties().value;
            let additional_fields = fields
                .iter()
                .copied()
                .filter(|f| !properties.contains_key(*f))
                .map(|f| (f, ElementData::Additional(element)));
            let property_fields = properties
                .iter()
                .map(|(k, v)| (k.value.as_str(), ElementData::Property(v)));
            let node_fields = property_fields.chain(additional_fields);
            visitor.visit_map(MapDeserializer::new(node_fields))
        }

        match self.value {
            BoltRef::Type(BoltType::Map(v)) => {
                visitor.visit_map(MapDeserializer::new(v.value.iter()))
            }
            BoltRef::Type(BoltType::Node(v)) => struct_with_additional(fields, v, visitor),
            BoltRef::Node(v) => struct_with_additional(fields, v, visitor),
            BoltRef::Type(BoltType::Relation(v)) => struct_with_additional(fields, v, visitor),
            BoltRef::Rel(v) => struct_with_additional(fields, v, visitor),
            BoltRef::Type(BoltType::UnboundedRelation(v)) => {
                struct_with_additional(fields, v, visitor)
            }
            BoltRef::URel(v) => struct_with_additional(fields, v, visitor),
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltRef::Type(BoltType::Node(v)) => {
                AdditionalDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::Node(v) => {
                AdditionalDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::Type(BoltType::Relation(v)) => {
                AdditionalDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::Rel(v) => {
                AdditionalDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::Type(BoltType::UnboundedRelation(v)) => {
                AdditionalDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::URel(v) => {
                AdditionalDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltRef::Type(BoltType::List(v)) if v.len() == len => {
                visitor.visit_seq(SeqDeserializer::new(v.value.iter()))
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::String(v)) = self.value {
            visitor.visit_borrowed_str(&v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::String(v)) = self.value {
            visitor.visit_string(v.value.clone())
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::Bytes(v)) = self.value {
            visitor.visit_borrowed_bytes(&v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::Bytes(v)) = self.value {
            visitor.visit_byte_buf(v.value.to_vec())
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::Boolean(v)) = self.value {
            visitor.visit_bool(v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_float(visitor)?;
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (v, visitor) = self.read_float(visitor)?;
        visitor.visit_f64(v)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::Null(_)) = self.value {
            visitor.visit_unit()
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let BoltRef::Type(BoltType::Null(_)) = self.value {
            visitor.visit_unit()
        } else {
            self.unexpected(visitor)
        }
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
        self.unexpected(visitor)
    }

    forward_to_deserialize_any! {
        char option enum identifier
    }
}

impl<'de> BoltTypeDeserializer<'de> {
    fn new(value: BoltRef<'de>) -> Self {
        Self { value }
    }

    fn read_integer<T, E, V>(self, visitor: V) -> Result<(T, V), DeError>
    where
        V: Visitor<'de>,
        i64: TryInto<T, Error = E>,
        E: Into<std::num::TryFromIntError>,
    {
        if let BoltRef::Type(BoltType::Integer(v)) = self.value {
            match v.value.try_into() {
                Ok(v) => Ok((v, visitor)),
                Err(e) => Err(DeError::IntegerOutOfBounds(
                    e.into(),
                    v.value,
                    std::any::type_name::<T>(),
                )),
            }
        } else {
            self.unexpected(visitor)
        }
    }

    fn read_float<T, V>(self, visitor: V) -> Result<(T, V), DeError>
    where
        V: Visitor<'de>,
        T: FromFloat,
    {
        if let BoltRef::Type(BoltType::Float(v)) = self.value {
            Ok((T::from_float(v.value), visitor))
        } else {
            self.unexpected(visitor)
        }
    }

    fn unexpected<V, T>(self, visitor: V) -> Result<T, DeError>
    where
        V: Visitor<'de>,
    {
        let typ = match self.value {
            BoltRef::Type(BoltType::String(v)) => Unexpected::Str(&v.value),
            BoltRef::Type(BoltType::Boolean(v)) => Unexpected::Bool(v.value),
            BoltRef::Type(BoltType::Map(_)) => Unexpected::Map,
            BoltRef::Type(BoltType::Null(_)) => Unexpected::Unit,
            BoltRef::Type(BoltType::Integer(v)) => Unexpected::Signed(v.value),
            BoltRef::Type(BoltType::Float(v)) => Unexpected::Float(v.value),
            BoltRef::Type(BoltType::List(_)) => Unexpected::Seq,
            BoltRef::Type(BoltType::Node(_)) => Unexpected::Map,
            BoltRef::Node(_) => Unexpected::Map,
            BoltRef::Type(BoltType::Relation(_)) => Unexpected::Map,
            BoltRef::Rel(_) => Unexpected::Map,
            BoltRef::Type(BoltType::UnboundedRelation(_)) => Unexpected::Map,
            BoltRef::URel(_) => Unexpected::Map,
            BoltRef::Type(BoltType::Point2D(_)) => Unexpected::Other("Point2D"),
            BoltRef::Type(BoltType::Point3D(_)) => Unexpected::Other("Point3D"),
            BoltRef::Type(BoltType::Bytes(v)) => Unexpected::Bytes(&v.value),
            BoltRef::Type(BoltType::Path(_)) => Unexpected::Other("Path"),
            BoltRef::Type(BoltType::Duration(_)) => Unexpected::Other("Duration"),
            BoltRef::Type(BoltType::Date(_)) => Unexpected::Other("Date"),
            BoltRef::Type(BoltType::Time(_)) => Unexpected::Other("Time"),
            BoltRef::Type(BoltType::LocalTime(_)) => Unexpected::Other("LocalTime"),
            BoltRef::Type(BoltType::DateTime(_)) => Unexpected::Other("DateTime"),
            BoltRef::Type(BoltType::LocalDateTime(_)) => Unexpected::Other("LocalDateTime"),
            BoltRef::Type(BoltType::DateTimeZoneId(_)) => Unexpected::Other("DateTimeZoneId"),
        };

        Err(Error::invalid_type(typ, &visitor))
    }
}

#[derive(Copy, Clone)]
enum BoltRef<'de> {
    Type(&'de BoltType),
    Node(&'de BoltNode),
    Rel(&'de BoltRelation),
    URel(&'de BoltUnboundedRelation),
}

#[allow(unused)]
struct AdditionalDataDeserializer<'de, T> {
    data: T,
    _lifetime: PhantomData<&'de ()>,
}

#[allow(unused)]
impl<'de, T: AdditionalData<'de>> Deserializer<'de> for AdditionalDataDeserializer<'de, T> {
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
            Err(Error::invalid_length(
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
            _ => Err(Error::invalid_length(fields.len(), &"1")),
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
        name: &'static str,
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::custom(
            "deserializing additional data requires a struct",
        ))
    }
}

impl<'de, T: AdditionalData<'de>> AdditionalDataDeserializer<'de, T> {
    fn new(data: T) -> Self {
        Self {
            data,
            _lifetime: PhantomData,
        }
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
                    .ok_or_else(|| Error::missing_field("start_node_id"))?
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
                    .ok_or_else(|| Error::missing_field("end_node_id"))?
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
                    .ok_or_else(|| Error::missing_field("labels"))?;
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
                    .ok_or_else(|| Error::missing_field("type"))?
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
            _ => Err(Error::invalid_type(
                Unexpected::Other(&format!("struct `{}`", name)),
                &"one of `Id`, `Labels`, `Type`, `StartNodeId`, `EndNodeId`, or `Keys`",
            )),
        }
    }
}

trait AdditionalData<'de> {
    fn id(self) -> &'de BoltInteger;

    fn start_node_id(self) -> Option<&'de BoltInteger>;

    fn end_node_id(self) -> Option<&'de BoltInteger>;

    type Labels: Iterator<Item = &'de BoltType>;
    fn labels(self) -> Option<Self::Labels>;

    fn typ(self) -> Option<&'de BoltString>;

    fn properties(self) -> &'de BoltMap;
}

impl<'de> AdditionalData<'de> for &'de BoltNode {
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
}

impl<'de> AdditionalData<'de> for &'de BoltRelation {
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
}

impl<'de> AdditionalData<'de> for &'de BoltUnboundedRelation {
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
}

impl Error for DeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Error(msg.to_string())
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltType {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(BoltRef::Type(self))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltNode {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(BoltRef::Node(self))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltRelation {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(BoltRef::Rel(self))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltUnboundedRelation {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(BoltRef::URel(self))
    }
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltString {
    type Deserializer = BorrowedStrDeserializer<'de, DeError>;

    fn into_deserializer(self) -> Self::Deserializer {
        BorrowedStrDeserializer::new(&self.value)
    }
}

impl<'de, T: AdditionalData<'de>> IntoDeserializer<'de, DeError> for ElementData<'de, T> {
    type Deserializer = ElementDataDeserializer<'de, T>;

    fn into_deserializer(self) -> Self::Deserializer {
        match self {
            ElementData::Property(v) => ElementDataDeserializer::Property(v.into_deserializer()),
            ElementData::Additional(v) => {
                ElementDataDeserializer::Additional(AdditionalDataDeserializer::new(v))
            }
        }
    }
}

enum Visitation {
    Newtype,
    Tuple,
    Struct(&'static str),
}

enum ElementData<'de, T> {
    Property(&'de BoltType),
    Additional(T),
}

enum ElementDataDeserializer<'de, T> {
    Property(BoltTypeDeserializer<'de>),
    Additional(AdditionalDataDeserializer<'de, T>),
}

impl<'de, T: AdditionalData<'de>> Deserializer<'de> for ElementDataDeserializer<'de, T> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_any(visitor),
            Self::Additional(v) => v.deserialize_any(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_bool(visitor),
            Self::Additional(v) => v.deserialize_bool(visitor),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i8(visitor),
            Self::Additional(v) => v.deserialize_i8(visitor),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i16(visitor),
            Self::Additional(v) => v.deserialize_i16(visitor),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i32(visitor),
            Self::Additional(v) => v.deserialize_i32(visitor),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i64(visitor),
            Self::Additional(v) => v.deserialize_i64(visitor),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u8(visitor),
            Self::Additional(v) => v.deserialize_u8(visitor),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u16(visitor),
            Self::Additional(v) => v.deserialize_u16(visitor),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u32(visitor),
            Self::Additional(v) => v.deserialize_u32(visitor),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u64(visitor),
            Self::Additional(v) => v.deserialize_u64(visitor),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_f32(visitor),
            Self::Additional(v) => v.deserialize_f32(visitor),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_f64(visitor),
            Self::Additional(v) => v.deserialize_f64(visitor),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_char(visitor),
            Self::Additional(v) => v.deserialize_char(visitor),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_str(visitor),
            Self::Additional(v) => v.deserialize_str(visitor),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_string(visitor),
            Self::Additional(v) => v.deserialize_string(visitor),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_bytes(visitor),
            Self::Additional(v) => v.deserialize_bytes(visitor),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_byte_buf(visitor),
            Self::Additional(v) => v.deserialize_byte_buf(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_option(visitor),
            Self::Additional(v) => v.deserialize_option(visitor),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_unit(visitor),
            Self::Additional(v) => v.deserialize_unit(visitor),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_unit_struct(name, visitor),
            Self::Additional(v) => v.deserialize_unit_struct(name, visitor),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_newtype_struct(name, visitor),
            Self::Additional(v) => v.deserialize_newtype_struct(name, visitor),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_seq(visitor),
            Self::Additional(v) => v.deserialize_seq(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_tuple(len, visitor),
            Self::Additional(v) => v.deserialize_tuple(len, visitor),
        }
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
        match self {
            Self::Property(v) => v.deserialize_tuple_struct(name, len, visitor),
            Self::Additional(v) => v.deserialize_tuple_struct(name, len, visitor),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_map(visitor),
            Self::Additional(v) => v.deserialize_map(visitor),
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
        match self {
            Self::Property(v) => v.deserialize_struct(name, fields, visitor),
            Self::Additional(v) => v.deserialize_struct(name, fields, visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_enum(name, variants, visitor),
            Self::Additional(v) => v.deserialize_enum(name, variants, visitor),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_identifier(visitor),
            Self::Additional(v) => v.deserialize_identifier(visitor),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_ignored_any(visitor),
            Self::Additional(v) => v.deserialize_ignored_any(visitor),
        }
    }
}

trait FromFloat {
    fn from_float(f: f64) -> Self;
}

impl FromFloat for f32 {
    fn from_float(f: f64) -> Self {
        f as f32
    }
}

impl FromFloat for f64 {
    fn from_float(f: f64) -> Self {
        f
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::{
        borrow::Cow,
        fmt::Debug,
        marker::PhantomData,
        sync::atomic::{AtomicU32, Ordering},
    };

    use super::*;
    use crate::types::BoltNull;

    #[test]
    fn map_with_extra_fields() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
            (BoltString::from("bar"), BoltType::from(1337)),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let map = BoltType::Map(map);

        let actual = map.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice".into(),
            age: 42,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn map_with_extra_fields_fails_for_deny_unknown_fields() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Person {
            name: String,
            age: u8,
        }

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
            (BoltString::from("bar"), BoltType::from(1337)),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let map = BoltType::Map(map);

        assert!(map.to::<Person>().is_err());
    }

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
    fn tuple_struct_from_list() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person(String, u8);

        let list = BoltType::from(vec![BoltType::from("Alice"), BoltType::from(42)]);
        let actual = list.to::<Person>().unwrap();
        let expected = Person("Alice".into(), 42);

        assert_eq!(actual, expected);
    }

    #[test]
    fn tuple_struct_from_map_fails() {
        // We do not support this since maps are unordered and
        // we cannot gurantee that the values are in the same
        // order as the tuple struct fields.
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person(String, u8);

        let map = [
            (BoltString::from("name"), BoltType::from("Alice")),
            (BoltString::from("age"), BoltType::from(42)),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let map = BoltType::Map(map);

        assert!(map.to::<Person>().is_err());
    }

    fn test_node() -> BoltType {
        let id = BoltInteger::new(1337);
        let labels = vec!["Person".into()].into();
        let properties = vec![
            ("name".into(), "Alice".into()),
            ("age".into(), 42_u16.into()),
        ]
        .into_iter()
        .collect();

        let node = BoltNode {
            id,
            labels,
            properties,
        };
        BoltType::Node(node)
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

        let node = match node {
            BoltType::Node(node) => node,
            _ => unreachable!(),
        };
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

        let node = match node {
            BoltType::Node(node) => node,
            _ => unreachable!(),
        };

        let id = node.to::<Id>().unwrap();
        let labels = node.to::<Labels>().unwrap();
        let keys = node.to::<Keys>().unwrap();

        assert_eq!(id, Id(1337));
        assert_eq!(labels, Labels(vec!["Person".to_owned()]));
        assert_eq!(keys, Keys(["name".to_owned(), "age".to_owned()].into()));
    }

    fn test_relation() -> BoltType {
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

        let relation = BoltRelation {
            id,
            start_node_id,
            end_node_id,
            properties,
            typ,
        };
        BoltType::Relation(relation)
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

        let relation = match relation {
            BoltType::Relation(relation) => relation,
            _ => unreachable!(),
        };
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

        let relation = match relation {
            BoltType::Relation(relation) => relation,
            _ => unreachable!(),
        };

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

    fn test_unbounded_relation() -> BoltType {
        let id = BoltInteger::new(1337);
        let typ = "Person".into();
        let properties = vec![
            ("name".into(), "Alice".into()),
            ("age".into(), 42_u16.into()),
        ]
        .into_iter()
        .collect();

        let relation = BoltUnboundedRelation {
            id,
            properties,
            typ,
        };
        BoltType::UnboundedRelation(relation)
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

        let relation = test_unbounded_relation();

        let actual = relation.to::<Person>().unwrap();

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

        let relation = test_unbounded_relation();

        let actual = relation.to::<Person>().unwrap();

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
        let relation = test_unbounded_relation();
        let actual = relation.to::<Person>().unwrap();
        assert_eq!(actual, expected);

        let relation = match relation {
            BoltType::UnboundedRelation(relation) => relation,
            _ => unreachable!(),
        };
        let actual = relation.to::<Person>().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_just_extract_unbounded_relation_extra() {
        let relation = test_unbounded_relation();

        let id = relation.to::<Id>().unwrap();
        let typ = relation.to::<Type>().unwrap();
        let keys = relation.to::<Keys>().unwrap();

        assert_eq!(id, Id(1337));
        assert_eq!(typ, Type("Person".to_owned()));
        assert_eq!(keys, Keys(["name".to_owned(), "age".to_owned()].into()));

        let relation = match relation {
            BoltType::UnboundedRelation(relation) => relation,
            _ => unreachable!(),
        };

        let id = relation.to::<Id>().unwrap();
        let typ = relation.to::<Type>().unwrap();
        let keys = relation.to::<Keys>().unwrap();

        assert_eq!(id, Id(1337));
        assert_eq!(typ, Type("Person".to_owned()));
        assert_eq!(keys, Keys(["name".to_owned(), "age".to_owned()].into()));
    }

    #[test]
    fn tuple() {
        let list = BoltType::from(vec![BoltType::from("Alice"), BoltType::from(42)]);
        let actual = list.to::<(String, u8)>().unwrap();
        let expected = ("Alice".into(), 42);

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
            (BoltString::from("unit"), BoltType::Null(BoltNull)),
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
    fn std_bytes() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Bytes<'a> {
            bytes: Vec<u8>,
            slice: &'a [u8],
        }

        let map = [
            (BoltString::from("bytes"), BoltType::from(vec![4_u8, 2])),
            (
                BoltString::from("slice"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Bytes>().unwrap();
        let expected = Bytes {
            bytes: vec![4, 2],
            slice: &[1, 3, 3, 7],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn serde_bytes_bytes() {
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct Bytes<'a> {
            #[serde(with = "serde_bytes")]
            bytes: Vec<u8>,
            #[serde(with = "serde_bytes")]
            slice: &'a [u8],
        }

        let map = [
            (BoltString::from("bytes"), BoltType::from(vec![4_u8, 2])),
            (
                BoltString::from("slice"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<Bytes>().unwrap();
        let expected = Bytes {
            bytes: vec![4, 2],
            slice: &[1, 3, 3, 7],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn serde_with_bytes() {
        use serde_with::{serde_as, Bytes};

        #[serde_as]
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        struct AsBytes<'a> {
            #[serde_as(as = "Bytes")]
            array: [u8; 4],

            #[serde_as(as = "Bytes")]
            boxed: Box<[u8]>,

            #[serde_as(as = "Bytes")]
            #[serde(borrow)]
            cow: Cow<'a, [u8]>,

            #[serde_as(as = "Bytes")]
            #[serde(borrow)]
            cow_array: Cow<'a, [u8; 2]>,

            #[serde_as(as = "Bytes")]
            bytes: Vec<u8>,

            #[serde_as(as = "Bytes")]
            slice: &'a [u8],
        }

        let map = [
            (
                BoltString::from("array"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
            (BoltString::from("boxed"), BoltType::from(vec![4_u8, 2])),
            (BoltString::from("cow"), BoltType::from(vec![1_u8, 3, 3, 7])),
            (BoltString::from("cow_array"), BoltType::from(vec![4_u8, 2])),
            (
                BoltString::from("bytes"),
                BoltType::from(vec![1_u8, 3, 3, 7]),
            ),
            (BoltString::from("slice"), BoltType::from(vec![4_u8, 2])),
        ]
        .into_iter()
        .collect::<BoltMap>();

        let actual = map.to::<AsBytes>().unwrap();
        let expected = AsBytes {
            array: [1, 3, 3, 7],
            boxed: vec![4, 2].into_boxed_slice(),
            cow: vec![1_u8, 3, 3, 7].into(),
            cow_array: Cow::Owned([4_u8, 2]),
            bytes: vec![1, 3, 3, 7],
            slice: &[4, 2],
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

    #[test]
    fn list() {
        let list = BoltType::from(vec![42_i64, 1337]);
        let actual = list.to::<Vec<i64>>().unwrap();

        assert_eq!(actual, vec![42_i64, 1337]);
    }

    #[test]
    fn nested_list() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Foo {
            bars: Vec<i64>,
        }

        let data = [(BoltString::from("bars"), BoltType::from(vec![42, 1337]))]
            .into_iter()
            .collect::<BoltMap>();
        let actual = data.to::<Foo>().unwrap();
        let expected = Foo {
            bars: vec![42, 1337],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn type_convert() {
        let i = BoltType::from(42);

        assert_eq!(i.to::<i8>().unwrap(), 42);
    }

    #[test]
    fn type_convert_error() {
        let i = BoltType::from(1337);

        assert_eq!(
            i.to::<i8>().unwrap_err().to_string(),
            "Could not convert the integer `1337` to the target type i8"
        );
    }
}
