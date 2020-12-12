use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::query::*;

pub struct Txn {
    connection: ManagedConnection,
}

impl Txn {
    pub async fn new(mut connection: ManagedConnection) -> Result<Self> {
        let begin = BoltRequest::begin();
        match connection.send_recv(begin).await? {
            BoltResponse::SuccessMessage(_) => Ok(Txn { connection }),
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn run_queries(&mut self, queries: Vec<Query>) -> Result<()> {
        for query in queries.into_iter() {
            query.run(&mut self.connection).await?;
        }
        Ok(())
    }

    pub async fn run(&mut self, q: Query) -> Result<()> {
        q.run(&mut self.connection).await
    }

    pub async fn commit(mut self) -> Result<()> {
        match self.connection.send_recv(BoltRequest::commit()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            msg => {
                eprintln!("unexpected message {:?}", msg);
                Err(Error::UnexpectedMessage)
            }
        }
    }

    pub async fn rollback(mut self) -> Result<()> {
        match self.connection.send_recv(BoltRequest::rollback()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
