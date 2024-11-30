mod connection_registry;
mod load_balancing;
mod routed_connection_manager;
use crate::types::{BoltMap, BoltString, BoltType};
use std::fmt::{Display, Formatter};
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
use {crate::messages::BoltRequest, crate::types::BoltList, neo4rs_macros::BoltStruct};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    serde::ser::SerializeMap,
    serde::{ser::SerializeStructVariant, Deserialize, Serialize},
};

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route<'a> {
    pub(crate) routing: Routing,
    pub(crate) bookmarks: Vec<&'a str>,
    pub(crate) db: Option<Database>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Routing {
    No,
    Yes(Vec<(BoltString, BoltString)>),
}

impl From<Routing> for Option<BoltMap> {
    fn from(routing: Routing) -> Self {
        match routing {
            Routing::No => None,
            Routing::Yes(routing) => Some(
                routing
                    .into_iter()
                    .map(|(k, v)| (k, BoltType::String(v)))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, BoltStruct, PartialEq)]
#[signature(0xB3, 0x66)]
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
pub struct Route {
    routing: BoltMap,
    bookmarks: BoltList,
    db: BoltString, // TODO: this can also be null. How do we represent a null string?
}

#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
impl Route {
    pub fn new(routing: BoltMap, bookmarks: Vec<&str>, db: Option<Database>) -> Self {
        Route {
            routing,
            bookmarks: BoltList::from(
                bookmarks
                    .into_iter()
                    .map(|b| BoltType::String(BoltString::new(b)))
                    .collect::<Vec<BoltType>>(),
            ),
            db: BoltString::from(db.map(|d| d.to_string()).unwrap_or("".to_string())),
        }
    }
}

// NOTE: this structure will be needed in the future when we implement the Bolt protocol v4.4
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", derive(Serialize))]
#[allow(dead_code)]
pub struct Extra<'a> {
    pub(crate) db: &'a str,
    pub(crate) imp_user: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", derive(Deserialize))]
pub struct RoutingTable {
    pub(crate) ttl: u64,
    pub(crate) db: Option<Database>,
    pub(crate) servers: Vec<Server>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", derive(Deserialize))]
pub struct Server {
    pub(crate) addresses: Vec<String>,
    pub(crate) role: String, // TODO: use an enum here
}

#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
pub struct RouteBuilder {
    routing: BoltMap,
    bookmarks: BoltList,
    db: BoltString,
}

#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
impl RouteBuilder {
    pub fn new(routing: Routing, bookmarks: Vec<&str>) -> Self {
        let map = match routing {
            Routing::No => BoltMap::default(),
            Routing::Yes(routing) => routing
                .into_iter()
                .map(|(k, v)| (k, BoltType::String(v)))
                .collect(),
        };
        RouteBuilder {
            routing: map,
            bookmarks: BoltList::from(
                bookmarks
                    .into_iter()
                    .map(|b| BoltType::String(BoltString::new(b)))
                    .collect::<Vec<BoltType>>(),
            ),
            db: BoltString::from("".to_string()),
        }
    }

    pub fn with_db(self, db: Database) -> Self {
        Self {
            db: BoltString::from(db.to_string()),
            ..self
        }
    }

    pub fn build(self, _version: Version) -> BoltRequest {
        BoltRequest::Route(Route {
            routing: self.routing,
            bookmarks: self.bookmarks,
            db: self.db,
        })
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
pub struct RouteBuilder<'a> {
    routing: Routing,
    bookmarks: Vec<&'a str>,
    db: Option<Database>,
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl<'a> RouteBuilder<'a> {
    pub fn new(routing: Routing, bookmarks: Vec<&'a str>) -> Self {
        Self {
            routing,
            bookmarks,
            db: None,
        }
    }

    pub fn with_db(self, db: Database) -> Self {
        Self {
            db: Some(db),
            ..self
        }
    }

    pub fn build(self, _version: Version) -> Route<'a> {
        Route {
            routing: self.routing,
            bookmarks: self.bookmarks,
            db: self.db,
        }
    }
}

impl Display for RoutingTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RoutingTable {{ ttl: {}, db: {:?}, servers: {} }}",
            self.ttl,
            self.db.clone().unwrap_or_default(),
            self.servers
                .iter()
                .map(|s| s.addresses.join(", "))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl Serialize for Routing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Routing::No => serializer.serialize_none(),
            Routing::Yes(routing) => {
                let mut map = serializer.serialize_map(Some(routing.len()))?;
                for (k, v) in routing {
                    map.serialize_entry(k.to_string().as_str(), v.to_string().as_str())?;
                }
                map.end()
            }
        }
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl<'a> Serialize for Route<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut structure = serializer.serialize_struct_variant("Request", 0x66, "ROUTE", 3)?;
        structure.serialize_field("routing", &self.routing)?;
        structure.serialize_field("bookmarks", &self.bookmarks)?;
        if let Some(db) = &self.db {
            structure.serialize_field("db", db.as_ref())?;
        } else {
            structure.serialize_field("db", &"")?;
        }
        structure.end()
    }
}

use crate::{Database, Version};
pub use load_balancing::round_robin_strategy::RoundRobinStrategy;
pub use routed_connection_manager::RoutedConnectionManager;
