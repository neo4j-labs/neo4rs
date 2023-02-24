use neo4rs::*;
use uuid::Uuid;

mod container;

#[tokio::test]
async fn streams_within_a_transaction() {
    let config = config().fetch_size(1);
    let neo4j = container::Neo4jContainer::from_config(config).await;
    let graph = neo4j.graph();

    let name = Uuid::new_v4().to_string();
    let txn = graph.start_txn().await.unwrap();

    txn.run_queries(vec![
        query("CREATE (p { name: $name })").param("name", name.clone()),
        query("CREATE (p { name: $name })").param("name", name.clone()),
    ])
    .await
    .unwrap();

    //start stream_one
    let mut stream_one = txn
        .execute(query("MATCH (p {name: $name}) RETURN p").param("name", name.clone()))
        .await
        .unwrap();
    let row = stream_one.next().await.unwrap().unwrap();
    assert_eq!(
        row.get::<Node>("p").unwrap().get::<String>("name").unwrap(),
        name.clone()
    );

    //start stream_two
    let mut stream_two = txn.execute(query("RETURN 1")).await.unwrap();
    let row = stream_two.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("1").unwrap(), 1);

    //stream_one is still active here
    let row = stream_one.next().await.unwrap().unwrap();
    assert_eq!(
        row.get::<Node>("p").unwrap().get::<String>("name").unwrap(),
        name.clone()
    );

    //stream_one completes
    assert!(stream_one.next().await.unwrap().is_none());
    //stream_two completes
    assert!(stream_two.next().await.unwrap().is_none());
    txn.commit().await.unwrap();
}
