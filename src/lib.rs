//! Neo4j driver imlemented using bolt 4.1 specification
//!
//! #Example
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let mut result = graph.execute(
//!      query( "CREATE (friend:Person {name: $name}) RETURN friend")
//!     .param("name", "Mr Mark")
//!    ).await.unwrap();
//!
//!    while let Ok(Some(row)) = result.next().await {
//!        let node: Node = row.get("friend").unwrap();
//!        let name: String = node.get("name").unwrap();
//!        assert_eq!(name, "Mr Mark");
//!     }
//! }
//! ```
mod connection;
mod convert;
mod errors;
mod messages;
mod pool;
mod query;
mod row;
mod stream;
mod txn;
mod types;
mod version;
pub use crate::errors::*;
use crate::pool::{create_pool, ConnectionPool};
pub use crate::query::Query;
pub use crate::row::{Node, Relation, Row};
pub use crate::stream::RowStream;
pub use crate::txn::Txn;
pub use crate::version::Version;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Graph {
    pool: ConnectionPool,
}

pub fn query(q: &str) -> Query {
    Query::new(q.to_owned())
}

impl Graph {
    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let pool = create_pool(uri, user, password).await;
        Ok(Graph { pool })
    }

    pub async fn start_txn(&self) -> Result<Txn> {
        let connection = self.pool.get().await?;
        Txn::new(connection).await
    }

    pub async fn run(&mut self, q: Query) -> Result<()> {
        let connection = self.pool.get().await?;
        q.run(Arc::new(Mutex::new(connection))).await
    }

    pub async fn execute(&self, q: Query) -> Result<RowStream> {
        let connection = self.pool.get().await?;
        q.execute(Arc::new(Mutex::new(connection))).await
    }
}
