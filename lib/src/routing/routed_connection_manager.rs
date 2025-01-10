use crate::connection::{Connection, ConnectionInfo};
use crate::pool::ManagedConnection;
use crate::routing::connection_registry::{BoltServer, ConnectionRegistry};
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::RoundRobinStrategy;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::routing::{RouteBuilder, RoutingTable};
use crate::{Config, Error, Operation};
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use futures::lock::Mutex;
use log::{debug, error};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    connection_registry: Arc<ConnectionRegistry>,
    #[allow(dead_code)]
    bookmarks: Arc<Mutex<Vec<String>>>,
    backoff: Arc<ExponentialBackoff>,
    config: Config,
}

impl RoutedConnectionManager {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let registry = Arc::new(ConnectionRegistry::new(config));
        let backoff = Arc::new(
            ExponentialBackoffBuilder::new()
                .with_initial_interval(Duration::from_millis(1))
                .with_randomization_factor(0.42)
                .with_multiplier(2.0)
                .with_max_elapsed_time(Some(Duration::from_secs(60)))
                .build(),
        );

        Ok(RoutedConnectionManager {
            load_balancing_strategy: Arc::new(RoundRobinStrategy::default()),
            connection_registry: registry,
            bookmarks: Arc::new(Mutex::new(vec![])),
            backoff,
            config: config.clone(),
        })
    }

    pub async fn refresh_routing_table(&self) -> Result<RoutingTable, Error> {
        let info = ConnectionInfo::new(
            &self.config.uri,
            &self.config.user,
            &self.config.password,
            &self.config.tls_config,
        )?;
        let mut connection = Connection::new(&info).await?;
        let mut builder = RouteBuilder::new(info.routing, vec![]);
        if let Some(db) = self.config.db.clone() {
            builder = builder.with_db(db);
        }
        let rt = connection
            .route(builder.build(connection.version()))
            .await?;
        debug!("Fetched a new routing table: {:?}", rt);
        Ok(rt)
    }

    pub(crate) async fn get(
        &self,
        operation: Option<Operation>,
    ) -> Result<ManagedConnection, Error> {
        // We probably need to do this in a more efficient way, since this will block the request of a connection
        // while we refresh the routing table. We should probably have a separate thread that refreshes the routing
        self.connection_registry
            .update_if_expired(|| self.refresh_routing_table())
            .await?;

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
}
