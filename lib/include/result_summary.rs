{
    use ::futures::TryStreamExt as _;

    use neo4rs::summary::{Type, Counters, ResultSummary};

    #[allow(dead_code)]
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct N {
        prop: String,
    }

    fn assert_item(n: N) {
        assert_eq!(n.prop, "frobnicate");
    }

    fn assert_summary(summary: &ResultSummary) {
        assert!(summary.available_after().is_some());
        assert!(summary.consumed_after().is_some());
        assert!(summary.db().is_some());
        assert_eq!(summary.query_type(), Type::ReadWrite);
        assert_eq!(summary.stats(), &Counters { nodes_created: 1, properties_set: 1, labels_added: 1, ..Default::default()});
    }

    //
    // next_or_summary

    let mut stream = graph
        .execute(query("CREATE (n:Node {prop: 'frobnicate'}) RETURN n"))
        .await
        .unwrap();

    let Ok(Some(row)) = stream.next_or_summary().await else { panic!() };
    assert!(row.row().is_some());
    assert!(row.summary().is_none());

    assert_item(row.row().unwrap().to().unwrap());

    let Ok(Some(row)) = stream.next_or_summary().await else { panic!() };
    assert!(row.row().is_none());
    assert!(row.summary().is_some());

    assert_summary(row.summary().unwrap());


    //
    // as_items

    let mut stream = graph
        .execute(query("CREATE (n:Node {prop: 'frobnicate'}) RETURN n"))
        .await
        .unwrap();

    let items = stream.as_items::<N>()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    for item in items {
        match item {
            RowItem::Row(row) => assert_item(row),
            RowItem::Summary(summary) => assert_summary(&summary),
        }
    }


    //
    // into_stream + finish

    let mut stream = graph
        .execute(query("CREATE (n:Node {prop: 'frobnicate'}) RETURN n"))
        .await
        .unwrap();

    let items = stream.into_stream_as::<N>()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    let Ok(Some(summary)) = stream.finish().await else { panic!() };

    for item in items {
        assert_item(item);
    }

    assert_summary(&summary);
}
