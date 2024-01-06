use std::collections::HashSet;

use chrono::FixedOffset;
pub use error::DeError;
pub use kind::BoltKind;

mod builder;
mod cenum;
mod date_time;
mod de;
mod element;
mod error;
mod kind;
mod node;
mod path;
mod point;
mod rel;
mod time;
mod typ;
mod urel;

/// Newtype to extract the node id or relationship id during deserialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

/// Newtype to extract the start node id of a relationship during deserialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StartNodeId(pub u64);

/// Newtype to extract the end node id of a relationship during deserialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndNodeId(pub u64);

/// Newtype to extract the node labels during deserialization.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Labels<Coll = Vec<String>>(pub Coll);

/// Newtype to extract the relationship type during deserialization.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Type<T = String>(pub T);

/// Newtype to extract the node property keys during deserialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keys<Coll = HashSet<String>>(pub Coll);

/// Newtype to extract the timezone info of datetimes during deserialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timezone<T = String>(pub T);

/// Newtype to extract the offset info of times during deserialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offset<T = FixedOffset>(pub T);

/// Newtype to extract the nodes of a path during deserialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nodes<T>(pub Vec<T>);

/// Newtype to extract the relationships of a path during deserialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relationships<T>(pub Vec<T>);

/// Newtype to extract the indices of a path during deserialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indices<T = i64>(pub Vec<T>);
