use neo4rs::*;

mod container;

#[tokio::test]
async fn points() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let distance = if neo4j.version().major >= 5 {
        "point.distance(p1,p2)"
    } else {
        "distance(p1,p2)"
    };

    let mut result = graph
        .execute(query(&format!(
            "WITH point({{ x: 2.3, y: 4.5, crs: 'cartesian' }}) AS p1,
point({{ x: 1.1, y: 5.4, crs: 'cartesian' }}) AS p2 RETURN {distance} AS dist, p1, p2",
            distance = distance
        )))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let dist: f64 = row.get("dist").unwrap();
    let p1: Point2D = row.get("p1").unwrap();
    let p2: Point2D = row.get("p2").unwrap();
    assert_eq!(1.5, dist);
    assert_eq!(p1.sr_id(), 7203);
    assert_eq!(p1.x(), 2.3);
    assert_eq!(p1.y(), 4.5);
    assert_eq!(p2.sr_id(), 7203);
    assert_eq!(p2.x(), 1.1);
    assert_eq!(p2.y(), 5.4);
    assert!(result.next().await.unwrap().is_none());

    let mut result = graph
        .execute(query(
            "RETURN point({ longitude: 56.7, latitude: 12.78, height: 8 }) AS point",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let point: Point3D = row.get("point").unwrap();
    assert_eq!(point.sr_id(), 4979);
    assert_eq!(point.x(), 56.7);
    assert_eq!(point.y(), 12.78);
    assert_eq!(point.z(), 8.0);
    assert!(result.next().await.unwrap().is_none());
}
