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
