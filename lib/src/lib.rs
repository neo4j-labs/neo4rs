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
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let id = uuid::Uuid::new_v4().to_string();
//!
//!    let graph = std::sync::Arc::new(Graph::new(uri, user, pass).await.unwrap());
//!
#![doc = include_str!("../include/example.rs")]
//! }
//! ```
//!
//! ## Configurations
//!
//! Use the config builder to override the default configurations like
//! * `fetch_size` - number of rows to fetch in batches (default is 200)
//! * `max_connections` - maximum size of the connection pool (default is 16)
//! * `db` - the database to connect to (default is `neo4j`)
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let config = ConfigBuilder::default()
//!        .uri("127.0.0.1:7687")
//!        .user("neo4j")
//!        .password("neo")
//!        .db("neo4j")
//!        .fetch_size(500)
//!        .max_connections(10)
//!        .build()
//!        .unwrap();
//!    let graph = Graph::connect(config).await.unwrap();
//!
#![doc = include_str!("../include/configurations.rs")]
//! }
//! ```
//!
//! ## Nodes
//! A simple example to create a node and consume the created node from the row stream.
//!
//! * [`Graph::run`] just returns [`errors::Result`]`<()>`, usually used for write only queries.
//! * [`Graph::execute`] returns [`errors::Result`]`<`[`RowStream`]`>`
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/nodes.rs")]
//! }
//! ```
//!
//! ## Transactions
//!
//! Start a new transaction using [`Graph::start_txn`], which will return a handle [`Txn`] that can
//! be used to [`Txn::commit`] or [`Txn::rollback`] the transaction.
//!
//! Note that the handle takes a connection from the connection pool, which will be released once
//! the Txn is dropped
//!
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/transactions.rs")]
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
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let config = ConfigBuilder::default()
//!        .uri("127.0.0.1:7687")
//!        .user("neo4j")
//!        .password("neo")
//!        .fetch_size(1)
//!        .build()
//!        .unwrap();
//!    let graph = Graph::connect(config).await.unwrap();
//!
#![doc = include_str!("../include/streams_within_a_transaction.rs")]
//! }
//!
//! ```
//!
//!
//! ### Rollback a transaction
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/rollback_a_transaction.rs")]
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
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/txn_vs_graph.rs")]
//! }
//!
//! ```
//!
//! ## Relationships
//!
//! Bounded Relationship between nodes are created using cypher queries and the same can be parsed
//! from the [`RowStream`]
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/relationships.rs")]
//! }
//! ```
//!
//!
//! Similar to bounded relation, an unbounded relation can also be created/parsed.
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/unbounded_relationships.rs")]
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
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
//!    let qry = "
//!        WITH point({ x: 2.3, y: 4.5, crs: 'cartesian' }) AS p1,
//!             point({ x: 1.1, y: 5.4, crs: 'cartesian' }) AS p2
//!        RETURN point.distance(p1,p2) AS dist, p1, p2
//!     ";
#![doc = include_str!("../include/points.rs")]
//! }
//!
//! ```
//!
//! ## Raw bytes
//!
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/raw_bytes.rs")]
//! }
//!
//! ```
//!
//! ## Durations
//!
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/durations.rs")]
//! }
//!
//! ```
//! ## Date
//!
//! See [NaiveDate][naive_date] for date abstraction, it captures the date without time component.
//!
//! [naive_date]: https://docs.rs/chrono/0.4.19/chrono/naive/struct.NaiveDate.html
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/dates.rs")]
//! }
//! ```
//!
//!
//! ## Time
//!
//! * [NaiveTime][naive_time] captures only the time of the day
//! * `tuple`([NaiveTime][naive_time], `Option`<[FixedOffset][fixed_offset]>) captures the time of the day along with the
//! offset
//!
//! [naive_time]: https://docs.rs/chrono/0.4.19/chrono/naive/struct.NaiveTime.html
//! [fixed_offset]: https://docs.rs/chrono/0.4.19/chrono/offset/struct.FixedOffset.html
//!
//!
//! ### Time as param
//!
//! Pass a time as a parameter to the query:
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/time_as_param.rs")]
//! }
//! ```
//!
//!
//! ### Parsing time from result
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/parse_time_from_result.rs")]
//! }
//!
//! ```
//!
//!
//! ## DateTime
//!
//!
//! * [DateTime][date_time] captures the date and time with offset
//! * [NaiveDateTime][naive_date_time] captures the date time without offset
//! * `tuple`([NaiveDateTime][naive_date_time], String)  captures the date/time and the time zone id
//!
//! [date_time]: https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html
//! [naive_date_time]: https://docs.rs/chrono/0.4.19/chrono/struct.NaiveDateTime.html
//!
//!
//! ### DateTime as param
//!
//! Pass a DateTime as parameter to the query:
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/datetime_as_param.rs")]
//! }
//! ```
//!
//! ### Parsing DateTime from result
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/parse_datetime_from_result.rs")]
//! }
//!
//! ```
//!
//!
//!
//! ## Path
//!
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).await.unwrap();
//!
#![doc = include_str!("../include/path.rs")]
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

pub use crate::config::{Config, ConfigBuilder};
pub use crate::errors::*;
pub use crate::graph::{query, Graph};
pub use crate::query::Query;
pub use crate::row::{Map, Node, Path, Point2D, Point3D, Relation, Row, UnboundedRelation};
pub use crate::stream::RowStream;
pub use crate::txn::Txn;
pub use crate::types::serde::{EndNodeId, Id, Keys, Labels, StartNodeId, Type};
pub use crate::version::Version;
