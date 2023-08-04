use neo4rs::*;

mod container;

#[tokio::test]
async fn configurations() {
    let config = ConfigBuilder::default()
        .db("neo4j")
        .fetch_size(500)
        .max_connections(10);
    let neo4j = container::Neo4jContainer::from_config(config).await;
    let graph = neo4j.graph();

    include!("../include/configurations.rs");
}
