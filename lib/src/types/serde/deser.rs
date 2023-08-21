use crate::{
    types::{
        serde::element::ElementDataDeserializer, BoltKind, BoltRelation, BoltString, BoltType,
        BoltUnboundedRelation,
    },
    DeError,
};

use std::result::Result;

use serde::{
    de::{
        value::{BorrowedStrDeserializer, MapDeserializer, SeqDeserializer},
        DeserializeSeed, Deserializer, EnumAccess, Error, IntoDeserializer, Unexpected as Unexp,
        VariantAccess, Visitor,
    },
    forward_to_deserialize_any,
};

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
            BoltRef::Type(BoltType::Node(v)) => v.into_deserializer().deserialize_map(visitor),
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
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltRef::Type(BoltType::Map(v)) => {
                visitor.visit_map(MapDeserializer::new(v.value.iter()))
            }
            BoltRef::Type(BoltType::Node(v)) => v
                .into_deserializer()
                .deserialize_struct(name, fields, visitor),
            BoltRef::Type(BoltType::Relation(v)) => {
                ElementDataDeserializer::new(v).deserialize_outer_struct(fields, visitor)
            }
            BoltRef::Rel(v) => {
                ElementDataDeserializer::new(v).deserialize_outer_struct(fields, visitor)
            }
            BoltRef::Type(BoltType::UnboundedRelation(v)) => {
                ElementDataDeserializer::new(v).deserialize_outer_struct(fields, visitor)
            }
            BoltRef::URel(v) => {
                ElementDataDeserializer::new(v).deserialize_outer_struct(fields, visitor)
            }
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
            BoltRef::Type(BoltType::Node(v)) => v
                .into_deserializer()
                .deserialize_newtype_struct(name, visitor),
            BoltRef::Type(BoltType::Relation(v)) => {
                ElementDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::Rel(v) => {
                ElementDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::Type(BoltType::UnboundedRelation(v)) => {
                ElementDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
            }
            BoltRef::URel(v) => {
                ElementDataDeserializer::new(v).deserialize_newtype_struct(name, visitor)
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

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if name != std::any::type_name::<BoltType>() {
            return Err(DeError::invalid_type(Unexp::Str(name), &"BoltType"));
        }

        visitor.visit_enum(BoltEnum { value: self.value })
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
        char option identifier
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
            BoltRef::Type(BoltType::String(v)) => Unexp::Str(&v.value),
            BoltRef::Type(BoltType::Boolean(v)) => Unexp::Bool(v.value),
            BoltRef::Type(BoltType::Map(_)) => Unexp::Map,
            BoltRef::Type(BoltType::Null(_)) => Unexp::Unit,
            BoltRef::Type(BoltType::Integer(v)) => Unexp::Signed(v.value),
            BoltRef::Type(BoltType::Float(v)) => Unexp::Float(v.value),
            BoltRef::Type(BoltType::List(_)) => Unexp::Seq,
            BoltRef::Type(BoltType::Node(_)) => Unexp::Map,
            BoltRef::Type(BoltType::Relation(_)) => Unexp::Map,
            BoltRef::Rel(_) => Unexp::Map,
            BoltRef::Type(BoltType::UnboundedRelation(_)) => Unexp::Map,
            BoltRef::URel(_) => Unexp::Map,
            BoltRef::Type(BoltType::Point2D(_)) => Unexp::Other("Point2D"),
            BoltRef::Type(BoltType::Point3D(_)) => Unexp::Other("Point3D"),
            BoltRef::Type(BoltType::Bytes(v)) => Unexp::Bytes(&v.value),
            BoltRef::Type(BoltType::Path(_)) => Unexp::Other("Path"),
            BoltRef::Type(BoltType::Duration(_)) => Unexp::Other("Duration"),
            BoltRef::Type(BoltType::Date(_)) => Unexp::Other("Date"),
            BoltRef::Type(BoltType::Time(_)) => Unexp::Other("Time"),
            BoltRef::Type(BoltType::LocalTime(_)) => Unexp::Other("LocalTime"),
            BoltRef::Type(BoltType::DateTime(_)) => Unexp::Other("DateTime"),
            BoltRef::Type(BoltType::LocalDateTime(_)) => Unexp::Other("LocalDateTime"),
            BoltRef::Type(BoltType::DateTimeZoneId(_)) => Unexp::Other("DateTimeZoneId"),
        };

        Err(DeError::invalid_type(typ, &visitor))
    }
}

struct BoltEnum<'de> {
    value: BoltRef<'de>,
}

