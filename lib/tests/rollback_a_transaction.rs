use neo4rs::*;
use uuid::Uuid;

mod container;

#[tokio::test]
async fn rollback_a_transaction() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    // create a node
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    // rollback the changes
    txn.rollback().await.unwrap();

    // changes not updated in the database
    let mut result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert!(result.next().await.unwrap().is_none());
}
