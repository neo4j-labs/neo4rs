use crate::connection::Connection;
use crate::errors::Error;
use async_trait::async_trait;
use log::info;

pub type ConnectionPool = deadpool::managed::Pool<Connection, Error>;
pub type ManagedConnection = deadpool::managed::Object<Connection, Error>;

pub struct ConnectionManager {
    uri: String,
    user: String,
    password: String,
}

impl ConnectionManager {
    pub fn new(uri: &str, user: &str, password: &str) -> ConnectionManager {
        ConnectionManager {
            uri: uri.to_owned(),
            user: user.to_owned(),
            password: password.to_owned(),
        }
    }
}

#[async_trait]
impl deadpool::managed::Manager<Connection, Error> for ConnectionManager {
    async fn create(&self) -> std::result::Result<Connection, Error> {
        info!("creating new connection...");
        Connection::new(&self.uri, &self.user, &self.password).await
    }

    async fn recycle(&self, conn: &mut Connection) -> deadpool::managed::RecycleResult<Error> {
        info!("resetting connection...");
        Ok(conn.reset().await?)
    }
}

pub async fn create_pool(uri: &str, user: &str, password: &str) -> ConnectionPool {
    let max_size = 16;
    let mgr = ConnectionManager::new(uri, user, password);
    info!("creating connection pool with max size {}", max_size);
    ConnectionPool::new(mgr, max_size)
}
