use neo4rs::*;

mod container;

#[tokio::test]
async fn rollback_a_transaction() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    include!("../include/rollback_a_transaction.rs");
}
