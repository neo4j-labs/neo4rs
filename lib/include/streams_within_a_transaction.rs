{
    let qry = "UNWIND [1, 2] AS x RETURN x";

    let mut txn = graph.start_txn().await.unwrap();

    // start stream_one
    let mut stream_one = txn.execute(qry).await.unwrap();
    let row = stream_one.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<i64>().unwrap(), 1);

    // start stream_two
    let mut stream_two = txn.execute(qry).await.unwrap();
    let row = stream_two.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<i64>().unwrap(), 1);

    // stream_one is still active here
    let row = stream_one.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<i64>().unwrap(), 2);

    // as is stream_two
    let row = stream_two.next(txn.handle()).await.unwrap().unwrap();
    assert_eq!(row.to::<i64>().unwrap(), 2);

    // stream_one completes
    assert!(stream_one.next(txn.handle()).await.unwrap().is_none());

    // stream_two completes
    assert!(stream_two.next(txn.handle()).await.unwrap().is_none());

    // commit the transaction
    txn.commit().await.unwrap();
}
