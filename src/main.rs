use futures::stream::StreamExt;
use neo4rs::*;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let tasks = args.get(1).map(|i| i.parse::<i32>().unwrap()).unwrap_or(5);
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Arc::new(Graph::new(uri, user, pass).await.unwrap());
    let mut handles = Vec::new();
    for _ in 1..=tasks {
        let graph = graph.clone();
        let handle = tokio::spawn(async move {
            let mut result = graph.execute(query("MATCH (p) RETURN p")).await.unwrap();
            let mut count = 0;
            while let Some(_) = result.next().await {
                count += 1;
            }
        });
        handles.push(handle);
    }

    futures::future::join_all(handles).await;
}

//#[tokio::main]
//async fn main() {
//    let uri = "127.0.0.1:7687";
//    let user = "neo4j";
//    let pass = "neo";
//    let graph = Arc::new(Graph::new(uri, user, pass).await.unwrap());
//    let mut txn = graph.start_txn().await.unwrap();
//    txn.run_queries(vec![
//        query("CREATE (p:Person:Txn1)"),
//        query("CREATE (p:Person:Txn2)"),
//    ])
//    .await
//    .unwrap();
//    txn.commit().await.unwrap();
//}
