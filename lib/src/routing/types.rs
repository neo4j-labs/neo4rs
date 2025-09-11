use crate::connection::NeoUrl;
use crate::pool::ConnectionPool;
use crate::routing::{RoutingTable, Server};
use crate::utils::ConcurrentHashMap;
use std::hash::Hash;
use std::time::Duration;

/// Represents a Bolt server, with its address, port and role.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BoltServer {
    pub(crate) address: String,
    pub(crate) port: u16,
    pub(crate) role: String,
}

impl BoltServer {
    pub(crate) fn resolve(server: &Server) -> Vec<Self> {
        server
            .addresses
            .iter()
            .map(|address| {
                let bs = NeoUrl::parse(address)
                    .map(|addr| BoltServer {
                        address: addr.host().to_string(),
                        port: addr.port(),
                        role: server.role.to_string(),
                    })
                    .unwrap_or_else(|_| panic!("Failed to parse address {address}"));
                bs
            })
            .collect()
    }

    pub fn has_same_address(&self, other: &Self) -> bool {
        self.address == other.address && self.port == other.port
    }

    pub(crate) fn to_neo_url(&self, scheme: &str) -> NeoUrl {
        NeoUrl::parse(&format!("{scheme}://{}:{}", self.address, self.port))
            .expect("Failed to parse BoltServer to NeoUrl")
    }
}

/// Represents a table of Bolt servers for a specific database, along with the last update time and TTL.
/// This is used to manage the routing table for a specific database.
#[derive(Debug, Clone)]
pub(crate) struct DatabaseTable {
    servers: Vec<BoltServer>,
    last_updated: std::time::Instant,
    ttl: Duration,
}

impl Default for DatabaseTable {
    fn default() -> Self {
        DatabaseTable {
            servers: Vec::new(),
            last_updated: std::time::Instant::now(),
            ttl: Duration::from_secs(0),
        }
    }
}

impl From<RoutingTable> for DatabaseTable {
    fn from(table: RoutingTable) -> Self {
        Self::from(&table)
    }
}

impl From<&RoutingTable> for DatabaseTable {
    fn from(table: &RoutingTable) -> Self {
        DatabaseTable {
            servers: table.resolve(),
            last_updated: std::time::Instant::now(),
            ttl: Duration::from_secs(table.ttl),
        }
    }
}

impl DatabaseTable {
    pub(crate) fn is_expired(&self) -> bool {
        self.last_updated.elapsed() >= self.ttl
    }

    pub(crate) fn resolve(&self) -> Vec<BoltServer> {
        self.servers.clone()
    }

    pub(crate) fn mark_server_unavailable(&mut self, server: &BoltServer) -> bool {
        let size = self.servers.len();
        self.servers.retain(|s| !server.has_same_address(s));
        size != self.servers.len()
    }
}

/// A registry of connection pools, indexed by the Bolt server they connect to.
pub(crate) type PoolRegistry = ConcurrentHashMap<NeoUrl, ConnectionPool>;
/// A map of registries, indexed by the database name.
pub(crate) type DatabaseServerMap = ConcurrentHashMap<String, DatabaseTable>;
