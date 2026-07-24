#![cfg(feature = "unstable-result-summary")]
use neo4rs::*;

mod container;

#[tokio::test]
async fn streaming_summary() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    // snippet-start
    use ::futures::TryStreamExt as _;

    use neo4rs::summary::{Counters, ResultSummary, Type};

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
        assert_eq!(
            summary.stats(),
            &Counters {
                nodes_created: 1,
                properties_set: 1,
                labels_added: 1,
                ..Default::default()
            }
        );
    }

    //
    // next + finish

    let mut stream = graph
        .execute(query("CREATE (n:Node {prop: 'frobnicate'}) RETURN n"))
        .await
        .unwrap();

    let Ok(Some(row)) = stream.next().await else {
        panic!()
    };
    assert_item(row.to().unwrap());

    let Ok(summary) = stream.finish().await else {
        panic!()
    };
    assert_summary(&summary);

    //
    // into_stream + finish

    let mut stream = graph
        .execute(query("CREATE (n:Node {prop: 'frobnicate'}) RETURN n"))
        .await
        .unwrap();

    let items = stream
        .into_stream_as::<N>()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    let Ok(summary) = stream.finish().await else {
        panic!()
    };

    for item in items {
        assert_item(item);
    }

    assert_summary(&summary);
    // snippet-end
}
