use std::time::Duration;

use crate::{
    auth::ClientCertificate,
    config::Config,
    connection::{Connection, ConnectionInfo},
    errors::{Error, Result},
};
use async_trait::async_trait;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use log::info;

pub type ConnectionPool = deadpool::managed::Pool<ConnectionManager>;
pub type ManagedConnection = deadpool::managed::Object<ConnectionManager>;

pub struct ConnectionManager {
    info: ConnectionInfo,
    backoff: ExponentialBackoff,
}

impl ConnectionManager {
    pub fn new(
        uri: &str,
        user: &str,
        password: &str,
        client_certificate: Option<&ClientCertificate>,
    ) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password, client_certificate)?;
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

#[async_trait]
impl deadpool::managed::Manager for ConnectionManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Connection, Error> {
        info!("creating new connection...");
        Connection::new(&self.info).await
    }

    async fn recycle(&self, conn: &mut Connection) -> deadpool::managed::RecycleResult<Error> {
        Ok(conn.reset().await?)
    }
}

pub async fn create_pool(config: &Config) -> Result<ConnectionPool, Error> {
    let mgr = ConnectionManager::new(
        &config.uri,
        &config.user,
        &config.password,
        config.client_certificate.as_ref(),
    )?;
    info!(
        "creating connection pool with max size {}",
        config.max_connections
    );
    Ok(ConnectionPool::builder(mgr)
        .max_size(config.max_connections)
        .build()?)
}
