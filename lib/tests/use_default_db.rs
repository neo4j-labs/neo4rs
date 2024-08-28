use futures::TryStreamExt;
use neo4rs::*;

mod container;

#[tokio::test]
async fn use_default_db() {
    let dbname = uuid::Uuid::new_v4().to_string().replace(['-', '_'], "");

    let neo4j = match container::Neo4jContainerBuilder::new()
        .with_server_config("initial.dbms.default_database", dbname.as_str())
        .with_enterprise_edition()
        .start()
        .await
    {
        Ok(n) => n,
        Err(e) => {
            if e.to_string().contains("Neo4j Enterprise Edition") {
                eprintln!("Skipping test: {}", e);
                return;
            }

            std::panic::panic_any(e);
        }
    };
    let graph = neo4j.graph();

    let default_db = graph
        .execute_on("system", query("SHOW DEFAULT DATABASE"))
        .await
        .unwrap()
        .column_into_stream::<String>("name")
        .try_fold(None::<String>, |acc, db| async { Ok(acc.or(Some(db))) })
        .await
        .unwrap()
        .unwrap();

    if default_db != dbname {
        eprintln!(
            concat!(
                "Skipping test: The test must run against a testcontainer ",
                "or have `{}` configured as the default database"
            ),
            dbname
        );
        return;
    }

    let id = uuid::Uuid::new_v4();
    graph
        .run(query("CREATE (:Node { uuid: $uuid })").param("uuid", id.to_string()))
        .await
        .unwrap();

    let count = graph
        .execute_on(
            dbname.as_str(),
            query("MATCH (n:Node {uuid: $uuid}) RETURN count(n) AS result")
                .param("uuid", id.to_string()),
        )
        .await
        .unwrap()
        .column_into_stream::<u64>("result")
        .try_fold(0, |sum, count| async move { Ok(sum + count) })
        .await
        .unwrap();

    assert_eq!(count, 1);
}
