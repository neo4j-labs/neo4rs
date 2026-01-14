use neo4rs::*;

mod container;

#[tokio::test]
async fn duration_deserialization() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let duration = std::time::Duration::new(5259600, 7);
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", duration))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let d: std::time::Duration = row.get("output").unwrap();
    assert_eq!(d, duration);

    let mut result = graph
        .execute(query("RETURN $d as output").param("d", duration))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let d = row.get::<BoltType>("output").unwrap();
    assert_eq!(
        d,
        BoltType::Duration(BoltDuration::new(
            0.into(),
            0.into(),
            5259600.into(),
            7.into(),
        ))
    );
}
