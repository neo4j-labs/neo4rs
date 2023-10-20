{ 
    let name = uuid::Uuid::new_v4().to_string();
    graph
        .run(
            query("CREATE (p:Person { name: $name })-[r:WORKS_AT]->(n:Company { name: 'Neo'})")
                .param("name", name.clone()),
        )
        .await
        .unwrap();

    let mut result = graph
        .execute(
            query("MATCH p = (person:Person { name: $name })-[r:WORKS_AT]->(c:Company) RETURN p")
                .param("name", name),
        )
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let path: Path = row.get("p").unwrap();
    assert_eq!(path.indices().len(), 2);
    assert_eq!(path.nodes().len(), 2);
    assert_eq!(path.rels().len(), 1);
    assert!(result.next().await.unwrap().is_none());
}
