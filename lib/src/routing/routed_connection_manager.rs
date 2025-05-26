use crate::pool::ManagedConnection;
use crate::routing::connection_registry::{
    start_background_updater, BoltServer, ConnectionRegistry, Registry, RegistryCommand,
};
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::RoundRobinStrategy;
use crate::Database;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{Config, Error, Operation};
use backon::ExponentialBuilder;
use futures::lock::Mutex;
use log::{debug, error};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    connection_registry: Arc<ConnectionRegistry>,
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: ExponentialBuilder,
    channel: Sender<RegistryCommand>,
}

impl RoutedConnectionManager {
    pub fn new(config: &Config, provider: Arc<dyn RoutingTableProvider>) -> Result<Self, Error> {
        let backoff = crate::pool::backoff();
        let connection_registry = Arc::new(ConnectionRegistry::default());
        let channel = start_background_updater(config, connection_registry.clone(), provider);
        Ok(RoutedConnectionManager {
            load_balancing_strategy: Arc::new(RoundRobinStrategy::default()),
            bookmarks: Arc::new(Mutex::new(vec![])),
            connection_registry,
            backoff,
            channel,
        })
    }

    pub(crate) async fn get(
        &self,
        operation: Option<Operation>,
        db: Option<Database>,
    ) -> Result<ManagedConnection, Error> {
        let op = operation.unwrap_or(Operation::Write);
        if db.is_some() {
            let registry = self
                .connection_registry
                .get_or_create_registry(db.clone().unwrap());
            // If the registry is empty, we need to refresh the routing table
            if registry.is_empty()
                && self
                    .channel
                    .send(RegistryCommand::Refresh(
                        self.bookmarks.lock().await.clone(),
                    ))
                    .await
                    .is_err()
            {
                error!("Failed to send refresh command to registry channel");
                return Err(Error::RoutingTableRefreshFailed(
                    "Failed to send refresh command to registry channel".to_string(),
                ));
            }
            self.inner_get(&registry, op, db).await
        } else {
            self.inner_get(&self.connection_registry.default_registry, op, db)
                .await
        }
    }

    async fn inner_get(
        &self,
        registry: &Registry,
        op: Operation,
        db: Option<Database>,
    ) -> Result<ManagedConnection, Error> {
        loop {
            // we loop here until we get a connection. If the routing table is empty, we force a refresh
            if registry.is_empty() {
                // the first time we need to wait until we get the routing table
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }

            while let Some(server) = match op {
                Operation::Write => self.select_writer(db.clone()),
                _ => self.select_reader(db.clone()),
            } {
                debug!("requesting connection for server: {:?}", server);
                if let Some(pool) = self.connection_registry.get_pool(&server, db.clone()) {
                    match pool.get().await {
                        Ok(connection) => return Ok(connection),
                        Err(e) => {
                            error!(
                                "Failed to get connection from pool for server `{}`: {}",
                                server.address, e
                            );
                            self.connection_registry
                                .mark_unavailable(&server, db.clone());
                            continue;
                        }
                    }
                } else {
                    // We couldn't find a connection manager for the server, it was probably marked unavailable
                    error!(
                        "No connection manager available for router `{}` in the registry",
                        server.address
                    );
                    return Err(Error::ServerUnavailableError(format!(
                        "No connection manager available for router `{}` in the registry",
                        server.address
                    )));
                }
            }
            debug!("Routing table is empty for requested {op} operation, forcing refresh");
            self.channel
                .send(RegistryCommand::Refresh(
                    self.bookmarks.lock().await.clone(),
                ))
                .await
                .map_err(|e| {
                    error!("Failed to send refresh command to registry: {}", e);
                    Error::RoutingTableRefreshFailed(
                        "Failed to send refresh command to registry".to_string(),
                    )
                })?;
            // table is not empty, but we couldn't get a connection, so we throw an error
            break Err(Error::ServerUnavailableError(format!(
                "No server available for {op} operation"
            )));
        }
    }

    pub(crate) fn backoff(&self) -> ExponentialBuilder {
        self.backoff
    }

    fn select_reader(&self, db: Option<Database>) -> Option<BoltServer> {
        self.load_balancing_strategy
            .select_reader(&self.connection_registry.servers(db))
    }

    fn select_writer(&self, db: Option<Database>) -> Option<BoltServer> {
        self.load_balancing_strategy
            .select_writer(&self.connection_registry.servers(db))
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
