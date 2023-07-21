use std::iter;

use ::serde::{
    de::{self, value::StrDeserializer},
    forward_to_deserialize_any, Deserialize,
};
use serde::de::value::{I64Deserializer, MapDeserializer, SeqDeserializer};

use crate::types::{BoltMap, BoltNode, BoltString, BoltType};

/// Newtype to extract the node id during deserialization.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub struct Id(pub u64);

impl BoltMap {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(MapDeserializer::new(self.value.iter()))
    }
}

impl BoltType {
    #[allow(unused)]
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(BoltTypeDeserializer::new(self))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeError {
    #[error("{0}")]
    Error(String),

    #[error("Could not convert the integer `{1}` to the target type {2}")]
    IntegerOutOfBounds(#[source] std::num::TryFromIntError, i64, &'static str),
}

impl de::Error for DeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Error(msg.to_string())
    }
}

pub struct BoltTypeDeserializer<'de> {
    value: &'de BoltType,
}

impl<'de> BoltTypeDeserializer<'de> {
    fn new(value: &'de BoltType) -> Self {
        Self { value }
    }
}

impl<'de> de::IntoDeserializer<'de, DeError> for &'de BoltType {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(self)
    }
}

impl<'de> de::IntoDeserializer<'de, DeError> for &'de BoltString {
    type Deserializer = StrDeserializer<'de, DeError>;

    fn into_deserializer(self) -> Self::Deserializer {
        StrDeserializer::new(&self.value)
    }
}

