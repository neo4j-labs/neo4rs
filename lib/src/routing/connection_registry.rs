use crate::config::ImpersonateUser;
use crate::connection::NeoUrl;
use crate::pool::{create_pool, ConnectionPool};
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::{RoutingTable, Server};
use crate::utils::ConcurrentHashMap;
use crate::{Config, Database, Error};
use log::{debug, error};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
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
}

/// Represents a table of Bolt servers for a specific database, along with the last update time and TTL.
/// This is used to manage the routing table for a specific database.
#[derive(Debug, Clone)]
struct DatabaseTable {
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
    fn is_expired(&self) -> bool {
        self.last_updated.elapsed() >= self.ttl
    }

    fn resolve(&self) -> Vec<BoltServer> {
        self.servers.clone()
    }

    fn mark_server_unavailable(&mut self, server: &BoltServer) -> bool {
        if let Some(index) = self
            .servers
            .iter()
            .position(|s| server.has_same_address(s))
        {
            self.servers.remove(index);
            true
        } else {
            debug!("Server not found in the database table: {server:?}");
            false
        }
    }
}

/// A registry of connection pools, indexed by the Bolt server they connect to.
type PoolRegistry = ConcurrentHashMap<BoltServer, ConnectionPool>;
/// A map of registries, indexed by the database name.
type DatabaseServerMap = ConcurrentHashMap<String, DatabaseTable>;

#[derive(Clone)]
pub(crate) struct ConnectionRegistry {
    config: Config,
    /// A map of connection registries, where each registry corresponds to a specific database.
    databases: DatabaseServerMap,
    pool_registry: PoolRegistry,
    provider: Arc<dyn RoutingTableProvider>,
}

#[allow(dead_code)]
pub(crate) enum RegistryCommand {
    RefreshSingleTable((Option<Database>, Vec<String>, Option<ImpersonateUser>)),
    Stop,
}

// pub(crate) fn start_background_updater(
//     config: &Config,
//     registry: Arc<ConnectionRegistry>,
//     provider: Arc<dyn RoutingTableProvider>,
// ) -> Sender<RegistryCommand> {
//     let config_clone = config.clone();
//     let (tx, mut rx) = mpsc::channel(1);
//
//     // This thread is in charge of refreshing the routing table periodically
//     tokio::spawn(async move {
//         let mut bookmarks = vec![];
//         let mut ttl = registry
//             .refresh_all_routing_tables(
//                 config_clone.clone(),
//                 registry.clone(),
//                 provider.clone(),
//                 bookmarks.as_slice(),
//             )
//             .await
//             .expect("Failed to get routing table. Exiting...");
//         debug!("Starting background updater with TTL: {ttl}");
//         let mut interval = tokio::time::interval(Duration::from_secs(ttl));
//         let now = std::time::Instant::now();
//         interval.tick().await; // first tick is immediate
//         loop {
//             tokio::select! {
//                 // Trigger periodic updates
//                 _ = interval.tick() => {
//                     debug!("Refreshing all routing tables ({})", registry.databases.len());
//                     ttl = match refresh_all_routing_tables(config_clone.clone(), registry.clone(), provider.clone(), bookmarks.as_slice()).await {
//                         Ok(ttl) => ttl,
//                         Err(e) => {
//                             debug!("Failed to refresh routing table: {e}");
//                             ttl
//                         }
//                     };
//                 }
//                 // Handle forced updates
//                 cmd = rx.recv() => {
//                     match cmd {
//                         Some(RegistryCommand::RefreshSingleTable((db, new_bookmarks, imp_user))) => {
//                             let db_name = db.as_ref().map(|d| d.to_string()).unwrap_or_default();
//                             debug!("Forcing refresh of routing table for database: {db_name}");
//                             bookmarks = new_bookmarks;
//                             ttl = match refresh_routing_table(&config_clone, &registry.pool_registry, provider.clone(), bookmarks.as_slice(), db, imp_user).await {
//                                 Ok(table) => {
//                                     registry.databases.upsert_sync(db_name.clone(), table.resolve());
//                                     // we don't want to lose the initial TTL synchronization: if the forced update is triggered,
//                                     // we derive the TTL from the initial time. Example:
//                                     // if the TTL is 60 seconds and the forced update is triggered after 10 seconds,
//                                     // we want to set the TTL to 50 seconds, so that the next update will be in 50 seconds
//                                     ttl - (now.elapsed().as_secs() % table.ttl)
//                                 }
//                                 Err(e) => {
//                                     error!("Failed to refresh routing table: {e}");
//                                     // we don't want to lose the initial TTL synchronization: if the forced update is triggered,
//                                     // we derive the TTL from the initial time. Example:
//                                     // if the TTL is 60 seconds and the forced update is triggered after 10 seconds,
//                                     // we want to set the TTL to 50 seconds, so that the next update will be in 50 seconds
//                                     ttl - (now.elapsed().as_secs() % ttl)
//                                 }
//                             };
//                         }
//                         Some(RegistryCommand::Stop) | None => {
//                             debug!("Stopping background updater");
//                             break;
//                         }
//                     }
//                 }
//             }
//
//             debug!("Resetting interval with TTL: {ttl}");
//             // recreate interval with the new TTL or the derived one in case of a forced update
//             interval = tokio::time::interval(Duration::from_secs(ttl));
//             interval.tick().await;
//         }
//     });
//     tx
// }

impl ConnectionRegistry {

    pub fn new(
        config: &Config,
        provider: Arc<dyn RoutingTableProvider>,
    ) -> Self {
        ConnectionRegistry {
            config: config.clone(),
            databases: ConcurrentHashMap::new(),
            pool_registry: ConcurrentHashMap::new(),
            provider,
        }
    }

