use crate::types::{
    BoltInteger, BoltList, BoltMap, BoltNode, BoltPath, BoltRelation, BoltString, BoltType,
    BoltUnboundedRelation,
};

use std::result::Result;

use delegate::delegate;
use serde::{de::Error, Deserialize};

#[derive(Debug, Clone, Default)]
pub struct BoltNodeBuilder {
    inner: ElementBuilder,
}

impl BoltNodeBuilder {
    delegate! {
        to self.inner {
            pub fn id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E>;
            pub fn labels<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E>;
            pub fn properties<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltMap, E>) -> Result<(), E>;
            pub fn insert<E: Error>(&mut self, entry: impl FnOnce() -> Result<(BoltString, BoltType), E>) -> Result<(), E>;
        }
    }

    pub fn build<E: Error>(self) -> Result<BoltNode, E> {
        let id = self.inner.id.ok_or_else(|| Error::missing_field("id"))?;
        let labels = self
            .inner
            .labels
            .ok_or_else(|| Error::missing_field("labels"))?;
        let properties = self.inner.properties.or_else(Default::default);

        Ok(BoltNode {
            id,
            labels,
            properties,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoltRelationBuilder {
    inner: ElementBuilder,
}

impl BoltRelationBuilder {
    delegate! {
        to self.inner {
            pub fn id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E>;
            pub fn start_node_id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E>;
            pub fn end_node_id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E>;
            pub fn typ<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltString, E>) -> Result<(), E>;
            pub fn properties<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltMap, E>) -> Result<(), E>;
            pub fn insert<E: Error>(&mut self, entry: impl FnOnce() -> Result<(BoltString, BoltType), E>) -> Result<(), E>;
        }
    }

    pub fn build<E: Error>(self) -> Result<BoltRelation, E> {
        let id = self.inner.id.ok_or_else(|| Error::missing_field("id"))?;
        let start_node_id = self
            .inner
            .start_node_id
            .ok_or_else(|| Error::missing_field("start_node_id"))?;
        let end_node_id = self
            .inner
            .end_node_id
            .ok_or_else(|| Error::missing_field("end_node_id"))?;
        let typ = self.inner.typ.ok_or_else(|| Error::missing_field("type"))?;
        let properties = self.inner.properties.or_else(Default::default);

        Ok(BoltRelation {
            id,
            start_node_id,
            end_node_id,
            typ,
            properties,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoltUnboundedRelationBuilder {
    inner: ElementBuilder,
}

impl BoltUnboundedRelationBuilder {
    delegate! {
        to self.inner {
            pub fn id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E>;
            pub fn typ<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltString, E>) -> Result<(), E>;
            pub fn properties<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltMap, E>) -> Result<(), E>;
            pub fn insert<E: Error>(&mut self, entry: impl FnOnce() -> Result<(BoltString, BoltType), E>) -> Result<(), E>;
        }
    }

    pub fn build<E: Error>(self) -> Result<BoltUnboundedRelation, E> {
        let id = self.inner.id.ok_or_else(|| Error::missing_field("id"))?;
        let typ = self.inner.typ.ok_or_else(|| Error::missing_field("type"))?;
        let properties = self.inner.properties.or_else(Default::default);

        Ok(BoltUnboundedRelation {
            id,
            typ,
            properties,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoltPathBuilder {
    inner: ElementBuilder,
}

impl BoltPathBuilder {
    delegate! {
        to self.inner {
            pub fn nodes<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E>;
            pub fn relations<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E>;
            pub fn indices<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E>;
        }
    }

    pub fn build<E: Error>(self) -> Result<BoltPath, E> {
        let nodes = self
            .inner
            .nodes
            .ok_or_else(|| Error::missing_field("nodes"))?;
        let rels = self
            .inner
            .rels
            .ok_or_else(|| Error::missing_field("relations"))?;
        let indices = self
            .inner
            .indices
            .ok_or_else(|| Error::missing_field("indices"))?;

        Ok(BoltPath {
            nodes,
            rels,
            indices,
        })
    }
}

pub struct Id(pub BoltInteger);
pub struct StartNodeId(pub BoltInteger);
pub struct EndNodeId(pub BoltInteger);

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::Id::deserialize(deserializer).map(|id| Id(BoltInteger::new(id.0 as _)))
    }
}

impl<'de> Deserialize<'de> for StartNodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::StartNodeId::deserialize(deserializer)
            .map(|id| StartNodeId(BoltInteger::new(id.0 as _)))
    }
}

impl<'de> Deserialize<'de> for EndNodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::EndNodeId::deserialize(deserializer).map(|id| EndNodeId(BoltInteger::new(id.0 as _)))
    }
}

#[derive(Debug, Clone, Default)]
struct ElementBuilder {
    id: SetOnce<BoltInteger>,
    start_node_id: SetOnce<BoltInteger>,
    end_node_id: SetOnce<BoltInteger>,
    labels: SetOnce<BoltList>,
    typ: SetOnce<BoltString>,
    properties: SetOnce<BoltMap>,
    nodes: SetOnce<BoltList>,
    rels: SetOnce<BoltList>,
    indices: SetOnce<BoltList>,
}

impl ElementBuilder {
    fn id<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        match self.id.try_insert_with(read) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("id")),
        }
    }

    fn start_node_id<E: Error>(
        &mut self,
        read: impl FnOnce() -> Result<BoltInteger, E>,
    ) -> Result<(), E> {
        match self.start_node_id.try_insert_with(read) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("start_node_id")),
        }
    }

    fn end_node_id<E: Error>(
        &mut self,
        read: impl FnOnce() -> Result<BoltInteger, E>,
    ) -> Result<(), E> {
        match self.end_node_id.try_insert_with(read) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("end_node_id")),
        }
    }

    fn labels<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E> {
        match self.labels.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("labels")),
        }
    }

    fn typ<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltString, E>) -> Result<(), E> {
        match self.typ.try_insert_with(read) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("type")),
        }
    }

    fn properties<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltMap, E>) -> Result<(), E> {
        match self.properties.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("properties")),
        }
    }

    fn nodes<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E> {
        match self.nodes.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("nodes")),
        }
    }

    fn relations<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E> {
        match self.rels.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("relations")),
        }
    }

    fn indices<E: Error>(&mut self, read: impl FnOnce() -> Result<BoltList, E>) -> Result<(), E> {
        match self.indices.try_insert_with(read)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field("indices")),
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
}

