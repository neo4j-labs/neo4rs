use futures::stream::StreamExt;
use neo4rs::*;

#[tokio::test]
async fn should_connect() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    assert_eq!(graph.version, Version::v4_1);
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
async fn should_run_a_simple_query() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph.query("RETURN 1").execute().await.unwrap();
    let row = result.next().await.unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_create_new_node() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: 'Mark'})")
        .execute()
        .await
        .unwrap();
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn should_return_created_node() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: 'Mark'}) RETURN friend")
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mark");
}
