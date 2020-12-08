use crate::errors::*;
use crate::messages::*;
use crate::pool::ConnectionManager;

#[derive(Debug)]
pub struct Txn {
    connections: bb8::Pool<ConnectionManager>,
}

impl Txn {
    pub async fn new(connections: bb8::Pool<ConnectionManager>) -> Result<Self> {
        let begin = BoltRequest::begin();
        match connections.get().await?.send_recv(begin).await? {
            BoltResponse::SuccessMessage(_) => Ok(Txn {
                connections: connections.clone(),
            }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn commit(&self) -> Result<()> {
        let mut connection = self.connections.get().await?;
        match connection.send_recv(BoltRequest::commit()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn rollback(&self) -> Result<()> {
        let mut connection = self.connections.get().await?;
        match connection.send_recv(BoltRequest::rollback()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
