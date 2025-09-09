mod connection_registry;
mod load_balancing;
mod routed_connection_manager;
mod routing_table_provider;

use std::fmt::{Display, Formatter};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use {crate::connection::Routing, serde::Deserialize};

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteExtra {
    V4_3(Option<Database>),
    V4_4(Extra),
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub(crate) routing: Routing,
    pub(crate) bookmarks: Vec<String>,
    pub(crate) extra: RouteExtra,
}

// NOTE: this structure will be needed in the future when we implement the Bolt protocol v4.4
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[allow(dead_code)]
pub struct Extra {
    pub(crate) db: Option<Database>,
    pub(crate) imp_user: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", derive(Deserialize))]
pub struct RoutingTable {
    pub(crate) ttl: u64,
    pub(crate) db: Option<Database>,
    pub(crate) servers: Vec<Server>,
}

impl RoutingTable {
    pub(crate) fn resolve(&self) -> Vec<BoltServer> {
        self.servers
            .iter()
            .flat_map(BoltServer::resolve)
            .collect::<Vec<BoltServer>>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", derive(Deserialize))]
pub struct Server {
    pub(crate) addresses: Vec<String>,
    pub(crate) role: String, // TODO: use an enum here
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
pub struct RouteBuilder {
    routing: Routing,
    bookmarks: Vec<String>,
    db: Option<Database>,
    imp_user: Option<ImpersonateUser>,
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl RouteBuilder {
    pub fn new(routing: Routing, bookmarks: Vec<String>) -> Self {
        Self {
            routing,
            bookmarks,
            db: None,
            imp_user: None,
        }
    }

    pub fn with_db(self, db: Database) -> Self {
        Self {
            db: Some(db),
            ..self
        }
    }

    pub fn with_imp_user(self, imp_user: ImpersonateUser) -> Self {
        Self {
            imp_user: Some(imp_user),
            ..self
        }
    }

    pub fn build(self, version: Version) -> Route {
        match version.cmp(&Version::V4_4) {
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => Route {
                routing: self.routing,
                bookmarks: self.bookmarks,
                extra: RouteExtra::V4_4(Extra {
                    db: self.db,
                    imp_user: self.imp_user.map(|s| s.to_string()),
                }),
            },
            std::cmp::Ordering::Less => Route {
                routing: self.routing,
                bookmarks: self.bookmarks,
                extra: RouteExtra::V4_3(self.db),
            },
        }
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (db, imp_user) = match self.extra {
            RouteExtra::V4_3(ref db) => {
                let db = db
                    .clone()
                    .map(|d| d.to_string())
                    .unwrap_or("null".to_string());
                let imp_user = "null".to_string();
                (db, imp_user)
            }
            RouteExtra::V4_4(ref extra) => {
                let db = extra
                    .db
                    .clone()
                    .map(|d| d.to_string())
                    .unwrap_or("null".to_string());
                let imp_user = extra.imp_user.clone().unwrap_or("null".to_string());
                (db, imp_user)
            }
        };

        write!(
            f,
            "ROUTE {{ {} }} [{}] {} {}",
            self.routing,
            self.bookmarks
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            db,
            imp_user
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

use crate::config::ImpersonateUser;
use crate::routing::connection_registry::BoltServer;
use crate::{Database, Version};
pub use load_balancing::round_robin_strategy::RoundRobinStrategy;
pub use routed_connection_manager::RoutedConnectionManager;
pub use routing_table_provider::ClusterRoutingTableProvider;