    /// Retrieve the pool for a specific server and database.
    pub(crate) fn get_server_pool(&self, server: &BoltServer) -> Option<ConnectionPool> {
        self.pool_registry.get(server).map(|pool| pool.clone())
    }

    /// Mark a server as available for a specific database.
    pub(crate) fn mark_unavailable(&self, server: &BoltServer, db: Option<Database>) {
        let db_name = db.map_or(String::new(), |d| d.to_string());
        if self.databases.contains_key(&db_name) {
            debug!("Marking server as available: {server:?}");
            let mut table = self.databases.get(&db_name).unwrap();
            if table.mark_server_unavailable(server) {
                self.pool_registry.remove(server);
            } else {
                debug!("Server not found in the registry: {server:?}");
            }
        }
    }

    /// Get all available Bolt servers for a specific database or the default database if none is provided.
    pub async fn servers(&self, db: Option<Database>, imp_user: Option<ImpersonateUser>, bookmarks: &[String]) -> Vec<BoltServer> {
        if let Some(db_name) = db.as_ref().map(|d| d.to_string()) {
            if let Some(table) = self.databases.get(&db_name) {
                if table.is_expired() {
                    debug!("Routing table for database {db_name} is expired");
                    match self.fetch_routing_table(db.clone(), imp_user.clone(), bookmarks).await {
                        Ok(new_table) => {
                            let database_table: DatabaseTable = new_table.into();
                            let servers = database_table.resolve();
                            debug!("Routing table for database {db_name} refreshed");
                            servers
                        }
                        Err(e) => {
                            error!("Failed to refresh routing table for database {}: {}", db_name, e);
                            vec![] // ??
                        }
                    }
                } else {
                    table.resolve()
                }
            } else {
                match self.fetch_routing_table(db.clone(), imp_user.clone(), bookmarks).await {
                    Ok(new_table) => {
                        let database_table: DatabaseTable = new_table.into();
                        let servers = database_table.resolve();
                        debug!("Routing table for database {} refreshed", db_name);
                        servers
                    }
                    Err(e) => {
                        error!("Failed to refresh routing table for database {}: {}", db_name, e);
                        vec![] // ??
                    }
                }
            }
        } else {
            match self.fetch_routing_table(db.clone(), imp_user.clone(), bookmarks).await {
                Ok(new_table) => {
                    let db = new_table.db.clone();
                    let database_table: DatabaseTable = new_table.into();
                    let servers = database_table.resolve();
                    let db_name = db.map_or(String::new(), |d| d.to_string());
                    debug!("Routing table for database {db_name} refreshed");
                    servers
                }
                Err(e) => {
                    error!("Failed to refresh routing table for database default: {}", e);
                    vec![] // ??
                }
            }
        }
    }

    pub fn all_servers(&self) -> Vec<BoltServer> {
        self.pool_registry.keys()
    }

    pub fn update(&self, config: &Config, routing_table: &RoutingTable) -> Result<(), Error> {
        let servers = routing_table.resolve();
        let url = NeoUrl::parse(config.uri.as_str())?;

        // Convert neo4j scheme to bolt scheme to create connection pools.
        // We need to use the bolt scheme since we don't want new connections to be routed.
        let scheme = match url.scheme() {
            "neo4j" => "bolt",
            "neo4j+s" => "bolt+s",
            "neo4j+ssc" => "bolt+ssc",
            _ => panic!("Unsupported URL scheme: {}", url.scheme()),
        };

        for server in servers.iter() {
            if self.pool_registry.contains_key(server) {
                debug!("Server already exists in the registry: {server:?}");
                continue;
            }
            let uri = format!("{scheme}://{}:{}", server.address, server.port);
            debug!("Creating pool for server: {uri}");
            self.pool_registry.insert(
                server.clone(),
                create_pool(&Config {
                    uri,
                    ..config.clone()
                })?,
            );
        }
        let db_name= routing_table
            .db
            .as_ref()
            .map_or("".to_string(), |d| d.to_string());
        debug!(
            "Registry updated for database {}. New size is {} with TTL {}s",
            db_name,
            self.pool_registry.len(),
            routing_table.ttl
        );
        let database_table: DatabaseTable = routing_table.into();
        self.databases.insert(db_name.clone(), database_table);
        Ok(())
    }

    pub async fn fetch_routing_table(
        &self,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<RoutingTable, Error> {
        let table = self
            .provider
            .fetch_routing_table(bookmarks, db, imp_user)
            .await?;
        self.update(&self.config, &table)?;
        Ok(table)
    }

    pub async fn get_default_db(&self, imp_user: Option<ImpersonateUser>, bookmarks: &[String]) -> Result<Option<Database>, Error> {
        let routing_table = self.fetch_routing_table(None, imp_user, bookmarks).await?;
        Ok(routing_table.db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ConnectionTLSConfig;
    use crate::routing::RoutingTable;
    use crate::routing::Server;
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
            _bookmarks: &[String],
            db: Option<Database>,
            _imp_user: Option<ImpersonateUser>,
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
            imp_user: None,
        };
        let registry = Arc::new(ConnectionRegistry::new(
            &config,
            Arc::new(TestRoutingTableProvider::new(&[cluster_routing_table.clone()])),
        ));
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
            imp_user: None,
        };
        let registry = Arc::new(ConnectionRegistry::new(
            &config,
            Arc::new(TestRoutingTableProvider::new(&[
                cluster_routing_table_default.clone(),
                cluster_routing_table_1.clone(),
                cluster_routing_table_2.clone(),
            ])),
        ));
        // get registry for db1 amd refresh routing table
        let provider = Arc::new(TestRoutingTableProvider::new(&[
            cluster_routing_table_default,
            cluster_routing_table_1,
            cluster_routing_table_2,
        ]));
    }
}
