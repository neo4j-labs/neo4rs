use std::time::Duration;

use crate::auth::ConnectionTLSConfig;
use crate::{
    config::Config,
    connection::{Connection, ConnectionInfo},
    errors::{Error, Result},
};
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use log::{info, trace};

pub type ConnectionPool = Pool<ConnectionManager>;
pub type ManagedConnection = Object<ConnectionManager>;

pub struct ConnectionManager {
    info: ConnectionInfo,
    backoff: ExponentialBackoff,
}

impl ConnectionManager {
    pub fn new(
        uri: &str,
        user: &str,
        password: &str,
        tls_config: &ConnectionTLSConfig,
    ) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password, tls_config)?;
        let backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_millis(1))
            .with_randomization_factor(0.42)
            .with_multiplier(2.0)
            .with_max_elapsed_time(Some(Duration::from_secs(60)))
            .build();
        Ok(ConnectionManager { info, backoff })
    }

    pub fn backoff(&self) -> ExponentialBackoff {
        self.backoff.clone()
    }
}

impl Manager for ConnectionManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        trace!("creating new connection");
        Connection::new(&self.info).await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        trace!("recycling connection");
        Ok(obj.reset().await?)
    }
}

pub async fn create_pool(config: &Config) -> Result<ConnectionPool> {
    let mgr = ConnectionManager::new(
        &config.uri,
        &config.user,
        &config.password,
        &config.tls_config,
    )?;
    info!(
        "creating connection pool with max size {}",
        config.max_connections
    );
    Ok(ConnectionPool::builder(mgr)
        .max_size(config.max_connections)
        .build()
        .expect("No timeouts configured"))
}
