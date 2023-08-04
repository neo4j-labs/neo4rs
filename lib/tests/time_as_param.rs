use neo4rs::*;

mod container;

#[tokio::test]
pub(crate) async fn time_as_param() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/time_as_param.rs");
}
