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
        connection_timeout: Duration,
        tcp_keepalive: Option<Duration>,
    ) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password, tls_config, connection_timeout, tcp_keepalive)?;
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
        match tokio::time::timeout(Duration::from_secs(5), obj.reset()).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(deadpool::managed::RecycleError::Backend(e)),
            Err(_) => Err(deadpool::managed::RecycleError::message(
                "Connection health check timed out",
            )),
        }
    }
}

pub fn create_pool(config: &Config) -> Result<ConnectionPool> {
    let mgr = ConnectionManager::new(
        &config.uri,
        &config.user,
        &config.password,
        &config.tls_config,
        config.connection_timeout,
        config.tcp_keepalive,
    )?;
    info!(
        "creating connection pool for node {} with max size {}",
        config.uri, config.max_connections
    );
    let mut builder = ConnectionPool::builder(mgr)
        .max_size(config.max_connections);

    // Wire idle_timeout as the recycle timeout — connections idle longer than this
    // will fail the recycle check, causing deadpool to discard and recreate them.
    if let Some(idle_timeout) = config.idle_timeout {
        builder = builder
            .recycle_timeout(Some(idle_timeout))
            .runtime(deadpool::Runtime::Tokio1);
    }

    // Wire max_lifetime as the wait timeout — puts an upper bound on how long
    // a caller will wait for a connection from the pool.
    if let Some(max_lifetime) = config.max_lifetime {
        builder = builder
            .wait_timeout(Some(max_lifetime))
            .runtime(deadpool::Runtime::Tokio1);
    }

    Ok(builder.build().expect("Pool build failed"))
}
