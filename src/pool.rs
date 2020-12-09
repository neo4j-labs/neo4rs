use crate::connection::Connection;
use crate::errors::Error;
use crate::messages::*;
use crate::version::Version;
use async_trait::async_trait;
use bb8::ManageConnection;
use bb8::PooledConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ConnectionWrapper {
    version: Version,
    connection: Arc<Mutex<Connection>>,
}

impl ConnectionWrapper {
    pub fn get(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

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
impl ManageConnection for ConnectionManager {
    type Connection = ConnectionWrapper;
    type Error = Error;

    async fn connect(&self) -> Result<ConnectionWrapper, Self::Error> {
        let (mut connection, version) = Connection::new(self.uri.clone()).await?;
        let hello = BoltRequest::hello("neo4rs", self.user.clone(), self.password.clone());
        match connection.send_recv(hello).await? {
            BoltResponse::SuccessMessage(_msg) => Ok(ConnectionWrapper {
                version,
                connection: Arc::new(Mutex::new(connection)),
            }),
            BoltResponse::FailureMessage(msg) => Err(Error::AuthenticationError {
                detail: msg.get("message").unwrap(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn create_pool(
    uri: &str,
    user: &str,
    password: &str,
) -> Result<bb8::Pool<ConnectionManager>, Error> {
    let manager = ConnectionManager::new(uri, user, password);
    let pool = bb8::Pool::builder().max_size(15).build(manager).await?;
    Ok(pool)
}
