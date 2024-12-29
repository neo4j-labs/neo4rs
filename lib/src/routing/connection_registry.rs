use crate::pool::{create_pool, ConnectionPool};
use crate::routing::{RoutingTable, Server};
use crate::{Config, Error};
use dashmap::DashMap;
use futures::lock::Mutex;
use log::info;
use std::sync::Arc;

pub type Registry = DashMap<Server, ConnectionPool>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    config: Config,
    creation_time: Arc<Mutex<u64>>,
    ttl: u64,
    pub(crate) connections: Registry,
    servers: Vec<Server>,
    readers: Vec<Server>,
    writers: Vec<Server>,
    routers: Vec<Server>,
}

impl ConnectionRegistry {
    pub(crate) async fn new(
        config: &Config,
        routing_table: Arc<RoutingTable>,
    ) -> Result<Self, Error> {
        let ttl = routing_table.ttl;
        let readers = routing_table
            .servers
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect();
        let writers = routing_table
            .servers
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect();
        let routers = routing_table
            .servers
            .iter()
            .filter(|s| s.role == "ROUTE")
            .cloned()
            .collect();
        let connections = Self::build_registry(config, &routing_table.servers).await?;
        Ok(ConnectionRegistry {
            config: config.clone(),
            creation_time: Arc::new(Mutex::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
            ttl,
            connections,
            servers: routing_table.servers.clone(),
            readers,
            writers,
            routers,
        })
    }

    async fn build_registry(
        config: &Config,
        servers: &Vec<Server>,
    ) -> Result<Registry, Error> {
        let registry = DashMap::new();
        for server in servers.iter() {
            registry.insert(server.clone(), create_pool(config).await?);
        }
        Ok(registry)
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
        info!("Checking if routing table is expired...");
        if let Some(mut guard) = self.creation_time.try_lock() {
            if now - *guard > self.ttl {
                info!("Routing table expired, refreshing...");
                let routing_table = f().await?;
                info!("Routing table refreshed: {:?}", routing_table);
                let registry = &self.connections;
                let servers = routing_table.servers.clone();
                for server in servers.iter() {
                    if registry.contains_key(server) {
                        continue;
                    }
                    registry.insert(server.clone(), create_pool(&self.config).await?);
                }
                registry.retain(|k, _| servers.contains(k));
                info!("Registry updated. New size is {}", registry.len());
                *guard = now;
            }
        }
        Ok(())
    }
    /// Retrieve the pool for a specific server.
    pub fn get_pool(&self, server: &Server) -> Option<ConnectionPool> {
        self.connections.get(server).map(|entry| entry.clone())
    }

    pub fn mark_unavailable(&self, server: &Server) {
        self.connections.remove(server);
    }

    #[allow(dead_code)]
    pub fn servers(&self) -> &[Server] {
        self.servers.as_slice()
    }

    pub fn readers(&self) -> &[Server] {
        self.readers.as_slice()
    }
    
    pub fn writers(&self) -> &[Server] {
        self.writers.as_slice()
    }
    
    pub fn routers(&self) -> &[Server] {
        self.routers.as_slice()
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
        let registry = ConnectionRegistry::new(&config, Arc::new(cluster_routing_table.clone()))
            .await
            .unwrap();
        assert_eq!(registry.connections.len(), 5);
        let strategy = RoundRobinStrategy::new(cluster_routing_table.clone());
        let router = strategy
            .select_router(registry.routers())
            .unwrap();
        assert_eq!(router, routers[0]);
        registry.mark_unavailable(&writers[0]);
        assert_eq!(registry.connections.len(), 4);
        let writer = strategy
            .select_writer(registry.writers())
            .unwrap();
        assert_eq!(writer, writers[1]);
    }
}
