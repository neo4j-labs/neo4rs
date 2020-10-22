use futures::stream::StreamExt;
use neo4rs::*;

#[tokio::test]
async fn should_connect() {
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j".to_owned();
    let pass = "neo4j".to_owned();
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    assert_eq!(graph.version, Version::v4_1);
}

#[tokio::test]
async fn should_identify_invalid_credentials() {
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j".to_owned();
    let pass = "invalid_pass".to_owned();

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
async fn should_run_a_query() {
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j".to_owned();
    let pass = "neo4j".to_owned();
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut stream = graph.query("RETURN 1").execute().await.unwrap();
    while let Some(row) = stream.next().await {
        println!("{:?}", row);
    }
    assert_eq!(graph.version, Version::v4_1);
}
