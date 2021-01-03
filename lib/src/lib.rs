//! Neo4j driver compatible with neo4j 4.x versions
//!
//! * An implementation of the [bolt protocol][bolt] to interact with Neo4j server
//! * async/await apis using [tokio][tokio]
//! * Supports bolt 4.2 specification
//! * tested with Neo4j versions: 4.0, 4.1, 4.2
//!
//!
//! [bolt]: https://7687.org/
//! [tokio]: https://github.com/tokio-rs/tokio
//!
//!
//! # Examples
//!
//! ```
//! use neo4rs::*;
//! use std::sync::Arc;
//! use std::sync::atomic::{AtomicU32, Ordering};
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let id = Uuid::new_v4().to_string();
//!
//!    let graph = Arc::new(Graph::new(&uri, user, pass).await.unwrap());
//!    let mut result = graph.run(
//!      query("CREATE (p:Person {id: $id})").param("id", id.clone())
//!    ).await.unwrap();
//!
//!    let mut handles = Vec::new();
//!    let mut count = Arc::new(AtomicU32::new(0));
//!    for _ in 1..=42 {
//!        let graph = graph.clone();
//!        let id = id.clone();
//!        let count = count.clone();
//!        let handle = tokio::spawn(async move {
//!            let mut result = graph.execute(
//!              query("MATCH (p:Person {id: $id}) RETURN p").param("id", id)
//!            ).await.unwrap();
//!            while let Ok(Some(row)) = result.next().await {
//!                count.fetch_add(1, Ordering::Relaxed);
//!            }
//!        });
//!        handles.push(handle);
//!    }
//!
//!    futures::future::join_all(handles).await;
//!    assert_eq!(count.load(Ordering::Relaxed), 42);
//! }
//! ```
//!
//!
//! ## Nodes
//! A simple example to create a node and consume the created node from the row stream.
//!
//! * [`Graph::run`] just returns [`errors::Result`]`<()>`, usually used for write only queries.
//! * [`Graph::execute`] returns [`errors::Result`]`<`[`RowStream`]`>`
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
//!
//!    assert!(graph.run(query("RETURN 1")).await.is_ok());
//!
//!    let mut result = graph.execute(
//!      query( "CREATE (friend:Person {name: $name}) RETURN friend")
//!     .param("name", "Mr Mark")
//!    ).await.unwrap();
//!
//!    while let Ok(Some(row)) = result.next().await {
//!         let node: Node = row.get("friend").unwrap();
//!         let id = node.id();
//!         let labels = node.labels();
//!         let name: String = node.get("name").unwrap();
//!         assert_eq!(name, "Mr Mark");
//!         assert_eq!(labels, vec!["Person"]);
//!         assert!(id > 0);
//!     }
//! }
//! ```
//! ## Configurations
//!
//! Use the config builder to override the default configurations like
//! * `fetch_size` - number of rows to fetch in batches (default is 200)
//! * `max_connections` - maximum size of the connection pool (default is 16)
//! * `db` - the database to connect to (default is `neo4j`)
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let config = config()
//!        .uri("127.0.0.1:7687")
//!        .user("neo4j")
//!        .password("neo")
//!        .db("neo4j")
//!        .fetch_size(500)
//!        .max_connections(10)
//!        .build()
//!        .unwrap();
//!    let graph = Graph::connect(config).await.unwrap();
//!    let mut result = graph.execute(query("RETURN 1")).await.unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let value: i64 = row.get("1").unwrap();
//!    assert_eq!(1, value);
//!    assert!(result.next().await.unwrap().is_none());
//! }
//! ```
//!
//! ## Transactions
//!
//! Start a new transaction using [`Graph::start_txn`], which will return a handle [`Txn`] that can
//! be used to [`Txn::commit`] or [`Txn::rollback`] the transaction.
//!
//! Note that the handle takes a connection from the connection pool, which will be dropped once
//! the Txn is dropped
//!
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let txn = graph.start_txn().await.unwrap();
//!    let id = Uuid::new_v4().to_string();
//!    let result = txn.run_queries(vec![
//!            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
//!            query("CREATE (p:Person {id: $id})").param("id", id.clone())
//!     ]).await;
//!
//!    assert!(result.is_ok());
//!    txn.commit().await.unwrap();
//!    let mut result = graph
//!        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    # assert!(result.next().await.unwrap().is_some());
//!    # assert!(result.next().await.unwrap().is_some());
//!    # assert!(result.next().await.unwrap().is_none());
//! }
//!
//! ```
//!
//! ### Streams within a transaction
//!
//! Each [`RowStream`] returned by various execute within the same transaction are well isolated,
//! so you can consume the stream anytime within the transaction using [`RowStream::next`]
//!
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let config = config()
//!        .uri("127.0.0.1:7687")
//!        .user("neo4j")
//!        .password("neo")
//!        .fetch_size(1)
//!        .build()
//!        .unwrap();
//!    let graph = Graph::connect(config).await.unwrap();
//!    let name = Uuid::new_v4().to_string();
//!    let txn = graph.start_txn().await.unwrap();
//!
//!    txn.run_queries(vec![
//!        query("CREATE (p { name: $name })").param("name", name.clone()),
//!        query("CREATE (p { name: $name })").param("name", name.clone()),
//!    ])
//!    .await
//!    .unwrap();
//!
//!
//!    //start stream_one
//!    let mut stream_one = txn
//!        .execute(query("MATCH (p {name: $name}) RETURN p").param("name", name.clone()))
//!        .await
//!        .unwrap();
//!    let row = stream_one.next().await.unwrap().unwrap();
//!    assert_eq!(row.get::<Node>("p").unwrap().get::<String>("name").unwrap(), name.clone());
//!
//!    //start stream_two
//!    let mut stream_two = txn.execute(query("RETURN 1")).await.unwrap();
//!    let row = stream_two.next().await.unwrap().unwrap();
//!    assert_eq!(row.get::<i64>("1").unwrap(), 1);
//!
//!    //stream_one is still active here
//!    let row = stream_one.next().await.unwrap().unwrap();
//!    assert_eq!(row.get::<Node>("p").unwrap().get::<String>("name").unwrap(), name.clone());
//!
//!    //stream_one completes
//!    assert!(stream_one.next().await.unwrap().is_none());
//!    //stream_two completes
//!    assert!(stream_two.next().await.unwrap().is_none());
//!    txn.commit().await.unwrap();
//! }
//!
//! ```
//!
//!
//! ### Rollback a transaction
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
//!    let txn = graph.start_txn().await.unwrap();
//!    let id = Uuid::new_v4().to_string();
//!    // create a node
//!    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    // rollback the changes
//!    txn.rollback().await.unwrap();
//!
//!    // changes not updated in the database
//!    let mut result = graph
//!        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    assert!(result.next().await.unwrap().is_none());
//! }
//!
//! ```
//!
//! ### Txn vs Graph
//!
//! Everytime you execute a query using [`Graph::run`] or [`Graph::execute`], a new connection is
//! taken from the pool and released immediately.
//!
//! However, when you execute a query on a transaction using [`Txn::run`] or [`Txn::execute`] the
//! same connection will be reused, the underlying connection will be released to the pool in a
//! clean state only after you commit/rollback the transaction and the [`Txn`] handle is dropped.
//!
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let txn = graph.start_txn().await.unwrap();
//!    let id = Uuid::new_v4().to_string();
//!    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    // graph.execute(..) will not see the changes done above as the txn is not committed yet
//!    let mut result = graph
//!        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    assert!(result.next().await.unwrap().is_none());
//!    txn.commit().await.unwrap();
//!
//!    //changes are now seen as the transaction is committed.
//!    let mut result = graph
//!        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
//!        .await
//!        .unwrap();
//!    assert!(result.next().await.unwrap().is_some());
//!    assert!(result.next().await.unwrap().is_some());
//!    assert!(result.next().await.unwrap().is_none());
//! }
//!
//! ```
//!
//! ## Relationships
//!
//! Bounded Relationship between nodes are created using cypher queries and the same can be parsed
//! from the [`RowStream`]
//!
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
//!        query("CREATE (p:Person { name: 'Oliver Stone' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r")
//!    ).await.unwrap();
//!
//!    let row = result.next().await.unwrap().unwrap();
//!    let relation: Relation = row.get("r").unwrap();
//!    assert!(relation.id() > -1);
//!    assert!(relation.start_node_id() > -1);
//!    assert!(relation.end_node_id() > -1);
//!    assert_eq!(relation.typ(), "WORKS_AT");
//!    assert_eq!(relation.get::<String>("as").unwrap(), "Engineer");
//! }
//! ```
//!
//!
//! Similar to bounded relation, an unbounded relation can also be created/parsed.
//!
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
//!        query("MERGE (p1:Person { name: 'Oliver Stone' })-[r:RELATED {as: 'friend'}]-(p2: Person {name: 'Mark'}) RETURN r")
//!    ).await.unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let relation: Relation = row.get("r").unwrap();
//!    assert!(relation.id() > -1);
//!    assert!(relation.start_node_id() > -1);
//!    assert!(relation.end_node_id() > -1);
//!    assert_eq!(relation.typ(), "RELATED");
//!    assert_eq!(relation.get::<String>("as").unwrap(), "friend");
//! }
//!
//! ```
//!
//!
//!
//! ## Points
//!
//! A 2d or 3d point can be represented with the types  [`Point2D`] and [`Point3D`]
//!
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
//!    let mut result = graph
//!        .execute(query(
//!            "WITH point({ x: 2.3, y: 4.5, crs: 'cartesian' }) AS p1,
//!             point({ x: 1.1, y: 5.4, crs: 'cartesian' }) AS p2 RETURN distance(p1,p2) AS dist, p1, p2",
//!        ))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let dist: f64 = row.get("dist").unwrap();
//!    let p1: Point2D = row.get("p1").unwrap();
//!    let p2: Point2D = row.get("p2").unwrap();
//!    assert_eq!(1.5, dist);
//!    assert_eq!(p1.sr_id(), 7203);
//!    assert_eq!(p1.x(), 2.3);
//!    assert_eq!(p1.y(), 4.5);
//!    assert_eq!(p2.sr_id(), 7203);
//!    assert_eq!(p2.x(), 1.1);
//!    assert_eq!(p2.y(), 5.4);
//!    assert!(result.next().await.unwrap().is_none());
//!
//!    let mut result = graph
//!        .execute(query(
//!            "RETURN point({ longitude: 56.7, latitude: 12.78, height: 8 }) AS point",
//!        ))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let point: Point3D = row.get("point").unwrap();
//!    assert_eq!(point.sr_id(), 4979);
//!    assert_eq!(point.x(), 56.7);
//!    assert_eq!(point.y(), 12.78);
//!    assert_eq!(point.z(), 8.0);
//!    assert!(result.next().await.unwrap().is_none());
//!
//! }
//!
//! ```
//!
//! ## Raw bytes
//!
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let mut result = graph
//!        .execute(query("RETURN $b as output").param("b", vec![11, 12]))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let b: Vec<u8> = row.get("output").unwrap();
//!    assert_eq!(b, &[11, 12]);
//!    assert!(result.next().await.unwrap().is_none());
//! }
//!
//! ```
//!
//! ## Durations
//!
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let duration = std::time::Duration::new(5259600, 7);
//!    let mut result = graph
//!        .execute(query("RETURN $d as output").param("d", duration))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let d: std::time::Duration = row.get("output").unwrap();
//!    assert_eq!(d.as_secs(), 5259600);
//!    assert_eq!(d.subsec_nanos(), 7);
//!    assert!(result.next().await.unwrap().is_none());
//! }
//!
//! ```
//! ## Date
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let date = chrono::NaiveDate::from_ymd(1985, 2, 5);
//!    let mut result = graph
//!        .execute(query("RETURN $d as output").param("d", date))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let d: chrono::NaiveDate = row.get("output").unwrap();
//!    assert_eq!(d.to_string(), "1985-02-05");
//!    assert!(result.next().await.unwrap().is_none());
//! }
//! ```
//!
//!
//! ## Time
//!
//! Neo4rs uses [chrono][chrono] crate for time abstractions.
//!
//! [chrono]: https://crates.io/crates/chrono
//!
//!
//! ### Time as param
//!
//! Pass a time (without offset) as a parameter to the query:
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
//!    //send time without offset as param
//!    let time = chrono::NaiveTime::from_hms_nano(11, 15, 30, 200);
//!    let mut result = graph.execute(query("RETURN $d as output").param("d", time)).await.unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let t: (chrono::NaiveTime, Option<chrono::FixedOffset>) = row.get("output").unwrap();
//!    assert_eq!(t.0.to_string(), "11:15:30.000000200");
//!    assert_eq!(t.1, None);
//!    assert!(result.next().await.unwrap().is_none());
//!
//!
//!    //send time with offset as param
//!    let time = chrono::NaiveTime::from_hms_nano(11, 15, 30, 200);
//!    let offset = chrono::FixedOffset::east(3 * 3600);
//!    let mut result = graph
//!        .execute(query("RETURN $d as output").param("d", (time, offset)))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let t: (chrono::NaiveTime, Option<chrono::FixedOffset>) = row.get("output").unwrap();
//!    assert_eq!(t.0.to_string(), "11:15:30.000000200");
//!    assert_eq!(t.1, Some(offset));
//!    assert!(result.next().await.unwrap().is_none());
//! }
//! ```
//!
//!
//! ### Parsing time from result
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
//!    //Parse time without offset
//!    let mut result = graph
//!        .execute(query(
//!            " WITH time({hour:10, minute:15, second:30, nanosecond: 200}) AS t RETURN t",
//!        ))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let t: (chrono::NaiveTime, Option<chrono::FixedOffset>) = row.get("t").unwrap();
//!    assert_eq!(t.0.to_string(), "10:15:30.000000200");
//!    assert_eq!(t.1, None);
//!    assert!(result.next().await.unwrap().is_none());
//!
//!    //Parse time with timezone information
//!    let mut result = graph
//!        .execute(query(
//!            " WITH time({hour:10, minute:15, second:33, nanosecond: 200, timezone: '+01:00'}) AS t RETURN t",
//!        ))
//!        .await
//!        .unwrap();
//!    let row = result.next().await.unwrap().unwrap();
//!    let t: (chrono::NaiveTime, Option<chrono::FixedOffset>) = row.get("t").unwrap();
//!    assert_eq!(t.0.to_string(), "10:15:33.000000200");
//!    assert_eq!(t.1, Some(chrono::FixedOffset::east(1 * 3600)));
//!    assert!(result.next().await.unwrap().is_none());
//! }
//!
//! ```
//!
//! ## Path
//!
//! ```
//! use neo4rs::*;
//! use futures::stream::*;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!    let name = Uuid::new_v4().to_string();
//!    graph.run(
//!      query("CREATE (p:Person { name: $name })-[r:WORKS_AT]->(n:Company { name: 'Neo'})").param("name", name.clone()),
//!    ).await.unwrap();
//!
//!    let mut result = graph.execute(
//!       query("MATCH p = (person:Person { name: $name })-[r:WORKS_AT]->(c:Company) RETURN p").param("name", name),
//!    ).await.unwrap();
//!
//!    let row = result.next().await.unwrap().unwrap();
//!    let path: Path = row.get("p").unwrap();
//!    assert_eq!(path.ids().len(), 2);
//!    assert_eq!(path.nodes().len(), 2);
//!    assert_eq!(path.rels().len(), 1);
//!    assert!(result.next().await.unwrap().is_none());
//! }
//! ```
//!
//!
mod config;
mod connection;
mod convert;
mod errors;
mod graph;
mod messages;
mod pool;
mod query;
mod row;
mod stream;
mod txn;
mod types;
mod version;

pub use crate::config::{config, Config, ConfigBuilder};
pub use crate::errors::*;
pub use crate::graph::{query, Graph};
pub use crate::query::Query;
pub use crate::row::{Node, Path, Point2D, Point3D, Relation, Row, UnboundedRelation};
pub use crate::stream::RowStream;
pub use crate::txn::Txn;
pub use crate::version::Version;
