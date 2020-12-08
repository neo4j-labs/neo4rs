//! Neo4j driver imlemented using bolt 4.1 specification
//!
//! #Example
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//!
//! pub async fn run_me() {
//!  let uri = "127.0.0.1:7687".to_owned();
//!  let user = "neo4j";
//!  let pass = "neo4j";
//!  let graph = Graph::connect(&uri, user, pass).await.unwrap();
//!  let mut result = graph
//!        .query("CREATE (friend:Person {name: $name}) RETURN friend")
//!        .param("name", "Mark")
//!        .execute()
//!        .await
//!        .unwrap();
//!
//!  while let Some(row) = result.next().await {
//!     let node: Node = row.get("friend").unwrap();
//!     let name: String = node.get("name").unwrap();
//!     assert_eq!(name, "Mark");
//!  }
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

#[derive(Debug)]
pub struct Graph {
    pub version: Version,
    connections: bb8::Pool<ConnectionManager>,
}

impl Graph {
    pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Self> {
        let manager = ConnectionManager::new(uri, user, password);
        let connections = bb8::Pool::builder().max_size(15).build(manager).await?;
        let connection = connections.get().await?;
        Ok(Graph {
            version: connection.version.clone(),
            connections: connections.clone(),
        })
    }

    pub fn query(&self, q: &str) -> QueryBuilder {
        QueryBuilder::new(q.to_owned(), self.connections.clone())
    }

    //pub async fn begin_txn(&self) -> Result<Txn> {
    //    Ok(Txn::new(self.connections.clone()).await?)
    //}
}
