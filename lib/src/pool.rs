use crate::{
    config::Config,
    connection::{Connection, ConnectionInfo},
    errors::{Error, Result},
};
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use log::info;

pub type ConnectionPool = Pool<ConnectionManager>;
pub type ManagedConnection = Object<ConnectionManager>;

pub struct ConnectionManager {
    info: ConnectionInfo,
}

impl ConnectionManager {
    pub fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password)?;
        Ok(ConnectionManager { info })
    }
}

impl Manager for ConnectionManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        info!("creating new connection...");
        Connection::new(&self.info).await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        Ok(obj.reset().await?)
    }
}

pub async fn create_pool(config: &Config) -> Result<ConnectionPool> {
    let mgr = ConnectionManager::new(&config.uri, &config.user, &config.password)?;
    info!(
        "creating connection pool with max size {}",
        config.max_connections
    );
    Ok(ConnectionPool::builder(mgr)
        .max_size(config.max_connections)
        .build()
        .expect("No timeouts configured"))
}
