use crate::config::ImpersonateUser;
use crate::pool::ManagedConnection;
use crate::routing::connection_registry::ConnectionRegistry;
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::types::BoltServer;
use crate::routing::RoundRobinStrategy;
use crate::Database;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{Config, Error, Operation};
use backon::ExponentialBuilder;
use futures::lock::Mutex;
use log::{debug, error};
use std::sync::Arc;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    connection_registry: Arc<ConnectionRegistry>,
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: ExponentialBuilder,
}

impl RoutedConnectionManager {
    pub fn new(config: &Config, provider: Arc<dyn RoutingTableProvider>) -> Result<Self, Error> {
        let backoff = crate::pool::backoff();
        let connection_registry = Arc::new(ConnectionRegistry::new(config, provider));
        Ok(RoutedConnectionManager {
            load_balancing_strategy: Arc::new(RoundRobinStrategy::new(connection_registry.clone())),
            bookmarks: Arc::new(Mutex::new(vec![])),
            connection_registry,
            backoff,
        })
    }

    pub(crate) async fn get(
        &self,
        operation: Option<Operation>,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<ManagedConnection, Error> {
        let op = operation.unwrap_or(Operation::Write);
        let servers = self
            .connection_registry
            .servers(db.clone(), imp_user.clone(), bookmarks)
            .await;

        loop {
            let selected_server = match op {
                Operation::Read => {
                    if let Some(server) = self.select_reader(&servers) {
                        debug!("Selected reader: {server:?}");
                        server
                    } else {
                        error!("No available readers in the routing table");
                        return Err(Error::ServerUnavailableError(format!(
                            "No available writers in the routing table for operation {op}"
                        )));
                    }
                }
                Operation::Write => {
                    if let Some(server) = self.select_writer(&servers) {
                        debug!("Selected writer: {server:?}");
                        server
                    } else {
                        error!("No available writers in the routing table");
                        return Err(Error::ServerUnavailableError(format!(
                            "No available writers in the routing table for operation {op}"
                        )));
                    }
                }
            };

            if let Some(pool) = self.connection_registry.get_server_pool(&selected_server) {
                debug!("Pool status {:?}", pool.status());
                match pool.get().await {
                    Ok(conn) => return Ok(conn),
                    Err(e) => {
                        error!("Failed to get connection from pool for server {selected_server:?}: {e}");
                        self.connection_registry
                            .mark_unavailable(&selected_server, db.clone());
                        continue; // Try selecting another server
                    }
                }
            } else {
                error!("No connection pool found for server: {selected_server:?}");
                return Err(Error::ServerUnavailableError(format!(
                    "No connection pool found for server: {selected_server:?}",
                )));
            }
        }
    }

    pub async fn get_default_db(
        &self,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<Option<Database>, Error> {
        self.connection_registry
            .get_default_db(imp_user, bookmarks)
            .await
    }

    pub(crate) fn backoff(&self) -> ExponentialBuilder {
        self.backoff
    }

    fn select_reader(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        self.load_balancing_strategy.select_reader(servers)
    }

    fn select_writer(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        self.load_balancing_strategy.select_writer(servers)
    }

    #[allow(dead_code)]
    pub(crate) async fn add_bookmark(&self, bookmark: &str) {
        let mut guard = self.bookmarks.lock().await;
        if !guard.contains(&bookmark.to_string()) {
            guard.push(bookmark.to_string());
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn clear_bookmarks(&self) {
        self.bookmarks.lock().await.clear();
    }
}
