#![cfg(feature = "unstable-result-summary")]
use neo4rs::*;

mod container;

#[tokio::test]
async fn streaming_summary() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/result_summary.rs");
}
