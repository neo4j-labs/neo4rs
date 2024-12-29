mod connection_registry;
mod load_balancing;
mod routed_connection_manager;
use std::fmt::{Display, Formatter};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {
    crate::connection::Routing,
    serde::{Deserialize, Serialize},
};

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route<'a> {
    pub(crate) routing: Routing,
    pub(crate) bookmarks: Vec<&'a str>,
    pub(crate) db: Option<Database>,
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

impl Display for Route<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ROUTE {{ {} }} [{}] {}",
            self.routing, self.bookmarks.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(", "), self.db.clone().map(|d| d.to_string()).unwrap_or("null".to_string())
        )
    }
}

impl Display for RoutingTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RoutingTable {{ ttl: {}, db: {:?}, servers: {} }}",
            self.ttl,
            self.db.clone(),
            self.servers
                .iter()
                .map(|s| s.addresses.join(", "))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

use crate::{Database, Version};
pub use load_balancing::round_robin_strategy::RoundRobinStrategy;
pub use routed_connection_manager::RoutedConnectionManager;
