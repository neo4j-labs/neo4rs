use neo4rs::*;

mod container;

#[tokio::test]
async fn streams_within_a_transaction() {
    let config = ConfigBuilder::default().fetch_size(1);
    let neo4j = container::Neo4jContainer::from_config(config).await;
    let graph = neo4j.graph();

    include!("../include/streams_within_a_transaction.rs");
}