#[derive(Debug, Copy, Clone)]
pub enum SetOnce<T> {
    Empty,
    Set(T),
}

impl<T> Default for SetOnce<T> {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetOnceError;

impl<T> SetOnce<T> {
    pub fn get_or_insert_with(&mut self, value: impl FnOnce() -> T) -> &mut T {
        match self {
            SetOnce::Empty => self.insert_with(value).unwrap(),
            SetOnce::Set(value) => value,
        }
    }

    pub fn insert_with(&mut self, value: impl FnOnce() -> T) -> Result<&mut T, SetOnceError> {
        match self {
            SetOnce::Empty => *self = Self::Set(value()),
            SetOnce::Set(_) => return Err(SetOnceError),
        };
        match self {
            SetOnce::Empty => unreachable!("value was just set"),
            SetOnce::Set(value) => Ok(value),
        }
    }

    pub fn try_insert_with<E>(
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

    pub fn take(&mut self) -> Option<T> {
        match self {
            SetOnce::Empty => None,
            SetOnce::Set(_) => {
                let value = match std::mem::take(self) {
                    Self::Set(value) => value,
                    Self::Empty => unreachable!("value is set"),
                };
                Some(value)
            }
        }
    }

    pub fn ok_or_else<E>(self, missing: impl FnOnce() -> E) -> Result<T, E> {
        match self {
            SetOnce::Empty => Err(missing()),
            SetOnce::Set(value) => Ok(value),
        }
    }

    pub fn or_else(self, missing: impl FnOnce() -> T) -> T {
        match self {
            SetOnce::Empty => missing(),
            SetOnce::Set(value) => value,
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }
}
