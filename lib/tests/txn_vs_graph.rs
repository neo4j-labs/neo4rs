use neo4rs::*;

mod container;

#[tokio::test]
async fn txn_vs_graph() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/txn_vs_graph.rs");
}
