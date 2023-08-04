{ 
    let mut result = graph.execute(
        query("MERGE (p1:Person { name: 'Oliver Stone' })-[r:RELATED {as: 'friend'}]-(p2: Person {name: 'Mark'}) RETURN r")
    ).await.unwrap();
    let row = result.next().await.unwrap().unwrap();

    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "RELATED");
    assert_eq!(relation.keys(), vec!["as"]);
    assert_eq!(relation.get::<String>("as").unwrap(), "friend");
}
