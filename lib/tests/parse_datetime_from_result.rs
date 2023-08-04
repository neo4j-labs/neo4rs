use neo4rs::*;

mod container;

#[tokio::test]
async fn parse_datetime_from_result() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/parse_datetime_from_result.rs");
}
