use neo4rs::{query, Node};
use serde::Deserialize;

mod container;

#[tokio::test]
async fn missing_properties() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let a_val = None::<String>;
    let mut result = graph
        .execute(query("CREATE (ts:TestStruct {a: $a}) RETURN ts").param("a", a_val))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let expected = StructWithOption { a: None, b: None };

    let test_struct: StructWithOption = row.to().unwrap();
    assert_eq!(test_struct, expected);

    let node = row.get::<Node>("ts").unwrap();
    let test_struct: StructWithOption = node.to().unwrap();
    assert_eq!(test_struct, expected);
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
struct StructWithOption {
    a: Option<String>,
    b: Option<String>,
}
