use neo4rs::*;
use uuid::Uuid;

async fn graph() -> Graph {
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    Graph::new(uri, user, pass).await.unwrap()
}

#[tokio::test]
async fn should_execute_a_simple_query() {
    let graph = graph().await;
    let mut result = graph.execute(query("RETURN 1")).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
    assert!(result.next().await.unwrap().is_none());
}

#[tokio::test]
async fn should_create_new_node() {
    let graph = graph().await;
    let mut result = graph
        .execute(query("CREATE (friend:Person {name: 'Mark'})"))
        .await
        .unwrap();
    assert!(result.next().await.unwrap().is_none());
}

#[tokio::test]
async fn should_return_created_node() {
    let graph = graph().await;
    let mut result = graph
        .execute(query("CREATE (friend:Person {name: 'Mark'}) RETURN friend"))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let node: Node = row.get("friend").unwrap();
    let id = node.id();
    let labels = node.labels();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mark");
    assert_eq!(labels, vec!["Person"]);
    assert!(id > 0);
}

#[tokio::test]
async fn should_execute_query_with_params() {
    let graph = graph().await;
    let mut result = graph
        .execute(
            query("CREATE (friend:Person {name: $name}) RETURN friend").param("name", "Mr Mark"),
        )
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let node: Node = row.get("friend").unwrap();
    let name: String = node.get("name").unwrap();
    assert_eq!(name, "Mr Mark");
}

#[tokio::test]
async fn should_run_a_simple_query() {
    let graph = graph().await;
    assert!(graph.run(query("RETURN 1")).await.is_ok());
}

#[tokio::test]
async fn should_create_bounded_relation() {
    let graph = graph().await;
    let mut result = graph.execute(
        query("CREATE (p:Person { name: 'Oliver Stone' })-[r:WORKS_AT {as: 'Engineer'}]->(neo) RETURN r")
    ).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "WORKS_AT");
    assert_eq!(relation.get::<String>("as").unwrap(), "Engineer");
}

#[tokio::test]
async fn should_create_unbounded_relation() {
    let graph = graph().await;
    let mut result = graph.execute(
        query("MERGE (p1:Person { name: 'Oliver Stone' })-[r:RELATED {as: 'friend'}]-(p2: Person {name: 'Mark'}) RETURN r")
    ).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let relation: Relation = row.get("r").unwrap();
    assert!(relation.id() > -1);
    assert!(relation.start_node_id() > -1);
    assert!(relation.end_node_id() > -1);
    assert_eq!(relation.typ(), "RELATED");
    assert_eq!(relation.get::<String>("as").unwrap(), "friend");
}

#[tokio::test]
async fn should_run_all_queries_in_txn() {
    let graph = graph().await;
    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    assert!(txn
        .run_queries(vec![
            query("CREATE (p:Person {id: $id})").param("id", id.clone()),
            query("CREATE (p:Person {id: $id})").param("id", id.clone())
        ])
        .await
        .is_ok());
    txn.commit().await.unwrap();
    let result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert_eq!(count_rows(result).await, 2);
}

#[tokio::test]
async fn should_isolate_txn() {
    let graph = graph().await;
    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    let result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert_eq!(count_rows(result).await, 0);
    txn.commit().await.unwrap();
    let result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert_eq!(count_rows(result).await, 2);
}

#[tokio::test]
async fn should_rollback_txn() {
    let graph = graph().await;
    let txn = graph.start_txn().await.unwrap();
    let id = Uuid::new_v4().to_string();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    txn.run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();
    txn.rollback().await.unwrap();
    let result = graph
        .execute(query("MATCH (p:Person) WHERE p.id = $id RETURN p.id").param("id", id.clone()))
        .await
        .unwrap();
    assert_eq!(count_rows(result).await, 0);
}

#[tokio::test]
async fn should_handle_2d_points() {
    let graph = graph().await;
    let mut result = graph
        .execute(query(
            "WITH point({ x: 2.3, y: 4.5, crs: 'cartesian' }) AS p1, 
             point({ x: 1.1, y: 5.4, crs: 'cartesian' }) AS p2 RETURN distance(p1,p2) AS dist, p1, p2",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let dist: f64 = row.get("dist").unwrap();
    let p1: Point2D = row.get("p1").unwrap();
    let p2: Point2D = row.get("p2").unwrap();
    assert_eq!(1.5, dist);
    assert_eq!(p1.sr_id(), 7203);
    assert_eq!(p1.x(), 2.3);
    assert_eq!(p1.y(), 4.5);
    assert_eq!(p2.sr_id(), 7203);
    assert_eq!(p2.x(), 1.1);
    assert_eq!(p2.y(), 5.4);
    assert!(result.next().await.unwrap().is_none());
}

#[tokio::test]
async fn should_handle_3d_points() {
    let graph = graph().await;
    let mut result = graph
        .execute(query(
            "RETURN point({ longitude: 56.7, latitude: 12.78, height: 8 }) AS point",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let point: Point3D = row.get("point").unwrap();
    assert_eq!(point.sr_id(), 4979);
    assert_eq!(point.x(), 56.7);
    assert_eq!(point.y(), 12.78);
    assert_eq!(point.z(), 8.0);
    assert!(result.next().await.unwrap().is_none());
}

#[tokio::test]
async fn should_handle_raw_bytes() {
    let graph = graph().await;
    let mut result = graph
        .execute(query("RETURN $b as output").param("b", vec![11, 12]))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let b: Vec<u8> = row.get("output").unwrap();
    assert_eq!(b, &[11, 12]);
    assert!(result.next().await.unwrap().is_none());
}

#[tokio::test]
async fn should_handle_paths() {
    let graph = graph().await;
    let name = Uuid::new_v4().to_string();
    graph
        .run(
            query("CREATE (p:Person { name: $name })-[r:WORKS_AT]->(n:Company { name: 'Neo'})")
                .param("name", name.clone()),
        )
        .await
        .unwrap();

    let mut result = graph
        .execute(
            query("MATCH p = (person:Person { name: $name })-[r:WORKS_AT]->(c:Company) RETURN p")
                .param("name", name),
        )
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let path: Path = row.get("p").unwrap();
    assert_eq!(path.ids().len(), 2);
    assert_eq!(path.nodes().len(), 2);
    assert_eq!(path.rels().len(), 1);
    assert!(result.next().await.unwrap().is_none());
}

async fn count_rows(mut s: RowStream) -> usize {
    let mut count = 0;
    while let Ok(Some(_)) = s.next().await {
        count += 1;
    }
    count
}
