{ 
    let mut result = graph.execute(
        query("MERGE (p1:Person { name: 'Oliver Stone' })-[r:RELATED {as: 'friend'}]-(p2: Person {name: 'Mark'}) RETURN r")
    ).await.unwrap();
    let row = result.next().await.unwrap().unwrap();

    #[derive(serde::Deserialize)]
    struct Related {
        id: Id,
        start_node_id: StartNodeId,
        end_node_id: EndNodeId,
        typ: Type,
        keys: Keys<Vec<String>>,
        #[serde(rename = "as")]
        related_as: String,
    }

    // use serde to extract the relationship data
    let relation: Related = row.get("r").unwrap();

    // The following checks are always true, but are included here
    // to demonstrate the types of the fields.
    #[allow(clippy::absurd_extreme_comparisons, unused_comparisons)]
    {
        assert!(relation.id.0 >= 0);
        assert!(relation.start_node_id.0 >= 0);
        assert!(relation.end_node_id.0 >= 0);
    }

    assert_eq!(relation.typ.0, "RELATED");
    assert_eq!(relation.keys.0, vec!["as"]);
    assert_eq!(relation.related_as, "friend");

    // or use the neo4rs::Relation type
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "RELATED");
    assert_eq!(relation.keys(), vec!["as"]);
    assert_eq!(relation.get::<String>("as").unwrap(), "friend");
}
