{ 
    let duration = std::time::Duration::new(5259600, 7);
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", duration))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let d: std::time::Duration = row.get("output").unwrap();
    assert_eq!(d.as_secs(), 5259600);
    assert_eq!(d.subsec_nanos(), 7);
    assert!(result.next().await.unwrap().is_none());
}
