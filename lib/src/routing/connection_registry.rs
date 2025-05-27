use crate::connection::NeoUrl;
use crate::pool::{create_pool, ConnectionPool};
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::{RoutingTable, Server};
use crate::{Config, Database, Error};
use dashmap::DashMap;
use log::debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

/// Represents a Bolt server, with its address, port and role.
#[derive(Debug, Clone)]
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
                bs
            })
            .collect()
    }
}

impl Eq for BoltServer {}

impl PartialEq for BoltServer {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.port == other.port
    }
}

impl std::hash::Hash for BoltServer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.port.hash(state);
    }
}

/// A registry of connection pools, indexed by the Bolt server they connect to.
pub type Registry = DashMap<BoltServer, ConnectionPool>;
/// A map of registries, indexed by the database name.
pub type ServerMap = DashMap<String, Vec<BoltServer>>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    /// A map of connection registries, where each registry corresponds to a specific database.
    connection_map: ServerMap,
    registry: Registry,
}

#[allow(dead_code)]
pub(crate) enum RegistryCommand {
    RefreshAll(Vec<String>),
    RefreshSingleTable((Option<Database>, Vec<String>)),
    Stop,
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        let connection_map = ServerMap::new();
        connection_map.insert("".to_string(), vec![]); // insert a default registry for the default database
        ConnectionRegistry {
            connection_map,
            registry: Registry::new(),
        }
    }
}

async fn refresh_routing_tables(
    config: Config,
    connection_registry: Arc<ConnectionRegistry>,
    provider: Arc<dyn RoutingTableProvider>,
    bookmarks: &[String],
) -> Result<u64, Error> {
    debug!("Routing tables are expired, refreshing...");
    let mut ttls = vec![];

    let map = connection_registry.connection_map.clone();
    for kv in map.iter() {
        let db = kv.key();

        let routing_table = refresh_routing_table(
            &config,
            &connection_registry.registry,
            provider.clone(),
            bookmarks,
            Some(Database::from(db.as_str())),
        )
        .await?;
        connection_registry
            .connection_map
            .insert(db.clone(), routing_table.resolve());
        ttls.push(routing_table.ttl);
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
    registry: &Registry,
    provider: Arc<dyn RoutingTableProvider>,
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
            debug!("Server already exists in the registry: {:?}", server);
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
    if let Some(db) = config.db.clone() {
        registry
            .connection_map
            .insert(db.as_ref().to_string(), Vec::new());
    }
    // This thread is in charge of refreshing the routing table periodically
    tokio::spawn(async move {
        let mut bookmarks = vec![];
        let mut ttl = refresh_routing_tables(
            config_clone.clone(),
            registry.clone(),
            provider.clone(),
            bookmarks.as_slice(),
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
                    ttl = match refresh_routing_tables(config_clone.clone(), registry.clone(), provider.clone(), bookmarks.as_slice()).await {
                        Ok(ttl) => ttl,
                        Err(e) => {
                            debug!("Failed to refresh routing table: {}", e);
                            ttl
                        }
                    };
                }
                // Handle forced updates
                cmd = rx.recv() => {
                    match cmd {
                        Some(RegistryCommand::RefreshAll(new_bookmarks)) => {
                            bookmarks = new_bookmarks;
                            ttl = match refresh_routing_tables(config_clone.clone(), registry.clone(), provider.clone(), bookmarks.as_slice()).await {
                                Ok(ttl) => ttl,
                                Err(e) => {
                                    debug!("Failed to refresh routing table: {}", e);
                                    ttl
                                }
                            };
                        }
                        Some(RegistryCommand::RefreshSingleTable((db, new_bookmarks))) => {
                            let db_name = db.as_ref().map(|d| d.to_string()).unwrap_or_default();
                            bookmarks = new_bookmarks;
                            ttl = match refresh_routing_table(&config_clone, &registry.registry, provider.clone(), bookmarks.as_slice(), db).await {
                                Ok(table) => {
                                    if let Some(mut registry) = registry.connection_map.get_mut(db_name.as_str()) {
                                        registry.value_mut().clear(); // clear the old entries
                                        registry.value_mut().extend(table.resolve());
                                    } else {
                                        debug!("Creating new registry for database: {}", db_name);
                                        registry.connection_map.insert(db_name.clone(), table.resolve());
                                    }
                                    debug!("Successfully refreshed routing table for new database {}", db_name);
                                    table.ttl
                                }
                                Err(e) => {
                                    debug!("Failed to refresh routing table: {}", e);
                                    ttl
                                }
                            };
                        }
                        Some(RegistryCommand::Stop) | None => {
                            debug!("Stopping background updater");
                            break;
                        }
                    }
                }
            }

            debug!("Resetting interval with TTL: {}", ttl);
            interval = tokio::time::interval(Duration::from_secs(ttl)); // recreate interval with the new TTL
            interval.tick().await;
        }
    });
    tx
}

