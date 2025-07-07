use std::time::Duration;

use crate::auth::ConnectionTLSConfig;
use crate::{
    config::Config,
    connection::{Connection, ConnectionInfo},
    errors::{Error, Result},
};
use backon::ExponentialBuilder;
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use log::{info, trace};

pub type ConnectionPool = Pool<ConnectionManager>;
pub type ManagedConnection = Object<ConnectionManager>;

pub struct ConnectionManager {
    info: ConnectionInfo,
    backoff: ExponentialBuilder,
}

impl ConnectionManager {
    pub fn new(
        uri: &str,
        user: &str,
        password: &str,
        tls_config: &ConnectionTLSConfig,
    ) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password, tls_config)?;
        let backoff = backoff();
        Ok(ConnectionManager { info, backoff })
    }

    pub fn backoff(&self) -> ExponentialBuilder {
        self.backoff
    }
}

pub(crate) fn backoff() -> ExponentialBuilder {
    ExponentialBuilder::new()
        .with_jitter()
        .with_factor(2.0)
        .without_max_times()
        .with_min_delay(Duration::from_millis(1))
        .with_max_delay(Duration::from_secs(10))
        .with_total_delay(Some(Duration::from_secs(60)))
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

pub fn create_pool(config: &Config) -> Result<ConnectionPool> {
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
