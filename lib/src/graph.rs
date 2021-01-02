use crate::config::{config, Config};
use crate::errors::*;
use crate::pool::{create_pool, ConnectionPool};
use crate::query::Query;
use crate::stream::RowStream;
use crate::txn::Txn;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Graph {
    config: Config,
    pool: ConnectionPool,
}

pub fn query(q: &str) -> Query {
    Query::new(q.to_owned())
}

impl Graph {
    pub async fn connect(config: Config) -> Result<Self> {
        let pool = create_pool(&config).await;
        Ok(Graph { config, pool })
    }

    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let config = config().uri(uri).user(user).password(password).build()?;
        Self::connect(config).await
    }

    pub async fn start_txn(&self) -> Result<Txn> {
        let connection = self.pool.get().await?;
        Txn::new(self.config.clone(), connection).await
    }

    pub async fn run(&self, q: Query) -> Result<()> {
        let connection = Arc::new(Mutex::new(self.pool.get().await?));
        q.run(&self.config, connection).await
    }

    pub async fn execute(&self, q: Query) -> Result<RowStream> {
        let connection = Arc::new(Mutex::new(self.pool.get().await?));
        q.execute(&self.config, connection).await
    }
}
