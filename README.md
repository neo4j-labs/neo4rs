# Neo4rs [![CI Status][ci-badge]][ci-url]  [![Crates.io][crates-badge]][crates-url]

[ci-badge]: https://circleci.com/gh/yehohanan7/neo4rs.svg?style=shield&circle-token=6537a33de9b96ea8f26a2732b9ca6ef95ab3762b
[ci-url]: https://github.com/yehohanan7/neo4rs
[crates-badge]: https://img.shields.io/crates/v/neo4rs.svg?style=shield
[crates-url]: https://crates.io/crates/neo4rs
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=shield
[docs-url]: https://docs.rs/neo4rs

Neo4rs is a Neo4j rust driver implemented using [bolt specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)

This driver is compatible with neo4j 4.x versions

## API Documentation: [![Docs.rs][docs-badge]][docs-url]

## Example

```rust    
use neo4rs::{Graph, Node, query};
use std::sync::Arc;

#[tokio::main]
async fn main() {
  // concurrent queries
  let uri = "127.0.0.1:7687";
  let user = "neo4j";
  let pass = "neo";
  let graph = Arc::new(Graph::new(uri, user, pass).await.unwrap());
  let q = graph.clone();
  let mut tasks = std::vec::Vec::new();

  tasks.push(tokio::spawn(async move {
    loop {
      let mut result = q.execute(
        query("MATCH (p:Person {name: $name}) RETURN p").param("name", "mark"))
        .await.unwrap();

      tokio::select! {
        Ok(maybe_row) = result.next() => {
          match maybe_row {
            Some(row) => {
              let node: Node = row.get("p").unwrap();
              let name: String = node.get("name").unwrap();
              println!("Found {} in the graph", name);
              return;
            },
            None      => println!("Waiting for mark to be added to the graph")
          }
        },
      }
    }
  }));

  //Transactions
  let txn = graph.start_txn().await.unwrap();

  txn.run_queries(vec![
    query("CREATE (p:Person {name: 'mark'})"),
    query("CREATE (p:Person {name: 'jake'})"),
    query("CREATE (p:Person {name: 'luke'})"),
  ])
  .await
  .unwrap();

  txn.commit().await.unwrap(); //or txn.rollback().await.unwrap();

  futures::future::join_all(tasks).await;
}
```


## License

Neo4rs is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
