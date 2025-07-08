use crate::connection::NeoUrl;
use crate::pool::{create_pool, ConnectionPool};
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::{RoutingTable, Server};
use crate::{Config, Database, Error};
use dashmap::DashMap;
use log::debug;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
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
                bs
            })
            .collect()
    }

    pub fn has_same_address(&self, other: &Self) -> bool {
        self.address == other.address && self.port == other.port
    }
}

/// A registry of connection pools, indexed by the Bolt server they connect to.
pub type PoolRegistry = DashMap<BoltServer, ConnectionPool>;
/// A map of registries, indexed by the database name.
pub type DatabaseServerMap = DashMap<String, Vec<BoltServer>>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    /// A map of connection registries, where each registry corresponds to a specific database.
    databases: DatabaseServerMap,
    pool_registry: PoolRegistry,
    default_db_name: Arc<RwLock<Option<String>>>,
}

#[allow(dead_code)]
pub(crate) enum RegistryCommand {
    RefreshSingleTable((Option<Database>, Vec<String>)),
    Stop,
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        ConnectionRegistry {
            databases: DatabaseServerMap::new(),
            pool_registry: PoolRegistry::new(),
            default_db_name: Arc::new(RwLock::new(Some(String::new()))),
        }
    }
}

async fn refresh_all_routing_tables(
    config: Config,
    connection_registry: Arc<ConnectionRegistry>,
    provider: Arc<dyn RoutingTableProvider>,
    bookmarks: &[String],
) -> Result<u64, Error> {
    debug!("Routing tables are expired, refreshing...");
    let mut ttls = vec![];

    if connection_registry.databases.is_empty() {
        debug!("No databases in the registry, initializing with default database");
        if let Some(db) = config.db.clone() {
            connection_registry
                .databases
                .insert(db.as_ref().to_string(), Vec::new());
        } else {
            let routing_table = refresh_routing_table(
                &config,
                &connection_registry.pool_registry,
                provider.clone(),
                bookmarks,
                None,
            )
            .await?;
            let default_db_name = routing_table.db.clone().unwrap().to_string();
            let mut rw = connection_registry.default_db_name.write().unwrap();
            *rw = Some(default_db_name.clone());
            connection_registry
                .databases
                .insert(default_db_name, routing_table.resolve());
            return Ok(routing_table.ttl);
        }
    }

    let map = connection_registry.databases.clone();
    for kv in map.iter() {
        let db = kv.key();

        let routing_table = refresh_routing_table(
            &config,
            &connection_registry.pool_registry,
            provider.clone(),
            bookmarks,
            Some(Database::from(db.as_str())),
        )
        .await?;
        connection_registry
            .databases
            .insert(db.clone(), routing_table.resolve());
        ttls.push(routing_table.ttl);
    }

    if ttls.is_empty() {
        return Err(Error::RoutingTableRefreshFailed(
            "No servers available in the routing table".to_string(),
        ));
    }

    // purge the pool registry of servers that are no longer in the routing tables
    let all_servers: Vec<BoltServer> = connection_registry
        .databases
        .iter()
        .flat_map(|kv| kv.value().clone())
        .collect();
    connection_registry
        .pool_registry
        .retain(|server, _| all_servers.contains(server));

    Ok(*ttls.iter().min().unwrap())
}

