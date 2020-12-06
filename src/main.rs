use futures::stream::StreamExt;
use neo4rs::*;

#[tokio::main]
async fn main() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let mut graph = Graph::connect(uri, user, pass).await.unwrap();

    graph
        .query("CREATE (p:Person {name: $name, age: $age}) RETURN p.name, p.age")
        .param("name", "Mr Main")
        .param("age", 37)
        .run()
        .await
        .unwrap();

    let mut result = graph
        .query("MATCH (p:Person) WHERE p.age > 35 RETURN p.name, p.age")
        .execute()
        .await
        .unwrap();

    while let Some(row) = result.next().await {
        println!("========================");
        let name: String = row.get("p.name").unwrap();
        let age: i64 = row.get("p.age").unwrap();
        println!("{:?}, {:?}", name, age);
        println!("========================");
    }
}
