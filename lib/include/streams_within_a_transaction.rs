{ 
    let name = uuid::Uuid::new_v4().to_string();
    let mut txn = graph.start_txn().await.unwrap();

    #[derive(serde::Deserialize)]
    struct Person {
        name: String,
    }

    txn.run_queries([
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
    let row = stream_one.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<Person>().unwrap().name, name);

    //start stream_two
    let mut stream_two = txn.execute(query("RETURN 1")).await.unwrap();
    let row = stream_two.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<i64>().unwrap(), 1);

    //stream_one is still active here
    let row = stream_one.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<Person>().unwrap().name, name);

    //stream_one completes
    assert!(stream_one.next(txn.handle()).await.unwrap().is_none());
    //stream_two completes
    assert!(stream_two.next(txn.handle()).await.unwrap().is_none());
    txn.commit().await.unwrap();
}
