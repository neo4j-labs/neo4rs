use crate::types::{
    serde::node::BoltNodeVisitor, BoltBoolean, BoltBytes, BoltFloat, BoltInteger, BoltKind,
    BoltList, BoltMap, BoltNull, BoltString, BoltType,
};

use bytes::Bytes;
use serde::{
    de::{Deserializer, Error, VariantAccess, Visitor},
    Deserialize,
};
use std::{fmt, result::Result};

impl<'de> Deserialize<'de> for BoltType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_enum(std::any::type_name::<BoltType>(), &[], BoltTypeVisitor)
    }
}

struct BoltTypeVisitor;

impl<'de> Visitor<'de> for BoltTypeVisitor {
    type Value = BoltType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid bolt type")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Boolean(BoltBoolean::new(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Integer(BoltInteger::new(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match i64::try_from(v) {
            Ok(v) => self.visit_i64(v),
            Err(_) => Err(E::custom(format!("i128 out of range: {}", v))),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match i64::try_from(v) {
            Ok(v) => self.visit_i64(v),
            Err(_) => Err(E::custom(format!("u64 out of range: {}", v))),
        }
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match i64::try_from(v) {
            Ok(v) => self.visit_i64(v),
            Err(_) => Err(E::custom(format!("u128 out of range: {}", v))),
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Float(BoltFloat::new(v)))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::String(BoltString::new(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Bytes(BoltBytes::new(Bytes::copy_from_slice(v))))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Null(BoltNull))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ::serde::de::Deserialize::deserialize(deserializer)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Null(BoltNull))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visit_some(deserializer)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::SeqAccess<'de>,
    {
        let mut items = match seq.size_hint() {
            Some(s) => BoltList::with_capacity(s),
            None => BoltList::new(),
        };

        while let Some(next) = seq.next_element()? {
            items.push(next);
        }

        Ok(BoltType::List(items))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut items = match map.size_hint() {
            Some(s) => BoltMap::with_capacity(s),
            None => BoltMap::new(),
        };

        while let Some((key, value)) = map.next_entry()? {
            items.put(key, value);
        }

        Ok(BoltType::Map(items))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::String(BoltString { value: v }))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BoltType::Bytes(BoltBytes {
            value: Bytes::from(v),
        }))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (kind, variant): (BoltKind, _) = data.variant()?;
        match kind {
            BoltKind::Null => variant.tuple_variant(1, self),
            BoltKind::String => variant.tuple_variant(1, self),
            BoltKind::Boolean => variant.tuple_variant(1, self),
            BoltKind::Map => variant.tuple_variant(1, self),
            BoltKind::Integer => variant.tuple_variant(1, self),
            BoltKind::Float => variant.tuple_variant(1, self),
            BoltKind::List => variant.tuple_variant(1, self),
            BoltKind::Node => variant
                .tuple_variant(1, BoltNodeVisitor)
                .map(BoltType::Node),
            BoltKind::Relation => variant.tuple_variant(1, self),
            BoltKind::UnboundedRelation => variant.tuple_variant(1, self),
            BoltKind::Point2D => variant.tuple_variant(1, self),
            BoltKind::Point3D => variant.tuple_variant(1, self),
            BoltKind::Bytes => variant.tuple_variant(1, self),
            BoltKind::Path => variant.tuple_variant(1, self),
            BoltKind::Duration => variant.tuple_variant(1, self),
            BoltKind::Date => variant.tuple_variant(1, self),
            BoltKind::Time => variant.tuple_variant(1, self),
            BoltKind::LocalTime => variant.tuple_variant(1, self),
            BoltKind::DateTime => variant.tuple_variant(1, self),
            BoltKind::LocalDateTime => variant.tuple_variant(1, self),
            BoltKind::DateTimeZoneId => variant.tuple_variant(1, self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{BoltMap, BoltNull, BoltType};

    #[test]
    fn roundtrips() {
        let map = [
            ("age".into(), 42.into()),
            ("awesome".into(), true.into()),
            ("values".into(), vec![13.37, 42.84].into()),
            ("payload".into(), b"Hello, World!".as_slice().into()),
            ("secret".into(), BoltType::Null(BoltNull)),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let map = BoltType::Map(map);

        let actual = map.to::<BoltType>().unwrap();
        assert_eq!(actual, map);
    }
}
