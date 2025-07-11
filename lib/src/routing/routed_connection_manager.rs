use crate::pool::ManagedConnection;
use crate::routing::connection_registry::{
    start_background_updater, BoltServer, ConnectionRegistry, RegistryCommand,
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
    backoff: Option<ExponentialBuilder>,
    channel: Sender<RegistryCommand>,
}

const ROUTING_TABLE_MAX_WAIT_TIME_MS: i32 = 5000;

impl RoutedConnectionManager {
    pub fn new(config: &Config, provider: Arc<dyn RoutingTableProvider>) -> Result<Self, Error> {
        // backoff config should be set to None here, since the routing table updater will handle retries
        // We could provide some configuration to "force" the retry mechanism in a clustered env,
        // but for now we will turn it off
        let backoff = config
            .backoff
            .clone()
            .map(|config| config.to_exponential_builder());
        let connection_registry = Arc::new(ConnectionRegistry::default());
        let channel = start_background_updater(config, connection_registry.clone(), provider);
        Ok(RoutedConnectionManager {
            load_balancing_strategy: Arc::new(RoundRobinStrategy::new(connection_registry.clone())),
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
        let registry = self.connection_registry.servers(db.clone());
        // If the registry is empty, we need to refresh the routing table immediately
        if registry.is_empty() {
            debug!("Routing table is empty, refreshing");
            if let Err(error) = self
                .channel
                .send(RegistryCommand::RefreshSingleTable((
                    db.clone(),
                    self.bookmarks.lock().await.clone(),
                )))
                .await
            {
                error!("Failed to send refresh command to registry channel");
                return Err(Error::RoutingTableRefreshFailed(error.to_string()));
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let mut attempts = 0;
        loop {
            // we loop here until we get a connection. If the routing table is empty, we force a refresh
            let servers = self.connection_registry.servers(db.clone());
            if servers.is_empty() {
                // the first time we need to wait until we get the routing table
                tokio::time::sleep(Duration::from_millis(10)).await;
                attempts += 10;
                if attempts > ROUTING_TABLE_MAX_WAIT_TIME_MS {
                    // 5 seconds max wait time by default (we don't want to block forever)
                    error!(
                        "Failed to get a connection after {} seconds, routing table is still empty",
                        ROUTING_TABLE_MAX_WAIT_TIME_MS / 1000
                    );
                    return Err(Error::ServerUnavailableError(format!(
                        "Routing table is still empty after {} seconds",
                        ROUTING_TABLE_MAX_WAIT_TIME_MS / 1000
                    )));
                }
                continue;
            }

            debug!(
                "Routing table is now not empty, trying to get a connection for operation: {:?}",
                op
            );

            while let Some(server) = match op {
                Operation::Write => self.select_writer(db.clone()),
                _ => self.select_reader(db.clone()),
            } {
                debug!("requesting connection for server: {:?}", server);
                if let Some(pool) = self.connection_registry.get_pool(&server) {
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
                }
            }
            debug!("No connection for requested {op} operation, forcing refresh of the routing table for database `{}`", db.as_deref().unwrap_or("default"));
            self.channel
                .send(RegistryCommand::RefreshSingleTable((
                    db.clone(),
                    self.bookmarks.lock().await.clone(),
                )))
                .await
                .map_err(|e| {
                    error!("Failed to send refresh command to registry: {}", e);
                    Error::RoutingTableRefreshFailed(
                        "Failed to send refresh command to registry".to_string(),
                    )
                })?;
            // table is not empty, but we couldn't get a connection, so we throw an error
            break Err(Error::ServerUnavailableError(format!(
                "No server available for {op} operation on db `{}`",
                db.as_deref().unwrap_or("default")
            )));
        }
    }

    pub(crate) fn backoff(&self) -> Option<ExponentialBuilder> {
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
