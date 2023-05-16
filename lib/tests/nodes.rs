use std::vec;

use neo4rs::*;

mod container;

#[tokio::test]
async fn nodes() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    assert!(graph.run(query("RETURN 1")).await.is_ok());

    let mut result = graph
        .execute(
            query("CREATE (friend:Person {name: $name}) RETURN friend").param("name", "Mr Mark"),
        )
        .await
        .unwrap();

    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("friend").unwrap();
        let id = node.id();
        let keys = node.keys();
        let labels = node.labels();
        let name: String = node.get("name").unwrap();
        assert_eq!(name, "Mr Mark");
        assert_eq!(labels, vec!["Person"]);
        assert_eq!(keys, vec![String::from("name")]);
        assert!(id >= 0);
    }
}
