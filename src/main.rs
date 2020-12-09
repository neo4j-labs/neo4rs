use futures::stream::StreamExt;
use neo4rs::*;
use std::sync::Arc;

//#[tokio::main]
//async fn main() {
//    let uri = "127.0.0.1:7687";
//    let user = "neo4j";
//    let pass = "neo";
//    let graph = Graph::connect(uri, user, pass).await.unwrap();
//    let mut result = graph.query("MATCH (p) RETURN p").execute().await.unwrap();
//    //graph
//    //    .query("CREATE (p:Person {name: $name, age: $age}) RETURN p.name, p.age")
//    //    .param("name", "Mr Main")
//    //    .param("age", 37)
//    //    .run()
//    //    .await
//    //    .unwrap();
//
//    //let mut result = graph.query("MATCH (p) RETURN p").execute().await.unwrap();
//    let mut count = 0;
//    while let Some(row) = result.next().await {
//        println!("{:?}", row);
//        count += 1;
//    }
//    println!("{:?}", count);
//}
//
//

//#[tokio::main]
//async fn main() {
//    let uri = "127.0.0.1:7687";
//    let user = "neo4j";
//    let pass = "neo";
//    let graph = Arc::new(Graph::connect(uri, user, pass).await.unwrap());
//    let mut handles = Vec::new();
//    for _ in 1..=1000 {
//        let graph = graph.clone();
//        let handle = tokio::spawn(async move {
//            let mut result = graph.query("MATCH (p) RETURN p").execute().await.unwrap();
//            let mut count = 0;
//            while let Some(_) = result.next().await {
//                count += 1;
//            }
//            println!("{:?}", count);
//        });
//        handles.push(handle);
//    }
//    futures::future::join_all(handles).await;
//}
//
//

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let neo = Arc::new(Neo4rs::new(uri, user, pass).await.unwrap());
    let mut handles = Vec::new();
    for _ in 1..=5 {
        let neo = neo.clone();
        let handle = tokio::spawn(async move {
            let graph = neo.connect().await.unwrap();
            let query = graph.query("MATCH (p) RETURN p");
            let mut result = query.execute().await.unwrap();
            let mut count = 0;
            while let Some(_) = result.next().await {
                count += 1;
            }
            println!("{:?}", count);
        });
        handles.push(handle);
    }

    futures::future::join_all(handles).await;
}
