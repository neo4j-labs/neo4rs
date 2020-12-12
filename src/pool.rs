use crate::connection::Connection;
use crate::errors::Error;
use crate::messages::*;
use async_trait::async_trait;

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
        let (mut connection, _version) = Connection::new(self.uri.clone()).await?;
        let hello = BoltRequest::hello("neo4rs", self.user.clone(), self.password.clone());
        match connection.send_recv(hello).await? {
            BoltResponse::SuccessMessage(_msg) => Ok(connection),
            BoltResponse::FailureMessage(msg) => Err(Error::AuthenticationError {
                detail: msg.get("message").unwrap(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    async fn recycle(&self, _conn: &mut Connection) -> deadpool::managed::RecycleResult<Error> {
        Ok(())
    }
}

pub async fn create_pool(uri: &str, user: &str, password: &str) -> ConnectionPool {
    let mgr = ConnectionManager::new(uri, user, password);
    ConnectionPool::new(mgr, 16)
}
