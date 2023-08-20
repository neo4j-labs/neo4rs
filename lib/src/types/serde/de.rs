use crate::{
    types::{
        serde::element::ElementDataKey, BoltBoolean, BoltBytes, BoltFloat, BoltInteger, BoltKind,
        BoltList, BoltMap, BoltNode, BoltNull, BoltString, BoltType,
    },
    Id, Labels,
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

impl<'de> Deserialize<'de> for BoltNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
                        "::id" => {
                            builder.id(|| Ok(BoltInteger::new(map.next_value::<Id>()?.0 as i64)))?
                        }
                        "::labels" => {
                            builder.labels(|| Ok(map.next_value::<Labels<BoltList>>()?.0))?
                        }
                        otherwise => builder
                            .insert(|| Ok((BoltString::from(otherwise), map.next_value()?)))?,
                    }
                }

                let node = builder.build()?;
                Ok(node)
            }
        }
        deserializer.deserialize_struct("BoltNode", &["::id", "::labels"], BoltNodeVisitor)
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

#[derive(Debug, Copy, Clone)]
enum SetOnce<T> {
    Empty,
    Set(T),
}

impl<T> Default for SetOnce<T> {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SetOnceError;

impl<T> SetOnce<T> {
    fn insert(&mut self, value: T) -> Result<&mut T, SetOnceError> {
        self.insert_with(move || value)
    }

    fn get_or_insert_with(&mut self, value: impl FnOnce() -> T) -> &mut T {
        match self {
            SetOnce::Empty => self.insert_with(value).unwrap(),
            SetOnce::Set(value) => value,
        }
    }

    fn insert_default(&mut self) -> Result<&mut T, SetOnceError>
    where
        T: Default,
    {
        self.insert_with(Default::default)
    }

    fn insert_with(&mut self, value: impl FnOnce() -> T) -> Result<&mut T, SetOnceError> {
        match self {
            SetOnce::Empty => *self = Self::Set(value()),
            SetOnce::Set(_) => return Err(SetOnceError),
        };
        match self {
            SetOnce::Empty => unreachable!("value was just set"),
            SetOnce::Set(value) => Ok(value),
        }
    }

    fn try_insert_with<E>(
        &mut self,
        value: impl FnOnce() -> Result<T, E>,
    ) -> Result<Result<&mut T, SetOnceError>, E> {
        match self {
            SetOnce::Empty => *self = Self::Set(value()?),
            SetOnce::Set(_) => return Ok(Err(SetOnceError)),
        };
        match self {
            SetOnce::Empty => unreachable!("value was just set"),
            SetOnce::Set(value) => Ok(Ok(value)),
        }
    }

    fn ok_or_else<E>(self, missing: impl FnOnce() -> E) -> Result<T, E> {
        match self {
            SetOnce::Empty => Err(missing()),
            SetOnce::Set(value) => Ok(value),
        }
    }

    fn or_else(self, missing: impl FnOnce() -> T) -> T {
        match self {
            SetOnce::Empty => missing(),
            SetOnce::Set(value) => value,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct BoltNodeBuilder {
    id: SetOnce<BoltInteger>,
    labels: SetOnce<BoltList>,
    properties: SetOnce<BoltMap>,
}

impl BoltNodeBuilder {
    fn id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        match self.id.try_insert_with(read) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("id")),
        }
    }

    fn labels<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E> {
        match self.labels.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("labels")),
        }
    }

    fn properties<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltMap, E>) -> Result<(), E> {
        match self.properties.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("properties")),
        }
    }

    fn insert<E: Error>(
        &mut self,
        entry: impl FnOnce() -> Result<(BoltString, BoltType), E>,
    ) -> Result<(), E> {
        let props = self.properties.get_or_insert_with(Default::default);
        let (key, value) = entry()?;
        props.put(key, value);
        Ok(())
    }

    fn build<E: Error>(self) -> Result<BoltNode, E> {
        let id = self.id.ok_or_else(|| Error::missing_field("id"))?;
        let labels = self.labels.ok_or_else(|| Error::missing_field("labels"))?;
        let properties = self.properties.or_else(Default::default);
        Ok(BoltNode {
            id,
            labels,
            properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use tap::Tap;

    use crate::{
        types::{BoltMap, BoltNode, BoltNull, BoltType},
        Node,
    };

    fn test_node() -> BoltNode {
        let map = [
            ("age".into(), 42.into()),
            ("awesome".into(), true.into()),
            ("values".into(), vec![13.37, 42.84].into()),
            ("payload".into(), b"Hello, World!".as_slice().into()),
            ("secret".into(), BoltType::Null(BoltNull)),
        ]
        .into_iter()
        .collect::<BoltMap>();

        BoltNode::new(42.into(), vec!["Person".into()].into(), map)
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
    //
    // let rel = BoltRelation {
    //     id: 84.into(),
    //     start_node_id: 13.into(),
    //     end_node_id: 37.into(),
    //     typ: "KNOWS".into(),
    //     properties: map.clone(),
    // };
}
