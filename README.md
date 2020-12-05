# Neo4rs [![CircleCI](https://circleci.com/gh/yehohanan7/neo4rs.svg?style=shield&circle-token=6537a33de9b96ea8f26a2732b9ca6ef95ab3762b)](https://circleci.com/gh/yehohanan7/neo4rs)

Neo4rs is a native rust driver implemented using [bolt 4.1 specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)


## Getting Started

*Run a simple query, discard the response data*

```rust    
    let uri = "127.0.0.1:7687".to_owned();
    let user = "neo4j";
    let pass = "neo4j";
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    assert!(graph.query("RETURN 1").run().await.is_ok());
```


*Create a node and process the response*
    
```rust
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


*Drain the result stream*
 
```rust
   
    let graph = Graph::connect(uri, user, pass).await.unwrap();
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
```


*Create explicit transactions*

```rust
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    let txn = graph.begin_txn().await.unwrap();
    graph.query("CREATE (p:Person {id: 'some_id'})") .run() .await .unwrap();
    txn.commit().await.unwrap();
    
    //Rollback a transaction
    txn.rollback().await.unwrap();
```


## Installation
neo4rs is available on [crates.io](https://crates.io/crates/neo4rs) and can be included in your Cargo enabled project like this:

```toml
[dependencies]
neo4rs = "0.1.0"
```

---

# Roadmap
- [x] bolt protocol
- [x] stream abstraction
- [x] explicit transactions
- [ ] use buffered TCP streams
- [ ] connection pooling & multiplexing
- [ ] multi db support
- [ ] batch queries/pipelining
- [ ] add support for older versions of the protocol
- [ ] Secure connection
- [ ] documentation
