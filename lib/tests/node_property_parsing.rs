use chrono::{DateTime, FixedOffset};
use neo4rs::{query, Node};

mod container;

#[tokio::test]
async fn node_property_parsing() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    graph
        .run(query("CREATE (:A {p1:DATETIME('2024-12-31T08:10:35')})"))
        .await
        .unwrap();

    let mut result = graph.execute(query("MATCH (p:A) RETURN p")).await.unwrap();

    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("p").unwrap();
        let p1 = node.get::<DateTime<FixedOffset>>("p1").unwrap();
        assert_eq!(p1.timestamp(), 1735632635);
    }
}