impl<'de> de::Deserializer<'de> for BoltTypeDeserializer<'de> {
    type Error = DeError;

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::List(v) => visitor.visit_seq(SeqDeserializer::new(v.value.iter())),
            BoltType::Bytes(v) => visitor.visit_seq(SeqDeserializer::new(v.value.iter().copied())),
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::Map(v) => visitor.visit_map(MapDeserializer::new(v.value.iter())),
            BoltType::Node(v) => visitor.visit_map(MapDeserializer::new(v.properties.value.iter())),
            BoltType::Relation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltType::UnboundedRelation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
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
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::Map(v) => visitor.visit_map(MapDeserializer::new(v.value.iter())),
            BoltType::Node(v) => {
                let additional_fields = fields
                    .iter()
                    .copied()
                    .filter(|f| !v.properties.value.contains_key(*f))
                    .map(|f| (f, NodeData::Additional(v)));
                let property_fields = v
                    .properties
                    .value
                    .iter()
                    .map(|(k, v)| (k.value.as_str(), NodeData::Property(v)));
                let node_fields = property_fields.chain(additional_fields);
                visitor.visit_map(MapDeserializer::new(node_fields))
            }
            BoltType::Relation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            BoltType::UnboundedRelation(v) => {
                visitor.visit_map(MapDeserializer::new(v.properties.value.iter()))
            }
            _ => self.unexpected(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BoltType::List(v) if v.len() == len => {
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
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::String(v) = self.value {
            visitor.visit_borrowed_str(&v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::String(v) = self.value {
            visitor.visit_string(v.value.clone())
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Bytes(v) = self.value {
            visitor.visit_borrowed_bytes(&v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Bytes(v) = self.value {
            visitor.visit_byte_buf(v.value.to_vec())
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Boolean(v) = self.value {
            visitor.visit_bool(v.value)
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_integer(visitor)?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_float(visitor)?;
        visitor.visit_f32(v)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (v, visitor) = self.read_float(visitor)?;
        visitor.visit_f64(v)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let BoltType::Null(_) = self.value {
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
        V: de::Visitor<'de>,
    {
        if let BoltType::Null(_) = self.value {
            visitor.visit_unit()
        } else {
            self.unexpected(visitor)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.unexpected(visitor)
    }

    forward_to_deserialize_any! {
        char option newtype_struct enum identifier
    }
}

impl<'de> BoltTypeDeserializer<'de> {
    fn read_integer<T, E, V>(self, visitor: V) -> Result<(T, V), DeError>
    where
        V: de::Visitor<'de>,
        i64: TryInto<T, Error = E>,
        E: Into<std::num::TryFromIntError>,
    {
        if let BoltType::Integer(v) = self.value {
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
        V: de::Visitor<'de>,
        T: FromFloat,
    {
        if let BoltType::Float(v) = self.value {
            Ok((T::from_float(v.value), visitor))
        } else {
            self.unexpected(visitor)
        }
    }

    fn unexpected<V, T>(self, visitor: V) -> Result<T, DeError>
    where
        V: de::Visitor<'de>,
    {
        let typ = match self.value {
            BoltType::String(v) => de::Unexpected::Str(&v.value),
            BoltType::Boolean(v) => de::Unexpected::Bool(v.value),
            BoltType::Map(_) => de::Unexpected::Map,
            BoltType::Null(_) => de::Unexpected::Unit,
            BoltType::Integer(v) => de::Unexpected::Signed(v.value),
            BoltType::Float(v) => de::Unexpected::Float(v.value),
            BoltType::List(_) => de::Unexpected::Seq,
            BoltType::Node(_) => de::Unexpected::Map,
            BoltType::Relation(_) => de::Unexpected::Map,
            BoltType::UnboundedRelation(_) => de::Unexpected::Map,
            BoltType::Point2D(_) => de::Unexpected::Other("Point2D"),
            BoltType::Point3D(_) => de::Unexpected::Other("Point3D"),
            BoltType::Bytes(v) => de::Unexpected::Bytes(&v.value),
            BoltType::Path(_) => de::Unexpected::Other("Path"),
            BoltType::Duration(_) => de::Unexpected::Other("Duration"),
            BoltType::Date(_) => de::Unexpected::Other("Date"),
            BoltType::Time(_) => de::Unexpected::Other("Time"),
            BoltType::LocalTime(_) => de::Unexpected::Other("LocalTime"),
            BoltType::DateTime(_) => de::Unexpected::Other("DateTime"),
            BoltType::LocalDateTime(_) => de::Unexpected::Other("LocalDateTime"),
            BoltType::DateTimeZoneId(_) => de::Unexpected::Other("DateTimeZoneId"),
        };

        Err(de::Error::invalid_type(typ, &visitor))
    }
}

enum NodeData<'de> {
    Property(&'de BoltType),
    Additional(&'de BoltNode),
}

impl<'de> de::IntoDeserializer<'de, DeError> for NodeData<'de> {
    type Deserializer = NodeDataDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        match self {
            NodeData::Property(v) => NodeDataDeserializer::Property(BoltTypeDeserializer::new(v)),
            NodeData::Additional(v) => {
                NodeDataDeserializer::Additional(AddidtionalNodeDataDeserializer { node: v })
            }
        }
    }
}

enum NodeDataDeserializer<'de> {
    Property(BoltTypeDeserializer<'de>),
    Additional(AddidtionalNodeDataDeserializer<'de>),
}

impl<'de> de::Deserializer<'de> for NodeDataDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_any(visitor),
            Self::Additional(v) => v.deserialize_any(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_bool(visitor),
            Self::Additional(v) => v.deserialize_bool(visitor),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i8(visitor),
            Self::Additional(v) => v.deserialize_i8(visitor),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i16(visitor),
            Self::Additional(v) => v.deserialize_i16(visitor),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i32(visitor),
            Self::Additional(v) => v.deserialize_i32(visitor),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_i64(visitor),
            Self::Additional(v) => v.deserialize_i64(visitor),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u8(visitor),
            Self::Additional(v) => v.deserialize_u8(visitor),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u16(visitor),
            Self::Additional(v) => v.deserialize_u16(visitor),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u32(visitor),
            Self::Additional(v) => v.deserialize_u32(visitor),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_u64(visitor),
            Self::Additional(v) => v.deserialize_u64(visitor),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_f32(visitor),
            Self::Additional(v) => v.deserialize_f32(visitor),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_f64(visitor),
            Self::Additional(v) => v.deserialize_f64(visitor),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_char(visitor),
            Self::Additional(v) => v.deserialize_char(visitor),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_str(visitor),
            Self::Additional(v) => v.deserialize_str(visitor),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_string(visitor),
            Self::Additional(v) => v.deserialize_string(visitor),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_bytes(visitor),
            Self::Additional(v) => v.deserialize_bytes(visitor),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_byte_buf(visitor),
            Self::Additional(v) => v.deserialize_byte_buf(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_option(visitor),
            Self::Additional(v) => v.deserialize_option(visitor),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_newtype_struct(name, visitor),
            Self::Additional(v) => v.deserialize_newtype_struct(name, visitor),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_seq(visitor),
            Self::Additional(v) => v.deserialize_seq(visitor),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_tuple_struct(name, len, visitor),
            Self::Additional(v) => v.deserialize_tuple_struct(name, len, visitor),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_enum(name, variants, visitor),
            Self::Additional(v) => v.deserialize_enum(name, variants, visitor),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_identifier(visitor),
            Self::Additional(v) => v.deserialize_identifier(visitor),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Property(v) => v.deserialize_ignored_any(visitor),
            Self::Additional(v) => v.deserialize_ignored_any(visitor),
        }
    }
}

#[allow(unused)]
struct AddidtionalNodeDataDeserializer<'de> {
    node: &'de BoltNode,
}

enum Visitation {
    Newtype,
    Tuple,
    Struct(&'static str),
}

impl<'de> AddidtionalNodeDataDeserializer<'de> {
    fn deserialize_any_struct<V>(
        self,
        name: &str,
        visitor: V,
        visitation: Visitation,
    ) -> Result<V::Value, DeError>
    where
        V: de::Visitor<'de>,
    {
        struct LabelsDeserializer<'de>(std::slice::Iter<'de, BoltType>);

        impl<'de> de::IntoDeserializer<'de, DeError> for LabelsDeserializer<'de> {
            type Deserializer = SeqDeserializer<std::slice::Iter<'de, BoltType>, DeError>;

            fn into_deserializer(self) -> Self::Deserializer {
                SeqDeserializer::new(self.0)
            }
        }

        match name {
            "Id" => match visitation {
                Visitation::Newtype => {
                    visitor.visit_newtype_struct(I64Deserializer::new(self.node.id.value))
                }
                Visitation::Tuple => {
                    Ok(visitor.visit_seq(SeqDeserializer::new(iter::once(self.node.id.value))))?
                }
                Visitation::Struct(field) => Ok(visitor.visit_map(MapDeserializer::new(
                    iter::once((field, self.node.id.value)),
                ))?),
            },
            "Labels" => match visitation {
                Visitation::Newtype => visitor
                    .visit_newtype_struct(SeqDeserializer::new(self.node.labels.value.iter())),
                Visitation::Tuple => Ok(visitor.visit_seq(SeqDeserializer::new(iter::once(
                    LabelsDeserializer(self.node.labels.value.iter()),
                ))))?,
                Visitation::Struct(field) => Ok(visitor.visit_map(MapDeserializer::new(
                    iter::once((field, LabelsDeserializer(self.node.labels.value.iter()))),
                ))?),
            },
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other(&format!("struct {}", name)),
                &"struct `Id` or struct `Labels`",
            )),
        }
    }
}

#[allow(unused)]
impl<'de> de::Deserializer<'de> for AddidtionalNodeDataDeserializer<'de> {
    type Error = DeError;

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
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
        V: de::Visitor<'de>,
    {
        if len == 1 {
            self.deserialize_any_struct(name, visitor, Visitation::Tuple)
        } else {
            Err(de::Error::invalid_length(
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
        V: de::Visitor<'de>,
    {
        match fields {
            [field] => self.deserialize_any_struct(name, visitor, Visitation::Struct(field)),
            _ => Err(de::Error::invalid_length(fields.len(), &"1")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option seq tuple map enum identifier
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::custom(
            "deserializing node id or labels requires a struct",
        ))
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
    use crate::types::{BoltInteger, BoltNode, BoltNull, BoltRelation};

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

    #[test]
    fn node() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let id = BoltInteger::new(42);
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
        let node = BoltType::Node(node);

        let actual = node.to::<Person>().unwrap();
        let expected = Person {
            name: "Alice".to_owned(),
            age: 42,
        };

        assert_eq!(actual, expected);
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
        test_extract_node_id(Id(1337));
    }

    #[test]
    fn extract_node_id_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Id(i16);

        test_extract_node_id(Id(1337));
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

        test_extract_node_id(Id { id: 1337.into() });
    }

    #[test]
    fn extract_node_labels_with_custom_newtype() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Labels([String; 1]);

        test_extract_node_labels(Labels(["Person".to_owned()]));
    }

    #[test]
    fn extract_node_labels_with_custom_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Labels {
            labels: Vec<String>,
        }

        test_extract_node_labels(Labels {
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
        let node = BoltType::Node(node);

        let actual = node.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    fn test_extract_node_id<T: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: T) {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<T> {
            id: T,
            name: String,
            age: u8,
        }

        let expected = Person {
            id: expected,
            name: "Alice".to_owned(),
            age: 42,
        };

        test_extract_node(expected);
    }

    fn test_extract_node_labels<T: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: T) {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Person<T> {
            labels: T,
            name: String,
            age: u8,
        }

        let expected = Person {
            labels: expected,
            name: "Alice".to_owned(),
            age: 42,
        };

        test_extract_node(expected);
    }

    fn test_extract_node<Person: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: Person) {
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
        let node = BoltType::Node(node);

        let actual = node.to::<Person>().unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn relation() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct Knows {
            since: u16,
        }

        let id = BoltInteger::new(42);
        let start_node_id = BoltInteger::new(13);
        let end_node_id = BoltInteger::new(37);
        let typ = BoltString::new("REL");
        let properties = vec![("since".into(), 1337_u16.into())]
            .into_iter()
            .collect();

        let relation = BoltRelation {
            id,
            start_node_id,
            end_node_id,
            typ,
            properties,
        };
        let relation = BoltType::Relation(relation);

        let actual = relation.to::<Knows>().unwrap();
        let expected = Knows { since: 1337 };

        assert_eq!(actual, expected);
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
