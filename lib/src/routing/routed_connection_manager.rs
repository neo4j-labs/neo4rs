use crate::pool::ManagedConnection;
use crate::routing::connection_registry::{
    start_background_updater, BoltServer, ConnectionRegistry, RegistryCommand,
};
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::RoundRobinStrategy;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{Config, Error, Operation};
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use futures::lock::Mutex;
use log::{debug, error};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    connection_registry: Arc<ConnectionRegistry>,
    #[allow(dead_code)]
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: Arc<ExponentialBackoff>,
    channel: Sender<RegistryCommand>,
}

impl RoutedConnectionManager {
    pub async fn new(
        config: &Config,
        provider: Box<dyn RoutingTableProvider>,
    ) -> Result<Self, Error> {
        let backoff = Arc::new(
            ExponentialBackoffBuilder::new()
                .with_initial_interval(Duration::from_millis(1))
                .with_randomization_factor(0.42)
                .with_multiplier(2.0)
                .with_max_elapsed_time(Some(Duration::from_secs(60)))
                .build(),
        );

        let connection_registry = Arc::new(ConnectionRegistry::default());
        let channel =
            start_background_updater(config, connection_registry.clone(), provider.into()).await;
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
    ) -> Result<ManagedConnection, Error> {
        let op = operation.unwrap_or(Operation::Write);
        while let Some(server) = match op {
            Operation::Write => self.select_writer(),
            _ => self.select_reader(),
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
                        self.connection_registry.mark_unavailable(&server);
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
            .send(RegistryCommand::Refresh)
            .await
            .map_err(|e| {
                error!("Failed to send refresh command to registry: {}", e);
                Error::RoutingTableRefreshFailed(
                    "Failed to send refresh command to registry".to_string(),
                )
            })?;
        Err(Error::ServerUnavailableError(format!(
            "No server available for {op} operation"
        )))
    }

    pub(crate) fn backoff(&self) -> ExponentialBackoff {
        self.backoff.as_ref().clone()
    }

    fn select_reader(&self) -> Option<BoltServer> {
        self.load_balancing_strategy
            .select_reader(&self.connection_registry.servers())
    }

    fn select_writer(&self) -> Option<BoltServer> {
        self.load_balancing_strategy
            .select_writer(&self.connection_registry.servers())
    }

    #[allow(dead_code)]
    pub(crate) async fn add_bookmark(&self, bookmark: String) {
        self.bookmarks.lock().await.push(bookmark);
    }

    #[allow(dead_code)]
    pub(crate) async fn remove_bookmark(&self, bookmark: &str) {
        let mut bookmarks = self.bookmarks.lock().await;
        if let Some(index) = bookmarks.iter().position(|b| b == bookmark) {
            bookmarks.remove(index);
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn clear_bookmarks(&self) {
        self.bookmarks.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_routed_connection_manager() {}
}