impl ConnectionRegistry {
    /// Retrieve the pool for a specific server and database.
    pub fn get_pool(&self, server: &BoltServer, db: Option<Database>) -> Option<ConnectionPool> {
        let pair = self.servers(db.clone());
        pair.iter()
            .find(|bs| *bs == server)
            .and_then(|bs| self.registry.get(bs).map(|pool| pool.value().clone()))
    }

    /// Mark a server as available for a specific database.
    pub fn mark_unavailable(&self, server: &BoltServer, db: Option<Database>) {
        let db_name = db.as_ref().map(|d| d.to_string()).unwrap_or_default();
        if self.connection_map.contains_key(db_name.as_str()) {
            if let Some(index) = self
                .connection_map
                .get(db_name.as_str())
                .and_then(|vec| vec.iter().position(|s| s == server))
            {
                debug!("Marking server as available: {:?}", server);
                self.connection_map
                    .get_mut(db_name.as_str())
                    .unwrap()
                    .remove(index);
            } else {
                debug!("Server not found in the registry: {:?}", server);
            }
        }
    }

    /// Get all available Bolt servers for a specific database or the default database if none is provided.
    pub fn servers(&self, db: Option<Database>) -> Vec<BoltServer> {
        let db_name = db.as_ref().map(|d| d.to_string()).unwrap_or_default();
        if self.connection_map.contains_key(db_name.as_str()) {
            self.connection_map
                .get(db_name.as_str())
                .map(|entry| entry.value().clone())
                .unwrap_or_default()
        } else {
            debug!("Creating new registry for database: {}", db_name);
            self.connection_map.insert(db_name.clone(), Vec::new());
            vec![]
        }
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
        let ttl = refresh_routing_tables(
            config.clone(),
            con_registry.clone(),
            Arc::new(TestRoutingTableProvider::new(&[cluster_routing_table])),
            &[],
        )
        .await
        .unwrap();
        assert_eq!(ttl, 300);
        let registry = con_registry.servers(None);
        assert_eq!(registry.len(), 5);
        let strategy = RoundRobinStrategy::default();
        con_registry.mark_unavailable(BoltServer::resolve(&writers[0]).first().unwrap(), None);
        let registry = con_registry.servers(None);
        assert_eq!(registry.len(), 4);
        let writer = strategy.select_writer(&con_registry.servers(None)).unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers[1].addresses[0]
        );

        con_registry.mark_unavailable(BoltServer::resolve(&writers[1]).first().unwrap(), None);
        let registry = con_registry.servers(None);
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
        let registry = Arc::new(ConnectionRegistry::default());
        let _ = registry.servers(Some("db1".into())); // ensure db1 is initialized
        let _ = registry.servers(Some("db2".into())); // ensure db2 is initialized
                                                      // get registry for db1 amd refresh routing table
        let provider = Arc::new(TestRoutingTableProvider::new(&[
            cluster_routing_table_1,
            cluster_routing_table_2,
        ]));
        let ttl = refresh_routing_tables(config.clone(), registry.clone(), provider.clone(), &[])
            .await
            .unwrap();
        let servers = registry.servers(Some("db1".into()));
        assert_eq!(ttl, 200); // must be the min of both
        assert_eq!(servers.len(), 5); // 2 readers, 2 writers, 1 router

        // get registry for db1 amd refresh routing table
        let ttl = refresh_routing_tables(config.clone(), registry.clone(), provider.clone(), &[])
            .await
            .unwrap();
        assert_eq!(ttl, 200); // must be the min of both
        let servers2 = registry.servers(Some("db2".into()));
        assert_eq!(servers2.len(), 5); // 2 readers, 2 writers, 1 router

        let strategy = RoundRobinStrategy::default();
        let writer = strategy
            .select_writer(&registry.servers(Some("db1".into())))
            .unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers1[1].addresses[0]
        );

        let writer = strategy
            .select_writer(&registry.servers(Some("db2".into())))
            .unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers2[0].addresses[0]
        );
    }
}
