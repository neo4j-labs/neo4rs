use crate::connection::NeoUrl;
use crate::pool::{create_pool, ConnectionPool};
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::Server;
use crate::{Config, Error};
use dashmap::DashMap;
use log::debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

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
                    .unwrap_or_else(|_| panic!("Failed to parse address {}", address));
                debug!("Resolved server: {:?}", bs);
                bs
            })
            .collect()
    }
}

/// A registry of connection pools, indexed by the Bolt server they connect to.
pub type Registry = DashMap<BoltServer, ConnectionPool>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    pub(crate) connections: Registry,
}

#[allow(dead_code)]
pub(crate) enum RegistryCommand {
    Refresh,
    Stop,
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        ConnectionRegistry {
            connections: Registry::new(),
        }
    }
}

async fn refresh_routing_table(
    config: Config,
    registry: Arc<ConnectionRegistry>,
    provider: Arc<Box<dyn RoutingTableProvider>>,
) -> Result<u64, Error> {
    debug!("Routing table expired or empty, refreshing...");
    let routing_table = provider.fetch_routing_table(&config).await?;
    debug!("Routing table refreshed: {:?}", routing_table);
    let servers = routing_table.resolve();
    let url = NeoUrl::parse(config.uri.as_str())?;
    // Convert neo4j scheme to bolt scheme to create connection pools.
    // We need to use the bolt scheme since we don't want new connections to be routed
    let scheme = match url.scheme() {
        "neo4j" => "bolt",
        "neo4j+s" => "bolt+s",
        "neo4j+ssc" => "bolt+ssc",
        _ => panic!("Unsupported scheme: {}", url.scheme()),
    };

    for server in servers.iter() {
        if registry.connections.contains_key(server) {
            continue;
        }
        let uri = format!("{}://{}:{}", scheme, server.address, server.port);
        debug!("Creating pool for server: {}", uri);
        registry.connections.insert(
            server.clone(),
            create_pool(&Config {
                uri,
                ..config.clone()
            })
            .await?,
        );
    }
    registry.connections.retain(|k, _| servers.contains(k));
    debug!(
        "Registry updated. New size is {} with TTL {}s",
        registry.connections.len(),
        routing_table.ttl
    );
    Ok(routing_table.ttl)
}

pub(crate) async fn start_background_updater(
    config: &Config,
    registry: Arc<ConnectionRegistry>,
    provider: Arc<Box<dyn RoutingTableProvider>>,
) -> Sender<RegistryCommand> {
    let config_clone = config.clone();
    let (tx, mut rx) = mpsc::channel(1);

    // This thread is in charge of refreshing the routing table periodically
    tokio::spawn(async move {
        let mut ttl =
            refresh_routing_table(config_clone.clone(), registry.clone(), provider.clone())
                .await
                .expect("Failed to get routing table. Exiting...");
        debug!("Starting background updater with TTL: {}", ttl);
        let mut interval = tokio::time::interval(Duration::from_secs(ttl));
        interval.tick().await; // first tick is immediate
        loop {
            tokio::select! {
                // Trigger periodic updates
                _ = interval.tick() => {
                    ttl = match refresh_routing_table(config_clone.clone(), registry.clone(), provider.clone()).await {
                        Ok(ttl) => ttl,
                        Err(e) => {
                            debug!("Failed to refresh routing table: {}", e);
                            ttl
                        }
                    };
                    interval = tokio::time::interval(Duration::from_secs(ttl)); // recreate interval with the new TTL
                }
                // Handle forced updates
                Some(cmd) = rx.recv() => {
                    match cmd {
                        RegistryCommand::Refresh => {
                            ttl = match refresh_routing_table(config_clone.clone(), registry.clone(), provider.clone()).await {
                                Ok(ttl) => ttl,
                                Err(e) => {
                                    debug!("Failed to refresh routing table: {}", e);
                                    ttl
                                }
                            };
                            interval = tokio::time::interval(Duration::from_secs(ttl)); // recreate interval with the new TTL
                        }
                        RegistryCommand::Stop => {
                            debug!("Stopping background updater");
                            break;
                        }
                    }
                }
            }

            interval.tick().await;
        }
    });
    tx
}

impl ConnectionRegistry {
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
    use crate::routing::{RoundRobinStrategy, RoutingTable};
    use crate::routing::Server;
    use std::future::Future;
    use std::pin::Pin;

    struct TestRoutingTableProvider {
        routing_table: RoutingTable,
    }

    impl TestRoutingTableProvider {
        fn new(routing_table: RoutingTable) -> Self {
            TestRoutingTableProvider { routing_table }
        }
    }

    impl RoutingTableProvider for TestRoutingTableProvider {
        fn fetch_routing_table(
            &self,
            _: &Config,
        ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>> {
            let routing_table = self.routing_table.clone();
            Box::pin(async move { Ok(routing_table) })
        }
    }

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
            ttl: 300,
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
        let registry = Arc::new(ConnectionRegistry::default());
        let ttl = refresh_routing_table(
            config.clone(),
            registry.clone(),
            Arc::new(Box::new(TestRoutingTableProvider::new(
                cluster_routing_table,
            ))),
        )
        .await
        .unwrap();
        assert_eq!(ttl, 300);
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
