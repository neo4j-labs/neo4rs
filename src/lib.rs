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
//!    let query = graph
//!        .query("CREATE (friend:Person {name: $name}) RETURN friend")
//!        .param("name", "Mr Mark");
//!    let mut result = graph.execute(query).await.unwrap();
//!
//!    while let Some(row) = result.next().await {
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
mod txn;
mod types;
mod version;
pub use crate::errors::*;
use crate::pool::*;
use crate::query::*;
pub use crate::row::{Node, Relation, Row};
pub use crate::txn::Txn;
pub use crate::version::Version;

pub struct Graph {
    pool: bb8::Pool<ConnectionManager>,
}

impl Graph {
    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let manager = ConnectionManager::new(uri, user, password);
        let pool = bb8::Pool::builder().max_size(15).build(manager).await?;
        Ok(Graph { pool })
    }

    pub fn query(&self, q: &str) -> Query {
        Query::new(q.to_owned())
    }

    pub async fn run(&self, q: Query) -> Result<()> {
        let connection = self.pool.get().await?;
        q.run(connection.get()).await
    }

    pub async fn version(&self) -> Result<Version> {
        Ok(self.pool.get().await?.version())
    }

    pub async fn execute(&self, q: Query) -> Result<tokio::sync::mpsc::Receiver<Row>> {
        let connection = self.pool.get().await?;
        q.execute(connection.get()).await
    }
}
