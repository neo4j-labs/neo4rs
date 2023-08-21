{ 
    assert!(graph.run(query("RETURN 1")).await.is_ok());

    let mut result = graph
        .execute(
            query("CREATE (friend:Person {name: $name}) RETURN friend").param("name", "Mr Mark"),
        )
        .await
        .unwrap();

    #[derive(serde::Deserialize)]
    struct Person {
        labels: Labels,
        keys: Keys<Vec<String>>,
        name: String,
    }

    while let Ok(Some(row)) = result.next().await {
        // use serde to extract the relationship data
        let friend: Person = row.get("friend").unwrap();
        assert_eq!(friend.name, "Mr Mark");
        assert_eq!(friend.labels.0, vec!["Person"]);
        assert_eq!(friend.keys.0, vec!["name"]);

        // or use the neo4rs::Relation type
        let node: Node = row.get("friend").unwrap();
        assert_eq!(node.get::<String>("name").unwrap(), "Mr Mark");
        assert_eq!(node.labels(), vec!["Person"]);
        assert_eq!(node.keys(), vec!["name"]);
        assert!(node.id() >= 0);
    }
}
