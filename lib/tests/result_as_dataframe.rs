#![cfg(feature = "polars_v0_43")]

use neo4rs::query;
use polars::prelude::{AnyValue, DataType};

mod container;

#[tokio::test]
async fn result_as_dataframe() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let result = graph
        .execute(query(
            "UNWIND [TRUE, FALSE, NULL, 1, 420000, 13.37] AS values RETURN values",
        ))
        .await
        .unwrap();

    let df = result.into_dataframe().await.unwrap();
    #[cfg(feature = "unstable-result-summary")]
    let df = df.0;

    assert_eq!(df.get_column_names(), ["values"]);
    assert_eq!(df.height(), 6);
    assert_eq!(df.width(), 1);

    let values = df.column("values").unwrap();

    assert_eq!(values.dtype(), &DataType::Float64);
    values
        .iter()
        .filter_map(|a| match a {
            AnyValue::Float64(a) => Some(a),
            AnyValue::Null => None,
            _ => panic!("`{a:?} is not a float or null"),
        })
        .zip([1.0, 0.0, 1.0, 420000.0, 13.37])
        .for_each(|(a, b)| {
            assert!((a - b).abs() <= f64::EPSILON);
        });
}
