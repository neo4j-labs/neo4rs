{ 
    let bytes = b"Hello, Neo4j!";
    let mut result = graph
        .execute(query("RETURN $b as output").param("b", bytes.as_ref()))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let b: Vec<u8> = row.get("output").unwrap();
    assert_eq!(b, bytes);
    assert!(result.next().await.unwrap().is_none());
}
