use futures::stream::{self, StreamExt, TryStreamExt};
use neo4rs::{query, ConfigBuilder, Graph};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let neo4j_uri = std::env::var("NEO4J_URI").unwrap();
    let neo4j_username = std::env::var("NEO4J_USERNAME").unwrap();
    let neo4j_password = std::env::var("NEO4J_PASSWORD").unwrap();

    let graph = Graph::connect(
        ConfigBuilder::new()
            .uri(neo4j_uri)
            .user(neo4j_username)
            .password(neo4j_password)
            .max_connections(420)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    stream::iter(1..=1337)
        .map(|i| work(i, graph.clone()))
        .buffer_unordered(420)
        .map(|(i, node_count, rel_count)| {
            if i % 100 == 0 || i == 1337 {
                println!("iteration: {i}, node count: {node_count}, rel count: {rel_count}");
            }
        })
        .collect::<()>()
        .await;
}

async fn work(i: u64, graph: Graph) -> (u64, u64, u64) {
    graph
        .run(query(
            "
CREATE
  (dan:Person {name: 'Dan'}),
  (annie:Person {name: 'Annie'}),
  (matt:Person {name: 'Matt'}),
  (jeff:Person {name: 'Jeff'}),
  (brie:Person {name: 'Brie'}),
  (elsa:Person {name: 'Elsa'}),

  (cookies:Product {name: 'Cookies'}),
  (tomatoes:Product {name: 'Tomatoes'}),
  (cucumber:Product {name: 'Cucumber'}),
  (celery:Product {name: 'Celery'}),
  (kale:Product {name: 'Kale'}),
  (milk:Product {name: 'Milk'}),
  (chocolate:Product {name: 'Chocolate'}),

  (dan)-[:BUYS {amount: 1.2}]->(cookies),
  (dan)-[:BUYS {amount: 3.2}]->(milk),
  (dan)-[:BUYS {amount: 2.2}]->(chocolate),

  (annie)-[:BUYS {amount: 1.2}]->(cucumber),
  (annie)-[:BUYS {amount: 3.2}]->(milk),
  (annie)-[:BUYS {amount: 3.2}]->(tomatoes),

  (matt)-[:BUYS {amount: 3}]->(tomatoes),
  (matt)-[:BUYS {amount: 2}]->(kale),
  (matt)-[:BUYS {amount: 1}]->(cucumber),

  (jeff)-[:BUYS {amount: 3}]->(cookies),
  (jeff)-[:BUYS {amount: 2}]->(milk),

  (brie)-[:BUYS {amount: 1}]->(tomatoes),
  (brie)-[:BUYS {amount: 2}]->(milk),
  (brie)-[:BUYS {amount: 2}]->(kale),
  (brie)-[:BUYS {amount: 3}]->(cucumber),
  (brie)-[:BUYS {amount: 0.3}]->(celery),

  (elsa)-[:BUYS {amount: 3}]->(chocolate),
  (elsa)-[:BUYS {amount: 3}]->(milk)
",
        ))
        .await
        .unwrap();

    let node_count = graph
        .execute(query("MATCH (n) RETURN count(n) AS count"))
        .await
        .unwrap()
        .column_into_stream::<u64>("count")
        .try_fold(0_u64, |acc, x| async move { Ok(acc + x) })
        .await
        .unwrap();

    let rel_count = graph
        .execute(query("MATCH ()-[r]->() RETURN count(r) AS count"))
        .await
        .unwrap()
        .column_into_stream::<u64>("count")
        .try_fold(0_u64, |acc, x| async move { Ok(acc + x) })
        .await
        .unwrap();

    (i, node_count, rel_count)
}
