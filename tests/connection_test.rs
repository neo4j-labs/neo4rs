use futures::stream::StreamExt;
use neo4rs::*;
use uuid::Uuid;

async fn graph() -> Graph {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    Graph::new(uri, user, pass).await.unwrap()
}

#[tokio::test]
async fn should_connect() {
    assert_eq!(graph().await.version().await.unwrap(), Version::v4_1);
}

#[tokio::test]
async fn should_execute_a_simple_query() {
    let graph = graph().await;
    let mut result = graph.execute(query("RETURN 1")).await.unwrap();
    let row = result.next().await.unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_create_new_node() {
    let graph = graph().await;
    let mut result = graph
        .execute(query("CREATE (friend:Person {name: 'Mark'})"))
        .await
        .unwrap();
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_return_created_node() {
    let graph = graph().await;
    let mut result = graph
        .execute(query("CREATE (friend:Person {name: 'Mark'}) RETURN friend"))
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let id = node.id();
    let labels = node.labels();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mark");
    assert_eq!(labels, vec!["Person"]);
    assert!(id > 0);
}

#[tokio::test]
async fn should_execute_query_with_params() {
    let graph = graph().await;
    let mut result = graph
        .execute(
            query("CREATE (friend:Person {name: $name}) RETURN friend").param("name", "Mr Mark"),
        )
        .await
        .unwrap();

    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mr Mark");
}

#[tokio::test]
async fn should_run_a_simple_query() {
    let graph = graph().await;
    assert!(graph.run(query("RETURN 1")).await.is_ok());
}

#[tokio::test]
async fn should_create_bounded_relation() {
    let graph = graph().await;
    let mut result = graph.execute(
        query("CREATE (p:Person { name: 'Oliver Stone' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r")
    ).await.unwrap();
    let row = result.next().await.unwrap();
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "WORKS_AT");
    assert_eq!(relation.get::<String>("as").unwrap(), "Engineer");
}

#[tokio::test]
async fn should_create_unbounded_relation() {
    let graph = graph().await;
    let mut result = graph.execute(
        query("MERGE (p1:Person { name: 'Oliver Stone' })-[r:RELATED {as: 'friend'}]-(p2: Person {name: 'Mark'}) RETURN r")
    ).await.unwrap();
    let row = result.next().await.unwrap();
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "RELATED");
    assert_eq!(relation.get::<String>("as").unwrap(), "friend");
}

#[tokio::test]
async fn should_run_all_queries_in_txn() {
    let graph = graph().await;
    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    assert!(txn
        .run_queries(vec![
            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
            query("CREATE (p:Person {id: $id})").param("id", id.clone())
        ])
        .await
        .is_ok());
    txn.commit().await.unwrap();
    let result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert_eq!(count_rows(result).await, 2);
}

#[tokio::test]
async fn should_queries_within_txn() {
    let graph = graph().await;
    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    let create_query = query("CREATE (p:Person {id: $id})").param("id", id.clone());
    let match_query =
        query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone());
    txn.run(create_query.clone()).await.unwrap();
    txn.run(create_query).await.unwrap();
    let result = graph.execute(match_query).await.unwrap();
    assert_eq!(count_rows(result).await, 2);
    txn.commit().await.unwrap();
}

#[tokio::test]
async fn should_rollback_txn() {
    let graph = graph().await;
    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    let create_query = query("CREATE (p:Person {id: $id})").param("id", id.clone());
    let match_query =
        query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone());
    txn.run(create_query.clone()).await.unwrap();
    txn.run(create_query).await.unwrap();
    let result = graph.execute(match_query.clone()).await.unwrap();
    assert_eq!(count_rows(result).await, 2);
    txn.rollback().await.unwrap();
    let mut result = graph.execute(match_query).await.unwrap();
    assert!(result.next().await.is_none());
}

async fn count_rows(mut rx: tokio::sync::mpsc::Receiver<Row>) -> usize {
    let mut count = 0;
    while let Some(_) = rx.next().await {
        count += 1;
    }
    count
}
