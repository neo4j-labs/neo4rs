use crate::connection::NeoUrl;
use crate::pool::{create_pool, ConnectionPool};
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::{RoutingTable, Server};
use crate::{Config, Database, Error};
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use log::debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};

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
/// A map of registries, indexed by the database name.
pub type RegistryMap = DashMap<String, Registry>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    /// The default registry for the default database key.
    pub(crate) default_registry: Registry,
    /// A map of connection registries, where each registry corresponds to a specific database.
    pub(crate) connection_map: RegistryMap,
}

#[allow(dead_code)]
pub(crate) enum RegistryCommand {
    Refresh(Vec<String>),
    Stop,
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        ConnectionRegistry {
            default_registry: Registry::new(),
            connection_map: RegistryMap::new(),
        }
    }
}

async fn refresh_routing_tables(
    config: Config,
    connection_registry: Arc<ConnectionRegistry>,
    provider: Arc<dyn RoutingTableProvider>,
    bookmarks: &[String],
) -> Result<u64, Error> {
    debug!("Routing tables are expired or empty, refreshing...");
    let mut ttls = vec![];

    // Refresh the routing table for the default database and all other databases in the registry.
    let default_routing_table = refresh_routing_table(
        &config,
        provider.clone(),
        &connection_registry.default_registry,
        bookmarks,
        None,
    )
    .await?;
    ttls.push(default_routing_table.ttl);

    for kv in connection_registry.connection_map.iter() {
        let db = kv.key();
        let registry: &Registry = kv.value();

        let routing_table = refresh_routing_table(
            &config,
            provider.clone(),
            registry,
            bookmarks,
            Some(Database::from(db.as_str())),
        )
        .await?;
        ttls.push(routing_table.ttl)
    }

    if ttls.is_empty() {
        return Err(Error::RoutingTableRefreshFailed(
            "No servers available in the routing table".to_string(),
        ));
    }
    Ok(*ttls.iter().min().unwrap())
}

async fn refresh_routing_table(
    config: &Config,
    provider: Arc<dyn RoutingTableProvider>,
    registry: &Registry,
    bookmarks: &[String],
    db: Option<Database>,
) -> Result<RoutingTable, Error> {
    let routing_table = provider
        .fetch_routing_table(config, bookmarks, db.clone())
        .await?;
    debug!(
        "Routing table for database {} refreshed: {:?} (bookmarks: {:?})",
        db.as_ref().map(|d| d.as_ref()).unwrap_or("(null)"),
        routing_table,
        bookmarks
    );
    let servers = routing_table.resolve();
    let url = NeoUrl::parse(config.uri.as_str())?;

    for server in servers.iter() {
        if registry.contains_key(server) {
            continue;
        }
        let uri = format!("{}://{}:{}", url.scheme(), server.address, server.port);
        debug!("Creating pool for server: {}", uri);
        registry.insert(
            server.clone(),
            create_pool(&Config {
                uri,
                ..config.clone()
            })?,
        );
    }
    registry.retain(|k, _| servers.contains(k));
    debug!(
        "Registry updated. New size is {} with TTL {}s",
        registry.len(),
        routing_table.ttl
    );
    Ok(routing_table)
}

