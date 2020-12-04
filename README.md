# Neo4j native driver in rust

Uses bolt 4.1 protocol to communicate with Neo4j server.


## Examples


```rust
    //run a simple query, ignore the response
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j";
    let pass = "neo4j";
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    assert!(graph.query("RETURN 1").run().await.is_ok());
```


```rust
    //simple query and consume the response stream
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j";
    let pass = "neo4j";
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut stream = graph.query("RETURN 1").execute().await.unwrap();
    while let Some(row) = stream.next().await {
        println!("{:?}", row);
    }
```


```rust
    //create a node and return it
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: $name}) RETURN friend")
        .param("name", "Mr Mark")
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let name: String = node.get("name").unwrap();
```

```rust
    //stream result
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph
        .query("MATCH (p:Person {name: 'Mark'}) RETURN p")
        .execute()
        .await
        .unwrap();

    while let Some(row) = result.next().await {
        let node: Node = row.get("friend").unwrap();
        let name: String = node.get("name").unwrap();
    }
```

# Roadmap
- [x] bolt protocol
- [x] stream abstraction
- [x] discard response stream
- [ ] explicit transactions
- [ ] batch queries/pipelining
- [ ] use buffered TCP streams
- [ ] connection pooling & multiplexing
- [ ] add support for older versions of the protocol
- [ ] Secure connection
- [ ] documentation
