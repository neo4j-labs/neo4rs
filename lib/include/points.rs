{ 
    let mut result = graph.execute(query(qry)).await.unwrap();
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
