# Neo4rs  [![CI Status][ci-badge]][ci-url]  [![Crates.io][crates-badge]][crates-url]

[ci-badge]: https://circleci.com/gh/yehohanan7/neo4rs.svg?style=shield&circle-token=6537a33de9b96ea8f26a2732b9ca6ef95ab3762b
[ci-url]: https://github.com/yehohanan7/neo4rs
[crates-badge]: https://img.shields.io/crates/v/neo4rs.svg?style=shield
[crates-url]: https://crates.io/crates/neo4rs

Neo4rs is a native rust driver implemented using [bolt 4.1 specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)

This driver is compatible with neo4j 4.x versions

## Getting Started


```rust    

    //Simple query
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(&uri, user, pass).await.unwrap();
    assert!(graph.run(query("RETURN 1")).await.is_ok());
    
    //Connect using configuration
    let config = config()
        .uri("127.0.0.1:7687")
        .user("neo4j")
        .password("neo")
        .db("neo4j")
        .fetch_size(500)
	.max_connections(15)
        .build()
        .unwrap();
    let graph = Graph::connect(config).await.unwrap();
    assert!(graph.run(query("RETURN 1")).await.is_ok());
    
    //Concurrent queries
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Arc::new(Graph::new(&uri, user, pass).await.unwrap());
    for _ in 1..=42 {
        let graph = graph.clone();
        tokio::spawn(async move {
            let mut result = graph.execute(
	       query("MATCH (p:Person {name: $name}) RETURN p").param("name", "Mark")
	    ).await.unwrap();
            while let Ok(Some(row)) = result.next().await {
        	let node: Node = row.get("p").unwrap();
        	let name: String = node.get("name").unwrap();
                println!("{}", name);
            }
        });
    }
    
    //Transactions
    let mut txn = graph.start_txn().await.unwrap();
    txn.run_queries(vec![
        query("CREATE (p:Person {name: 'mark'})"),
        query("CREATE (p:Person {name: 'jake'})"),
        query("CREATE (p:Person {name: 'luke'})"),
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap(); //or txn.rollback().await.unwrap();
    
    
    //Create and parse relationship
    let mut result = graph
        .execute(query("CREATE (p:Person { name: 'Mark' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r"))
        .await
        .unwrap();
	
    let row = result.next().await.unwrap().unwrap();
    
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > 0);
    assert!(relation.start_node_id() > 0);
    assert!(relation.end_node_id() > 0);
    assert_eq!(relation.typ(), "WORKS_AT");
    assert_eq!(relation.get::<String>("as").unwrap(), "Engineer");
    
    //Work with raw bytes
    let mut result = graph
        .execute(query("RETURN $b as output").param("b", vec![11, 12]))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let b: Vec<u8> = row.get("output").unwrap();
    assert_eq!(b, &[11, 12]);
    
    //Date
    let date = chrono::NaiveDate::from_ymd(1985, 2, 5);
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", date))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let d: chrono::NaiveDate = row.get("output").unwrap();
    assert_eq!(d.to_string(), "1985-02-05");
    
    //Duration
    let duration = std::time::Duration::new(5259600, 7);
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", duration))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let d: Duration = row.get("output").unwrap();
    assert_eq!(d.as_secs(), 5259600);
    assert_eq!(d.subsec_nanos(), 7);
    
    //Time without timezone
    let date = chrono::NaiveTime::from_hms_nano(10, 15, 30, 200);
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", date))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: (chrono::NaiveTime, Option<chrono::FixedOffset>) = row.get("output").unwrap();
    assert_eq!(t.0.to_string(), "10:15:30.000000200");
    assert_eq!(t.1, None);
    
    
    //Time with timezone offset
    let time = chrono::NaiveTime::from_hms_nano(11, 15, 30, 200);
    let offset = chrono::FixedOffset::east(3 * 3600);
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", (time, offset)))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: (chrono::NaiveTime, Option<chrono::FixedOffset>) = row.get("output").unwrap();
    assert_eq!(t.0.to_string(), "11:15:30.000000200");
    assert_eq!(t.1, Some(offset));
    
    
    //Work with points
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
    
    //Work with paths
    let mut result = graph
        .execute(
            query("MATCH p = (person:Person { name: $name })-[r:WORKS_AT]->(c:Company) RETURN p")
                .param("name", name),
        )
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let path: Path = row.get("p").unwrap();
    assert_eq!(path.ids().len(), 2);
    assert_eq!(path.nodes().len(), 2);
    assert_eq!(path.rels().len(), 1);
    
```


---

# Roadmap
- [x] bolt protocol
- [x] stream abstraction
- [x] query.run() vs query.execute() abstraction
- [x] respect "has_more" flag returned for PULL
- [x] connection pooling
- [x] explicit transactions
- [x] use buffered TCP streams
- [x] improve error messages & logging
- [x] fetch rows in blocks
- [x] configureable fetch size
- [x] multi db support
- [x] multi version support
- [x] support data types
	- [x] Float
	- [x] Bytes
- [ ] support structures
	- [x] Relationship
	- [x] Point2D
	- [x] Point3D
	- [x] UnboundedRelationship
	- [x] Path
	- [x] Duration
	- [x] Date
	- [x] Time
	- [x] LocalTime
	- [ ] DateTime
	- [ ] DateTimeZoneId
	- [ ] LocalDateTime

## License

Neo4rs is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