async fn refresh_routing_table(
    config: &Config,
    registry: &PoolRegistry,
    provider: Arc<dyn RoutingTableProvider>,
    bookmarks: &[String],
    db: Option<Database>,
) -> Result<RoutingTable, Error> {
    let routing_table = provider
        .fetch_routing_table(config, bookmarks, db.clone())
        .await?;
    debug!(
        "Routing table for database {} refreshed: {:?} (bookmarks: {:?})",
        routing_table
            .db
            .clone()
            .map(|db| db.to_string())
            .unwrap_or_default(),
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
    debug!(
        "Registry updated for database {}. New size is {} with TTL {}s",
        db.as_ref().map_or("default".to_string(), |d| d.to_string()),
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

    // This thread is in charge of refreshing the routing table periodically
    tokio::spawn(async move {
        let mut bookmarks = vec![];
        let mut ttl = refresh_all_routing_tables(
            config_clone.clone(),
            registry.clone(),
            provider.clone(),
            bookmarks.as_slice(),
        )
        .await
        .expect("Failed to get routing table. Exiting...");
        debug!("Starting background updater with TTL: {}", ttl);
        let mut interval = tokio::time::interval(Duration::from_secs(ttl));
        let now = std::time::Instant::now();
        interval.tick().await; // first tick is immediate
        loop {
            tokio::select! {
                // Trigger periodic updates
                _ = interval.tick() => {
                    debug!("Refreshing all routing tables ({})", registry.databases.len());
                    ttl = match refresh_all_routing_tables(config_clone.clone(), registry.clone(), provider.clone(), bookmarks.as_slice()).await {
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
                        Some(RegistryCommand::RefreshSingleTable((db, new_bookmarks))) => {
                            let db_name = db.as_ref().map(|d| d.to_string()).unwrap_or_default();
                            debug!("Forcing refresh of routing table for database: {}", db_name);
                            bookmarks = new_bookmarks;
                            ttl = match refresh_routing_table(&config_clone, &registry.pool_registry, provider.clone(), bookmarks.as_slice(), db).await {
                                Ok(table) => {
                                    registry.databases.insert(db_name.clone(), table.resolve());
                                    // we don't want to lose the initial TTL synchronization: if the forced update is triggered,
                                    // we derive the TTL from the initial time. Example:
                                    // if the TTL is 60 seconds and the forced update is triggered after 10 seconds,
                                    // we want to set the TTL to 50 seconds, so that the next update will be in 50 seconds
                                    ttl - (now.elapsed().as_secs() % table.ttl)
                                }
                                Err(e) => {
                                    debug!("Failed to refresh routing table: {}", e);
                                    // we don't want to lose the initial TTL synchronization: if the forced update is triggered,
                                    // we derive the TTL from the initial time. Example:
                                    // if the TTL is 60 seconds and the forced update is triggered after 10 seconds,
                                    // we want to set the TTL to 50 seconds, so that the next update will be in 50 seconds
                                    ttl - (now.elapsed().as_secs() % ttl)
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
            // recreate interval with the new TTL or the derived one in case of a forced update
            interval = tokio::time::interval(Duration::from_secs(ttl));
            interval.tick().await;
        }
    });
    tx
}

impl ConnectionRegistry {
    /// Retrieve the pool for a specific server and database.
    pub fn get_pool(&self, server: &BoltServer) -> Option<ConnectionPool> {
        self.pool_registry
            .get(server)
            .map(|pool| pool.value().clone())
    }

    /// Mark a server as available for a specific database.
    pub fn mark_unavailable(&self, server: &BoltServer, db: Option<Database>) {
        let db_name = self.get_db_name(db);
        if self.databases.contains_key(db_name.as_str()) {
            if let Some(index) = self
                .databases
                .get(db_name.as_str())
                .and_then(|vec| vec.iter().position(|s| server.has_same_address(s)))
            {
                debug!("Marking server as available: {:?}", server);
                self.databases
                    .get_mut(db_name.as_str())
                    .unwrap()
                    .remove(index);
                self.pool_registry.remove(server);
            } else {
                debug!("Server not found in the registry: {:?}", server);
            }
        }
    }

    /// Get all available Bolt servers for a specific database or the default database if none is provided.
    pub fn servers(&self, db: Option<Database>) -> Vec<BoltServer> {
        let db_name = self.get_db_name(db);
        if self.databases.contains_key(db_name.as_str()) {
            self.databases
                .get(db_name.as_str())
                .map(|entry| entry.value().clone())
                .unwrap_or_default()
        } else {
            debug!("Creating new registry for database: {}", db_name);
            self.databases.insert(db_name.clone(), Vec::new());
            vec![]
        }
    }

    pub fn all_servers(&self) -> Vec<BoltServer> {
        self.pool_registry
            .iter()
            .map(|kv| kv.key().clone())
            .collect::<Vec<BoltServer>>()
    }

    fn get_db_name(&self, db: Option<Database>) -> String {
        db.as_ref()
            .map(|d| d.to_string())
            .unwrap_or_else(|| self.default_db_name.read().unwrap().clone().unwrap())
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
            db: Some("neo4j".into()),
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
        let registry = Arc::new(ConnectionRegistry::default());
        let ttl = refresh_all_routing_tables(
            config.clone(),
            registry.clone(),
            Arc::new(TestRoutingTableProvider::new(&[cluster_routing_table])),
            &[],
        )
        .await
        .unwrap();
        assert_eq!(ttl, 300);
        assert_eq!(
            registry.default_db_name.read().unwrap().clone().unwrap(),
            "neo4j"
        );
        assert!(registry.databases.contains_key("neo4j"));

        let servers = registry.servers(None);
        assert_eq!(servers.len(), 5);

        let strategy = RoundRobinStrategy::new(registry.clone());
        registry.mark_unavailable(BoltServer::resolve(&writers[0]).first().unwrap(), None);
        let servers = registry.servers(None);
        assert_eq!(servers.len(), 4);
        let writer = strategy.select_writer(&registry.servers(None)).unwrap();
        assert_eq!(
            format!("{}:{}", writer.address, writer.port),
            writers[1].addresses[0]
        );

        registry.mark_unavailable(BoltServer::resolve(&writers[1]).first().unwrap(), None);
        let servers = registry.servers(None);
        assert_eq!(servers.len(), 3);
        let writer = strategy.select_writer(&registry.servers(None));
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
        let cluster_routing_table_default = RoutingTable {
            ttl: 300,
            db: Some("".into()),
            servers: readers1
                .clone()
                .into_iter()
                .chain(writers1.clone())
                .chain(routers.clone())
                .collect(),
        };
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
        // get registry for db1 amd refresh routing table
        let provider = Arc::new(TestRoutingTableProvider::new(&[
            cluster_routing_table_default,
            cluster_routing_table_1,
            cluster_routing_table_2,
        ]));
        refresh_all_routing_tables(config.clone(), registry.clone(), provider.clone(), &[])
            .await
            .unwrap();

        let servers = registry.servers(None);
        assert_eq!(servers.len(), 5); // 2 readers, 2 writers, 1 router (default db)
        assert_eq!(servers.first().unwrap().address, "host1");

        let _ = registry.servers(Some("db1".into())); // ensure db1 is initialized
        let _ = registry.servers(Some("db2".into())); // ensure db2 is initialized
        let ttl = refresh_all_routing_tables(config.clone(), registry.clone(), provider.clone(), &[])
            .await
            .unwrap();

        let servers = registry.servers(Some("db1".into()));
        assert_eq!(ttl, 200); // must be the min of both
        assert_eq!(servers.len(), 5); // 2 readers, 2 writers, 1 router
        assert_eq!(servers.first().unwrap().address, "host1");

        let servers2 = registry.servers(Some("db2".into()));
        assert_eq!(servers2.len(), 5); // 2 readers, 2 writers, 1 router
        assert_eq!(servers2.first().unwrap().address, "host5");

        let strategy = RoundRobinStrategy::new(registry.clone());
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
            writers2[1].addresses[0]
        );
    }
}
