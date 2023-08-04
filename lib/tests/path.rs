use neo4rs::*;

mod container;

#[tokio::test]
pub async fn path() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/path.rs");
}
