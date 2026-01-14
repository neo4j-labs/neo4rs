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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/example.rs")]
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
//!    let graph = Graph::connect(config).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/configurations.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/nodes.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/transactions.rs")]
//! }
//!
//! ```
//!
//! ### Streams within a transaction
//!
//! Each [`RowStream`] returned by various execute functions within the same
//! transaction are well isolated, so you can consume the stream anytime
//! within the transaction using [`RowStream::next`]
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
//!    let graph = Graph::connect(config).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/streams_within_a_transaction.rs")]
//! }
//!
//! ```
//!
//! ### Streams are evaluated lazily
//!
//! The [`RowStream`] returned by various `execute` functions need to be
//! consumed with [`RowStream::next`] in order to actually execute the
//! query.
//! The various `run` functions on the other hand are always executed
//! eagerly.
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/result_stream.rs")]
//! }
//!
//! ```
//!
#![cfg_attr(
    feature = "unstable-bolt-protocol-impl-v2",
    doc = r##"### Bookmarks and transactions

Start a new transaction using [`Graph::start_txn`], which will return a handle [`Txn`] that can
be used to [`Txn::commit`] or [`Txn::rollback`] the transaction. The commit message eventually returns
a bookmark which can be used to start a new transaction with the same state.

```no_run
use neo4rs::*;

#[tokio::main]
async fn main() {
   let uri = "127.0.0.1:7687";
   let user = "neo4j";
   let pass = "neo";
   let graph = Graph::new(uri, user, pass).unwrap();

"##
)]
#![cfg_attr(
    feature = "unstable-bolt-protocol-impl-v2",
    doc = include_snippet!("../integrationtests/tests/bookmarks.rs")
)]
#![cfg_attr(
    feature = "unstable-bolt-protocol-impl-v2",
    doc = r"
}
```

"
)]
#![cfg_attr(
    feature = "unstable-result-summary",
    doc = r##"### Streaming summary

To get access to the result summary after streaming a [`RowStream`], use the [`RowStream::finish`] method.

```no_run
use neo4rs::*;

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(uri, user, pass).unwrap();

"##
)]
#![cfg_attr(
    feature = "unstable-result-summary",
    doc = include_snippet!("../integrationtests/tests/result_summary.rs")
)]
#![cfg_attr(
    feature = "unstable-result-summary",
    doc = r"
}
```

"
)]
//! ### Rollback a transaction
//! ```no_run
//! use neo4rs::*;
//!
//! #[tokio::main]
//! async fn main() {
//!    let uri = "127.0.0.1:7687";
//!    let user = "neo4j";
//!    let pass = "neo";
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/rollback_a_transaction.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/txn_vs_graph.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/relationships.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/unbounded_relationships.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
//!    let qry = "
//!        WITH point({ x: 2.3, y: 4.5, crs: 'cartesian' }) AS p1,
//!             point({ x: 1.1, y: 5.4, crs: 'cartesian' }) AS p2
//!        RETURN point.distance(p1,p2) AS dist, p1, p2
//!     ";
#![doc = include_snippet!("../integrationtests/tests/points.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/raw_bytes.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/durations.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/dates.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/time_as_param.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/parse_time_from_result.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/datetime_as_param.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/parse_datetime_from_result.rs")]
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
//!    let graph = Graph::new(uri, user, pass).unwrap();
//!
#![doc = include_snippet!("../integrationtests/tests/path.rs")]
//! }
//! ```
//!
//!
mod auth;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
pub mod bolt;
mod bookmarks;
mod config;
mod connection;
mod convert;
mod errors;
mod graph;
mod messages;
#[cfg(feature = "unstable-serde-packstream-format")]
mod packstream;
mod pool;
mod query;
mod retry;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
mod routing;
mod row;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
mod session;
mod stream;
#[cfg(feature = "unstable-result-summary")]
pub mod summary;
mod txn;
mod types;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
mod utils;
mod version;

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
pub use {
    session::{Session, SessionConfig, SessionConfigBuilder},
    utils::ConcurrentHashMap,
};

pub use crate::auth::ClientCertificate;
pub use crate::config::{Config, ConfigBuilder, Database};
pub use crate::errors::{
    Error, Neo4jClientErrorKind, Neo4jError, Neo4jErrorKind, Neo4jSecurityErrorKind, Result,
};
pub use crate::graph::{query, Graph};
pub use crate::query::{Query, QueryParameter, RunResult};
pub use crate::row::{Node, Path, Point2D, Point3D, Relation, Row, UnboundedRelation};
pub use crate::stream::{DetachedRowStream, RowStream};
pub use crate::txn::Txn;
pub use crate::types::serde::{
    DeError, EndNodeId, Id, Indices, Keys, Labels, Nodes, Offset, Relationships, StartNodeId,
    Timezone, Type,
};
pub use crate::types::{
    BoltBoolean, BoltBytes, BoltDate, BoltDateTime, BoltDateTimeZoneId, BoltDuration, BoltFloat,
    BoltInteger, BoltList, BoltLocalDateTime, BoltLocalTime, BoltMap, BoltNode, BoltNull, BoltPath,
    BoltPoint2D, BoltPoint3D, BoltRelation, BoltString, BoltTime, BoltType, BoltUnboundedRelation,
};
pub use crate::version::Version;
pub(crate) use messages::Success;
use neo4rs_include_snippet::include_snippet;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    Read,
    Write,
}

impl Operation {
    pub fn is_read(&self) -> bool {
        matches!(self, Operation::Read)
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Read => write!(f, "READ"),
            Operation::Write => write!(f, "WRITE"),
        }
    }
}
