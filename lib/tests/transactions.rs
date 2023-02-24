use neo4rs::*;
use uuid::Uuid;

mod container;

#[tokio::test]
async fn transactions() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    let result = txn
        .run_queries(vec![
            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
        ])
        .await;

    assert!(result.is_ok());
    txn.commit().await.unwrap();
    let mut result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert!(result.next().await.unwrap().is_some());
    assert!(result.next().await.unwrap().is_some());
    assert!(result.next().await.unwrap().is_none());
}
