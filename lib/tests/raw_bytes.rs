use neo4rs::*;

mod container;

#[tokio::test]
async fn raw_bytes() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/raw_bytes.rs");
}