pub(crate) fn start_background_updater(
    config: &Config,
    registry: Arc<ConnectionRegistry>,
    provider: Arc<dyn RoutingTableProvider>,
) -> Sender<RegistryCommand> {
    let config_clone = config.clone();
    let (tx, mut rx) = mpsc::channel(1);
    let bookmarks = Mutex::new(vec![]);
    if let Some(db) = config.db.clone() {
        registry
            .connection_map
            .insert(db.as_ref().to_string(), Registry::new());
    }
    // This thread is in charge of refreshing the routing table periodically
    tokio::spawn(async move {
        let mut ttl = refresh_routing_tables(
            config_clone.clone(),
            registry.clone(),
            provider.clone(),
            bookmarks.lock().await.as_slice(),
        )
        .await
        .expect("Failed to get routing table. Exiting...");
        debug!("Starting background updater with TTL: {}", ttl);
        let mut interval = tokio::time::interval(Duration::from_secs(ttl));
        interval.tick().await; // first tick is immediate
        loop {
            tokio::select! {
                // Trigger periodic updates
                _ = interval.tick() => {
                    ttl = match refresh_routing_tables(config_clone.clone(), registry.clone(), provider.clone(), bookmarks.lock().await.as_slice()).await {
                        Ok(ttl) => ttl,
                        Err(e) => {
                            debug!("Failed to refresh routing table: {}", e);
                            ttl
                        }
                    };
                    interval = tokio::time::interval(Duration::from_secs(ttl)); // recreate interval with the new TTL
                }
                // Handle forced updates
                cmd = rx.recv() => {
                    match cmd {
                        Some(RegistryCommand::Refresh(new_bookmarks)) => {
                            *bookmarks.lock().await = new_bookmarks;
                            ttl = match refresh_routing_tables(config_clone.clone(), registry.clone(), provider.clone(), bookmarks.lock().await.as_slice()).await {
                                Ok(ttl) => ttl,
                                Err(e) => {
                                    debug!("Failed to refresh routing table: {}", e);
                                    ttl
                                }
                            };
                            interval = tokio::time::interval(Duration::from_secs(ttl)); // recreate interval with the new TTL
                        }
                        Some(RegistryCommand::Stop) | None => {
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
    /// Retrieve the pool for a specific server and database.
    pub fn get_pool(&self, server: &BoltServer, db: Option<Database>) -> Option<ConnectionPool> {
        if db.is_some() {
            let pair = self.get_or_create_registry(db.unwrap());
            pair.get(server).map(|entry| entry.clone())
        } else {
            self.default_registry.get(server).map(|entry| entry.clone())
        }
    }

    /// Mark a server as available for a specific database.
    pub fn mark_unavailable(&self, server: &BoltServer, db: Option<Database>) {
        if let Some(database) = db.as_ref() {
            if let Some(registry) = self.connection_map.get(&database.to_string()) {
                registry.remove(server);
            }
        } else {
            self.default_registry.remove(server);
        }
    }

    /// Get all available Bolt servers for a specific database or the default database if none is provided.
    pub fn servers(&self, db: Option<Database>) -> Vec<BoltServer> {
        if let Some(database) = db.as_ref() {
            if let Some(registry) = self.connection_map.get(&database.to_string()) {
                registry.iter().map(|entry| entry.key().clone()).collect()
            } else {
                vec![]
            }
        } else {
            self.default_registry
                .iter()
                .map(|entry| entry.key().clone())
                .collect()
        }
    }

    /// Get or create a registry for a specific database.
    /// Panics if the database name is not found in the connection map.
    pub(crate) fn get_or_create_registry(&self, db: Database) -> Ref<String, Registry> {
        let db_name = db.as_ref().to_string();
        if !self.connection_map.contains_key(db_name.as_str()) {
            self.connection_map.insert(db_name.clone(), Registry::new());
        }
        let registry = self.connection_map.get(db_name.as_str()).unwrap();
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ConnectionTLSConfig;
    use crate::routing::load_balancing::LoadBalancingStrategy;
    use crate::routing::Server;
    use crate::routing::{RoundRobinStrategy, RoutingTable};
    use std::future::Future;
    use std::pin::Pin;

    struct TestRoutingTableProvider {
        routing_tables: Vec<RoutingTable>,
    }

    impl TestRoutingTableProvider {
        fn new(routing_tables: &[RoutingTable]) -> Self {
            TestRoutingTableProvider {
                routing_tables: routing_tables.to_vec(),
            }
        }
    }

    impl RoutingTableProvider for TestRoutingTableProvider {
        fn fetch_routing_table(
            &self,
            _: &Config,
            _bookmarks: &[String],
            db: Option<Database>,
        ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>> {
            let vec = self.routing_tables.clone();
            if let Some(db) = db {
                if let Some(table) = vec.iter().find(|t| t.db.as_ref() == Some(&db)) {
                    let t = table.clone();
                    return Box::pin(async move { Ok(t) });
                }
            }
            Box::pin(async move {
                if let Some(table) = vec.first() {
                    Ok(table.clone())
                } else {
                    Err(Error::RoutingTableRefreshFailed(
                        "No routing table available".to_string(),
                    ))
                }
            })
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
            db: None,
            fetch_size: 200,
            tls_config: ConnectionTLSConfig::None,
        };
        let con_registry = Arc::new(ConnectionRegistry::default());
        let registry = &con_registry.default_registry;
        let ttl = refresh_routing_tables(
            config.clone(),
            con_registry.clone(),
            Arc::new(TestRoutingTableProvider::new(&[cluster_routing_table])),
            &[],
        )
        .await
        .unwrap();
        assert_eq!(ttl, 300);
        assert_eq!(registry.len(), 5);
        let strategy = RoundRobinStrategy::default();
        con_registry.mark_unavailable(BoltServer::resolve(&writers[0]).first().unwrap(), None);
        assert_eq!(registry.len(), 4);
        let writer = strategy.select_writer(&con_registry.servers(None)).unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers[1].addresses[0]
        );

        con_registry.mark_unavailable(BoltServer::resolve(&writers[1]).first().unwrap(), None);
        assert_eq!(registry.len(), 3);
        let writer = strategy.select_writer(&con_registry.servers(None));
        assert!(writer.is_none());
    }

    #[tokio::test]
    async fn test_available_servers_multi_db() {
        let readers1 = vec![
            Server {
                addresses: vec!["host1:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["host2:7688".to_string()],
                role: "READ".to_string(),
            },
        ];
        let writers1 = vec![
            Server {
                addresses: vec!["host3:7687".to_string()],
                role: "WRITE".to_string(),
            },
            Server {
                addresses: vec!["host4:7688".to_string()],
                role: "WRITE".to_string(),
            },
        ];
        let readers2 = vec![
            Server {
                addresses: vec!["host5:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["host6:7688".to_string()],
                role: "READ".to_string(),
            },
        ];
        let writers2 = vec![
            Server {
                addresses: vec!["host7:7687".to_string()],
                role: "WRITE".to_string(),
            },
            Server {
                addresses: vec!["host8:7688".to_string()],
                role: "WRITE".to_string(),
            },
        ];
        let routers = vec![Server {
            addresses: vec!["host0:7687".to_string()],
            role: "ROUTE".to_string(),
        }];
        let cluster_routing_table_1 = RoutingTable {
            ttl: 300,
            db: Some("db1".into()),
            servers: readers1
                .clone()
                .into_iter()
                .chain(writers1.clone())
                .chain(routers.clone())
                .collect(),
        };
        let cluster_routing_table_2 = RoutingTable {
            ttl: 200,
            db: Some("db2".into()),
            servers: readers2
                .clone()
                .into_iter()
                .chain(writers2.clone())
                .chain(routers.clone())
                .collect(),
        };
        let config = Config {
            uri: "neo4j://localhost:7687".to_string(),
            user: "user".to_string(),
            password: "password".to_string(),
            max_connections: 10,
            db: None,
            fetch_size: 200,
            tls_config: ConnectionTLSConfig::None,
        };
        let con_registry = Arc::new(ConnectionRegistry::default());
        // get registry for db1 amd refresh routing table
        let registry = con_registry.get_or_create_registry("db1".into());
        let provider = Arc::new(TestRoutingTableProvider::new(&[
            cluster_routing_table_1,
            cluster_routing_table_2,
        ]));
        let ttl =
            refresh_routing_tables(config.clone(), con_registry.clone(), provider.clone(), &[])
                .await
                .unwrap();
        assert_eq!(ttl, 300);
        assert_eq!(registry.len(), 5); // 2 readers, 2 writers, 1 router

        // get registry for db1 amd refresh routing table
        let registry2 = con_registry.get_or_create_registry("db2".into());
        let ttl =
            refresh_routing_tables(config.clone(), con_registry.clone(), provider.clone(), &[])
                .await
                .unwrap();
        assert_eq!(ttl, 200); // must be the min of both
        assert_eq!(registry2.len(), 5); // 2 readers, 2 writers, 1 router

        let strategy = RoundRobinStrategy::default();
        let writer = strategy
            .select_writer(&con_registry.servers(Some("db1".into())))
            .unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers1[1].addresses[0]
        );

        let writer = strategy
            .select_writer(&con_registry.servers(Some("db2".into())))
            .unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers2[1].addresses[0]
        );
    }
}