impl<'de> EnumAccess<'de> for BoltEnum<'de> {
    type Error = DeError;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let kind = match self.value {
            BoltRef::Type(value) => match value {
                BoltType::String(_) => BoltKind::String,
                BoltType::Boolean(_) => BoltKind::Boolean,
                BoltType::Map(_) => BoltKind::Map,
                BoltType::Null(_) => BoltKind::Null,
                BoltType::Integer(_) => BoltKind::Integer,
                BoltType::Float(_) => BoltKind::Float,
                BoltType::List(_) => BoltKind::List,
                BoltType::Node(_) => BoltKind::Node,
                BoltType::Relation(_) => BoltKind::Relation,
                BoltType::UnboundedRelation(_) => BoltKind::UnboundedRelation,
                BoltType::Point2D(_) => BoltKind::Point2D,
                BoltType::Point3D(_) => BoltKind::Point3D,
                BoltType::Bytes(_) => BoltKind::Bytes,
                BoltType::Path(_) => BoltKind::Path,
                BoltType::Duration(_) => BoltKind::Duration,
                BoltType::Date(_) => BoltKind::Date,
                BoltType::Time(_) => BoltKind::Time,
                BoltType::LocalTime(_) => BoltKind::LocalTime,
                BoltType::DateTime(_) => BoltKind::DateTime,
                BoltType::LocalDateTime(_) => BoltKind::LocalDateTime,
                BoltType::DateTimeZoneId(_) => BoltKind::DateTimeZoneId,
            },
            BoltRef::Rel(_) => BoltKind::Relation,
            BoltRef::URel(_) => BoltKind::UnboundedRelation,
        };
        let val = seed.deserialize(kind.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de> VariantAccess<'de> for BoltEnum<'de> {
    type Error = DeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(DeError::invalid_type(Unexp::TupleVariant, &"unit variant"))
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Err(DeError::invalid_type(
            Unexp::TupleVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltRef::Type(value) => match value {
                BoltType::String(s) => visitor.visit_borrowed_str(&s.value),
                BoltType::Boolean(b) => visitor.visit_bool(b.value),
                BoltType::Map(m) => visitor.visit_map(MapDeserializer::new(m.value.iter())),
                BoltType::Null(_) => visitor.visit_unit(),
                BoltType::Integer(i) => visitor.visit_i64(i.value),
                BoltType::Float(f) => visitor.visit_f64(f.value),
                BoltType::List(l) => visitor.visit_seq(SeqDeserializer::new(l.value.iter())),
                BoltType::Node(n) => ElementDataDeserializer::new(n).tuple_variant(len, visitor),
                BoltType::Relation(_) => todo!("relation as mapaccess visit_map"),
                BoltType::UnboundedRelation(_) => {
                    todo!("unbounded relation as mapaccess visit_map")
                }
                BoltType::Point2D(_) => todo!("point2d as mapaccess visit_map"),
                BoltType::Point3D(_) => todo!("point3d as mapaccess visit_map"),
                BoltType::Bytes(b) => visitor.visit_borrowed_bytes(&b.value),
                BoltType::Path(_) => todo!("path as mapaccess visit_map"),
                BoltType::Duration(_) => todo!("duration as mapaccess visit_map"),
                BoltType::Date(_) => todo!("date as mapaccess visit_map"),
                BoltType::Time(_) => todo!("time as mapaccess visit_map"),
                BoltType::LocalTime(_) => todo!("localtime as mapaccess visit_map"),
                BoltType::DateTime(_) => todo!("datetime as mapaccess visit_map"),
                BoltType::LocalDateTime(_) => todo!("localdatetime as mapaccess visit_map"),
                BoltType::DateTimeZoneId(_) => todo!("datetimezoneid as mapaccess visit_map"),
            },
            BoltRef::Rel(_) => todo!("relation as mapaccess visit_map"),
            BoltRef::URel(_) => todo!("unbounded relation as mapaccess visit_map"),
        }
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
            Unexp::TupleVariant,
            &"struct variant",
        ))
    }
}

#[derive(Copy, Clone)]
enum BoltRef<'de> {
    Type(&'de BoltType),
    Rel(&'de BoltRelation),
    URel(&'de BoltUnboundedRelation),
}

impl<'de> IntoDeserializer<'de, DeError> for &'de BoltType {
    type Deserializer = BoltTypeDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltTypeDeserializer::new(BoltRef::Type(self))
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

    use super::super::*;
    use crate::types::{BoltInteger, BoltNull, BoltString};
    use crate::{EndNodeId, Id, Keys, Labels, StartNodeId, Type};

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
            id: Id,
            labels: Labels,
            keys: Keys,
            name: String,
            age: u8,
        }

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
        let expected = Person {
            id: Id(1337),
            labels: Labels(vec!["Person".into()]),
            keys: Keys(["name".into(), "age".into()].into()),
            name: "Alice".into(),
            age: 42,
        };
        assert_eq!(actual, expected);
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
