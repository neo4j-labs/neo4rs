# Neo4j driver in rust
The driver uses the native bolt 4.1 protocol to communicate with the server. 

##Examples

```rust
    //simple query
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j".to_owned();
    let pass = "neo4j".to_owned();
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut stream = graph.query("RETURN 1").execute().await.unwrap();
    while let Some(row) = stream.next().await {
        println!("{:?}", row);
    }
```


```rust
    //create a node and return it
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph
        .query("CREATE (friend:Person {name: 'Mark'}) RETURN friend")
        .execute()
        .await
        .unwrap();
    let row = result.next().await.unwrap();
    let node: Node = row.get("friend").unwrap();
    let name: String = node.get("name").unwrap();
```

```rust
    //stream result
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
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
- [ ] explicit transactions
- [ ] batch queries/pipelining
- [ ] use buffered TCP streams
- [ ] connection pooling & multiplexing
- [ ] add support for older versions of the protocol
- [ ] Secure connection
- [ ] documentation
