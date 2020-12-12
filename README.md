# Neo4rs [![CircleCI](https://circleci.com/gh/yehohanan7/neo4rs.svg?style=shield&circle-token=6537a33de9b96ea8f26a2732b9ca6ef95ab3762b)](https://circleci.com/gh/yehohanan7/neo4rs)

Neo4rs is a native rust driver implemented using [bolt 4.1 specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)


## Getting Started


```rust    
    //Run a query
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(&uri, user, pass).await.unwrap();
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
```



## Installation
neo4rs is available on [crates.io](https://crates.io/crates/neo4rs) and can be included in your Cargo enabled project like this:

```toml
[dependencies]
neo4rs = "0.2.5"
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
- [ ] query & transaction macros/dsl
- [ ] add support for older versions of the protocol
- [ ] multi db support
- [ ] support data types
	- [ ] Float
	- [ ] Bytes
- [ ] support structures
	- [X] Relationship
	- [ ] UnboundedRelationship
	- [ ] Path
	- [ ] Date
	- [ ] Time
	- [ ] LocalTime
	- [ ] DateTime
	- [ ] DateTimeZoneId
	- [ ] LocalDateTime
	- [ ] Duration
	- [ ] Point2D
	- [ ] Point3D

## License

Neo4rs is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
