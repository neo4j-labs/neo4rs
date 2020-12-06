use futures::stream::StreamExt;
use neo4rs::*;
use uuid::Uuid;

async fn connect() -> Result<Graph> {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    Graph::connect(uri, user, pass).await
}

#[tokio::test]
async fn should_connect() {
    assert_eq!(connect().await.unwrap().version, Version::v4_1);
}

#[tokio::test]
async fn should_identify_invalid_credentials() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "invalid_pass";

    let error = Graph::connect(uri, user, pass).await.expect_err("error");

    match error {
        Error::AuthenticationError { detail } => assert_eq!(
            detail,
            "The client is unauthorized due to authentication failure.".to_owned()
        ),
        _ => assert!(false),
    }
}

#[tokio::test]
async fn should_execute_a_simple_query() {
    let graph = connect().await.unwrap();
    let mut result = graph.query("RETURN 1").execute().await.unwrap();
    let row = result.next().await.unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_create_new_node() {
    let graph = connect().await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: 'Mark'})")
        .execute()
        .await
        .unwrap();
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_return_created_node() {
    let graph = connect().await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: 'Mark'}) RETURN friend")
        .execute()
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
    let graph = connect().await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: $name}) RETURN friend")
        .param("name", "Mr Mark")
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mr Mark");
}

#[tokio::test]
async fn should_run_a_simple_query() {
    let graph = connect().await.unwrap();
    assert!(graph.query("RETURN 1").run().await.is_ok());
}

#[tokio::test]
async fn should_commit_txn() {
    let graph = connect().await.unwrap();
    let txn = graph.begin_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    assert!(graph
        .query("CREATE (p:Person {id: $id}) RETURN p")
        .param("id", id.clone())
        .run()
        .await
        .is_ok());
    txn.commit().await.unwrap();
    let mut result = graph
        .query("MATCH (p:Person) WHERE p.id = $id RETURN p.id")
        .param("id", id.clone())
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let actual_id: String = row.get("p.id").unwrap();
    assert_eq!(actual_id, id);
}

#[tokio::test]
async fn should_rollback_txn() {
    let graph = connect().await.unwrap();
    let txn = graph.begin_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    assert!(graph
        .query("CREATE (p:Person {id: $id}) RETURN p")
        .param("id", id.clone())
        .run()
        .await
        .is_ok());
    txn.rollback().await.unwrap();
    let mut result = graph
        .query("MATCH (p:Person) WHERE p.id = $id RETURN p.id")
        .param("id", id.clone())
        .execute()
        .await
        .unwrap();
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_create_bounded_relation() {
    let graph = connect().await.unwrap();
    let mut result = graph
        .query("CREATE (p:Person { name: 'Oliver Stone' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r")
        .execute()
        .await
        .unwrap();
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
    let graph = connect().await.unwrap();
    let mut result = graph
        .query("MERGE (p1:Person { name: 'Oliver Stone' })-[r:RELATED {as: 'friend'}]-(p2: Person {name: 'Mark'}) RETURN r")
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "RELATED");
    assert_eq!(relation.get::<String>("as").unwrap(), "friend");
}
