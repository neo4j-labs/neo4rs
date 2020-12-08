# Neo4rs [![CircleCI](https://circleci.com/gh/yehohanan7/neo4rs.svg?style=shield&circle-token=6537a33de9b96ea8f26a2732b9ca6ef95ab3762b)](https://circleci.com/gh/yehohanan7/neo4rs)

Neo4rs is a native rust driver implemented using [bolt 4.1 specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)


## Getting Started


```rust    
    //Connect to server
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j";
    let pass = "neo4j";
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    assert!(graph.query("RETURN 1").run().await.is_ok());
    
    //Concurrent queries
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Arc::new(Graph::connect(uri, user, pass).await.unwrap());
    for _ in 1..=42 {
        let graph = graph.clone();
        tokio::spawn(async move {
            let mut result = graph.query("MATCH (p) RETURN p").execute().await.unwrap();
            while let Some(row) = result.next().await {
                //process row
            }
        });
    }
    
    //Create a node and process the response
    let mut result = graph
        .query("CREATE (friend:Person {name: $name}) RETURN friend")
        .param("name", "Mark")
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let id = node.id();
    let labels = node.labels();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mark");
    assert_eq!(labels, vec!["Person"]);
    
    
    //Drain the response stream
    let mut result = graph
        .query("MATCH (p:Person {name: 'Mark'}) RETURN p")
        .execute()
        .await
        .unwrap();

    while let Some(row) = result.next().await {
        let node: Node = row.get("friend").unwrap();
        let name: String = node.get("name").unwrap();
	//process name & node
    }
    
    
    //Create and parse relationship
    let mut result = graph
        .query("CREATE (p:Person { name: 'Mark' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r")
        .execute()
        .await
        .unwrap();
	
    let row = result.next().await.unwrap();
    
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
neo4rs = "0.1.1"
```

---

# Roadmap
- [x] bolt protocol
- [x] stream abstraction
- [x] query.run() vs query.execute() abstraction
- [x] respect "has_more" flag returned for PULL
- [x] connection pooling
- [ ] improve logging
- [ ] explicit transactions
- [ ] batch queries/pipelining
- [ ] add support for older versions of the protocol
- [ ] use buffered TCP streams
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
