use crate::{
    config::Config,
    connection::{Connection, ConnectionInfo},
    errors::{Error, Result},
};
use async_trait::async_trait;
use log::info;

pub type ConnectionPool = deadpool::managed::Pool<ConnectionManager>;
pub type ManagedConnection = deadpool::managed::Object<ConnectionManager>;

pub struct ConnectionManager {
    info: ConnectionInfo,
}

impl ConnectionManager {
    pub fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let info = ConnectionInfo::new(uri, user, password)?;
        Ok(ConnectionManager { info })
    }
}

#[async_trait]
impl deadpool::managed::Manager for ConnectionManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> std::result::Result<Connection, Error> {
        info!("creating new connection...");
        Connection::new(&self.info).await
    }

    async fn recycle(&self, conn: &mut Connection) -> deadpool::managed::RecycleResult<Error> {
        Ok(conn.reset().await?)
    }
}

pub async fn create_pool(config: &Config) -> Result<ConnectionPool, Error> {
    let mgr = ConnectionManager::new(&config.uri, &config.user, &config.password)?;
    info!(
        "creating connection pool with max size {}",
        config.max_connections
    );
    Ok(ConnectionPool::builder(mgr)
        .max_size(config.max_connections)
        .build()?)
}
