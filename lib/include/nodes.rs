{ 
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
        let labels = node.labels();
        let keys = node.keys();
        let name: String = node.get("name").unwrap();
        assert_eq!(name, "Mr Mark");
        assert_eq!(labels, vec!["Person"]);
        assert_eq!(keys, vec!["name"]);
        assert!(id >= 0);
    }
}
