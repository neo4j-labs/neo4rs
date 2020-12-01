# native neo4j client

```rust
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j".to_owned();
    let pass = "neo4j".to_owned();
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut stream = graph.query("RETURN 1").execute().await.unwrap();
    while let Some(row) = stream.next().await {
        println!("{:?}", row);
    }
```

# Roadmap
- [x] bolt protocol
- [x] stream abstraction
- [ ] Secure connection
- [ ] use buffered TCP streams
- [ ] connection pooling & multiplexing
- [ ] documentation
