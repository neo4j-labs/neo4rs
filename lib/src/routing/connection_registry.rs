use crate::connection::NeoUrl;
use crate::pool::{create_pool, ConnectionPool};
use crate::routing::{RoutingTable, Server};
use crate::{Config, Error};
use dashmap::DashMap;
use futures::lock::Mutex;
use log::debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
                    .unwrap_or_else(|_| panic!("Failed to parse address {}", address));
                debug!("Resolved server: {:?}", bs);
                bs
            })
            .collect()
    }
}

pub type Registry = DashMap<BoltServer, ConnectionPool>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    config: Config,
    creation_time: Arc<Mutex<u64>>,
    ttl: Arc<AtomicU64>,
    pub(crate) connections: Registry,
}

impl ConnectionRegistry {
    pub(crate) fn new(config: &Config) -> Self {
        ConnectionRegistry {
            config: config.clone(),
            creation_time: Arc::new(Mutex::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
            ttl: Arc::new(AtomicU64::new(0)),
            connections: DashMap::new(),
        }
    }

    pub(crate) async fn update_if_expired<F, R>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce() -> R,
        R: std::future::Future<Output = Result<RoutingTable, Error>>,
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        debug!("Checking if routing table is expired...");
        if let Some(mut guard) = self.creation_time.try_lock() {
            if self.connections.is_empty() || now - *guard > self.ttl.load(Ordering::Relaxed) {
                debug!("Routing table expired or empty, refreshing...");
                let routing_table = f().await?;
                debug!("Routing table refreshed: {:?}", routing_table);
                let registry = &self.connections;
                let servers = routing_table.resolve();
                let url = NeoUrl::parse(self.config.uri.as_str())?;
                // Convert neo4j scheme to bolt scheme to create connection pools
                let scheme = match url.scheme() {
                    "neo4j" => "bolt",
                    "neo4j+s" => "bolt+s",
                    "neo4j+ssc" => "bolt+ssc",
                    _ => return Err(Error::UnsupportedScheme(url.scheme().to_string())),
                };

                for server in servers.iter() {
                    if registry.contains_key(server) {
                        continue;
                    }
                    let uri = format!("{}://{}:{}", scheme, server.address, server.port);
                    debug!("Creating pool for server: {}", uri);
                    registry.insert(
                        server.clone(),
                        create_pool(&Config {
                            uri,
                            ..self.config.clone()
                        })
                        .await?,
                    );
                }
                registry.retain(|k, _| servers.contains(k));
                let _ = self
                    .ttl
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_ttl| {
                        Some(routing_table.ttl)
                    })
                    .unwrap();
                debug!(
                    "Registry updated. New size is {} with TTL {}s",
                    registry.len(),
                    routing_table.ttl
                );
                *guard = now;
            }
        } else {
            debug!("Routing table is not expired");
        }
        Ok(())
    }
    /// Retrieve the pool for a specific server.
    pub fn get_pool(&self, server: &BoltServer) -> Option<ConnectionPool> {
        self.connections.get(server).map(|entry| entry.clone())
    }

    pub fn mark_unavailable(&self, server: &BoltServer) {
        self.connections.remove(server);
    }

    pub fn servers(&self) -> Vec<BoltServer> {
        self.connections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ConnectionTLSConfig;
    use crate::routing::load_balancing::LoadBalancingStrategy;
    use crate::routing::RoundRobinStrategy;
    use crate::routing::Server;

    #[tokio::test]
    async fn test_available_servers() {
        let readers = vec![
            Server {
                addresses: vec!["host1:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["host2:7688".to_string()],
                role: "READ".to_string(),
            },
        ];
        let writers = vec![
            Server {
                addresses: vec!["host3:7687".to_string()],
                role: "WRITE".to_string(),
            },
            Server {
                addresses: vec!["host4:7688".to_string()],
                role: "WRITE".to_string(),
            },
        ];
        let routers = vec![Server {
            addresses: vec!["host0:7687".to_string()],
            role: "ROUTE".to_string(),
        }];
        let cluster_routing_table = RoutingTable {
            ttl: 0,
            db: None,
            servers: readers
                .clone()
                .into_iter()
                .chain(writers.clone())
                .chain(routers.clone())
                .collect(),
        };
        let config = Config {
            uri: "neo4j://localhost:7687".to_string(),
            user: "user".to_string(),
            password: "password".to_string(),
            max_connections: 10,
            db: Some("neo4j".into()),
            fetch_size: 0,
            tls_config: ConnectionTLSConfig::None,
        };
        let registry = ConnectionRegistry::new(&config);
        registry
            .update_if_expired(|| async { Ok(cluster_routing_table) })
            .await
            .unwrap();
        assert_eq!(registry.connections.len(), 5);
        let strategy = RoundRobinStrategy::default();
        registry.mark_unavailable(BoltServer::resolve(&writers[0]).first().unwrap());
        assert_eq!(registry.connections.len(), 4);
        let writer = strategy.select_writer(&registry.servers()).unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers[1].addresses[0]
        );

        registry.mark_unavailable(BoltServer::resolve(&writers[1]).first().unwrap());
        assert_eq!(registry.connections.len(), 3);
        let writer = strategy.select_writer(&registry.servers());
        assert!(writer.is_none());
    }
}
