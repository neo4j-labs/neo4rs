use neo4rs::*;

mod container;

#[tokio::test]
async fn datetime_as_param() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/datetime_as_param.rs");
}
