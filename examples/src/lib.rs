use neo4rs::*;

async fn configurations() {
    let config = config()
        .uri("127.0.0.1:7687")
        .user("neo4j")
        .password("neo")
        .db("neo4j")
        .fetch_size(500)
        .build()
        .unwrap();
    let graph = Graph::connect(config).await.unwrap();
}

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

async fn process_nodes() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(uri, user, pass).await.unwrap();
    let mut result = graph
        .execute(query("CREATE (friend:Person {name: 'Mark'}) RETURN friend"))
        .await
        .unwrap();

    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("friend").unwrap();
        let id = node.id();
        let labels = node.labels();
        let name: String = node.get("name").unwrap();
        println!("{:?}, {:?}, {:?}", id, labels, name);
    }
}

async fn process_relations() {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(uri, user, pass).await.unwrap();
    let mut result = graph.execute(
        query("CREATE (p:Person { name: 'Oliver Stone' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r")
    ).await.unwrap();

    while let Ok(Some(row)) = result.next().await {
        let relation: Relation = row.get("r").unwrap();

        println!("{:?}", relation.start_node_id());
        println!("{:?}", relation.end_node_id());
        println!("{:?}", relation.typ());
        println!("{:?}", relation.get::<String>("as").unwrap());
    }
}
