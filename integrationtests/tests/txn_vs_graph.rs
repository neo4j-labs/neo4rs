use neo4rs::*;

mod container;

#[tokio::test]
async fn txn_vs_graph() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    // snippet-start
    let mut txn = graph.start_txn().await.unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    // graph.execute(..) will not see the changes done above as the txn is not committed yet
    let mut result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert!(result.next().await.unwrap().is_none());
    txn.commit().await.unwrap();

    //changes are now seen as the transaction is committed.
    let mut result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert!(result.next().await.unwrap().is_some());
    assert!(result.next().await.unwrap().is_some());
    assert!(result.next().await.unwrap().is_none());
    // snippet-end
}
