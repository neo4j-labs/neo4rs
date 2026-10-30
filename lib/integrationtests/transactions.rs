use neo4rs::*;
use std::time::Duration;

mod container;

#[tokio::test]
async fn transactions() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    // snippet-start
    let mut txn = graph.start_txn().await.unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let result = txn
        .run_queries([
            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
        ])
        .await;

    assert!(result.is_ok());
    txn.commit().await.unwrap();

    let mut txn = graph
        .start_txn_with_config(
            TransactionConfig::default()
                .with_tx_metadata(vec![("operation".into(), "example".into())])
                .with_tx_timeout(Duration::from_secs(15)),
        )
        .await
        .unwrap();
    txn.rollback().await.unwrap();

    let mut result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert!(result.next().await.unwrap().is_some());
    assert!(result.next().await.unwrap().is_some());
    assert!(result.next().await.unwrap().is_none());
    // snippet-end
}

#[tokio::test]
async fn transaction_metadata_is_visible_to_the_server() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();
    let request_id = uuid::Uuid::new_v4().to_string();
    let config = TransactionConfig::default()
        .with_tx_timeout(Duration::from_secs(30))
        .with_tx_metadata(vec![
            ("operation".into(), "transaction-metadata-test".into()),
            ("request_id".into(), request_id.clone()),
        ]);
    let mut txn = graph.start_txn_with_config(config).await.unwrap();
    txn.run("RETURN 1").await.unwrap();

    let mut result = graph
        .execute(
            query(
                "SHOW TRANSACTIONS YIELD metaData \
                 WHERE metaData.request_id = $request_id \
                 RETURN count(*) AS count",
            )
            .param("request_id", request_id),
        )
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("count").unwrap(), 1);
    txn.rollback().await.unwrap();
}
