use crate::auth::ConnectionTLSConfig;
use crate::config::BackoffConfig;
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
    backoff: Option<ExponentialBuilder>,
}

impl ConnectionManager {
    pub fn new(
        uri: &str,
        user: &str,
        password: &str,
        tls_config: &ConnectionTLSConfig,
        backoff_config: Option<&BackoffConfig>,
    ) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password, tls_config)?;
        let backoff = backoff_config.map(|backoff_config| backoff_config.to_exponential_builder());
        Ok(ConnectionManager { info, backoff })
    }

    pub fn backoff(&self) -> Option<ExponentialBuilder> {
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

pub fn create_pool(config: &Config) -> Result<ConnectionPool> {
    let mgr = ConnectionManager::new(
        &config.uri,
        &config.user,
        &config.password,
        &config.tls_config,
        config.backoff.as_ref(),
    )?;
    info!(
        "creating connection pool for node {} with max size {}",
        config.uri, config.max_connections
    );
    Ok(ConnectionPool::builder(mgr)
        .max_size(config.max_connections)
        .build()
        .expect("No timeouts configured"))
}
