#![cfg(feature = "unstable-bolt-protocol-impl-v2")]
use neo4rs::*;

mod container;

#[tokio::test]
async fn transactions() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/bookmarks.rs");
}
