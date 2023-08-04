use neo4rs::*;

mod container;

#[tokio::test]
async fn dates() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/dates.rs");
}
