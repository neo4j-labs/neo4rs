use futures::stream::StreamExt;
use neo4rs::*;

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::connect(uri, user, pass).await.unwrap();
    let mut result = graph.query("MATCH (p) RETURN p").execute().await.unwrap();
    //graph
    //    .query("CREATE (p:Person {name: $name, age: $age}) RETURN p.name, p.age")
    //    .param("name", "Mr Main")
    //    .param("age", 37)
    //    .run()
    //    .await
    //    .unwrap();

    //let mut result = graph.query("MATCH (p) RETURN p").execute().await.unwrap();
    let mut count = 0;
    while let Some(row) = result.next().await {
        println!("{:?}", row);
        count += 1;
    }
    println!("{:?}", count);
}
