use neo4rs::*;

mod container;

#[tokio::test]
async fn basic_example() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/example.rs");
}
