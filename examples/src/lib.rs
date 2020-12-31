use neo4rs::*;

async fn simple_query() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(uri, user, pass).await.unwrap();
    let mut result = graph.execute(query("RETURN 1")).await.unwrap();
    while let Ok(Some(row)) = result.next().await {
        println!("{:?}", row);
    }
}

async fn transactions() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(uri, user, pass).await.unwrap();
    let txn = graph.start_txn().await.unwrap();
    let id = "some_id";
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    let mut result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    while let Ok(Some(row)) = result.next().await {
        println!("{:?}", row);
    }
    txn.commit().await.unwrap();
}
