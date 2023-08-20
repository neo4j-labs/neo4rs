use crate::types::{BoltMap, BoltNode, BoltRelation, BoltType, BoltUnboundedRelation};

use std::{collections::HashSet, result::Result};

use serde::{
    de::{value::MapDeserializer, IntoDeserializer},
    Deserialize,
};

pub use error::{DeError, Unexpected};
pub use kind::BoltKind;

mod cenum;
mod de;
mod deser;
mod element;
mod error;
mod kind;

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

impl BoltType {
    pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
    where
        T: Deserialize<'this>,
    {
        T::deserialize(self.into_deserializer())
    }
}
