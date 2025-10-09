use chrono::{DateTime, FixedOffset};
use neo4rs::{Node, Point2D, Point3D};

mod container;

#[tokio::test]
async fn node_property_parsing() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    graph
        .run(
            "CREATE
(:Datetime {p1:DATETIME('2024-12-31T08:10:35')}),
(:Point2D {a:Point ({x:2,y:3})}),
(:Point3D {a:Point ({x:3,y:4,z:5})})
",
        )
        .await
        .unwrap();

    let mut result = graph.execute("MATCH (p:DateTime) RETURN p").await.unwrap();

    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("p").unwrap();
        let p1 = node.get::<DateTime<FixedOffset>>("p1").unwrap();
        assert_eq!(p1.timestamp(), 1735632635);
    }

    let mut result = graph.execute("MATCH (p:Point2D) RETURN p").await.unwrap();

    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("p").unwrap();
        let p1 = node.get::<Point2D>("a").unwrap();
        assert_eq!(p1.x(), 2.0);
        assert_eq!(p1.y(), 3.0);
    }

    let mut result = graph.execute("MATCH (p:Point3D) RETURN p").await.unwrap();

    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("p").unwrap();
        let p1 = node.get::<Point3D>("a").unwrap();
        assert_eq!(p1.x(), 3.0);
        assert_eq!(p1.y(), 4.0);
        assert_eq!(p1.z(), 5.0);
    }
}
