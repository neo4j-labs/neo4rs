use neo4rs::*;

mod container;

// The purpose of the test is to not use a `must_use`
#[allow(unused_must_use)]
#[tokio::test]
async fn result_stream() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/result_stream.rs");
}
