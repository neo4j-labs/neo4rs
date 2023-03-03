use neo4rs::*;

mod container;

#[tokio::test]
async fn configurations() {
    let config = config().db("neo4j").fetch_size(500).max_connections(10);
    let neo4j = container::Neo4jContainer::from_config(config).await;
    let graph = neo4j.graph();

    let mut result = graph.execute(query("RETURN 1")).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
    assert!(result.next().await.unwrap().is_none());
}
