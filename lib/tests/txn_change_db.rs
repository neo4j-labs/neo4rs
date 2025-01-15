use futures::TryStreamExt;
use neo4rs::query;
use serde::Deserialize;

mod container;

#[tokio::test]
async fn txn_changes_db() {
    let neo4j = match container::Neo4jContainerBuilder::new()
        .modify_driver_config(|c| c.db("deebee"))
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

            std::panic::panic_any(e.to_string());
        }
    };
    let graph = neo4j.graph();

    let mut txn = graph.start_txn_on("system").await.unwrap();
    txn.run_queries([
        "CREATE DATABASE deebee IF NOT EXISTS",
        "START DATABASE deebee",
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap();

    #[derive(Deserialize)]
    struct Database {
        database: String,
    }

    let status_field = if neo4j.version().major >= 5 {
        "currentQueryStatus"
    } else {
        "status"
    };

    let mut txn = graph.start_txn().await.unwrap();
    let mut databases = txn
        .execute(query!(
            "SHOW TRANSACTIONS YIELD * WHERE username = {username} AND currentQuery
STARTS WITH {query} AND toLower({status_field}) = {status} RETURN database",
            username = "neo4j",
            query = "SHOW TRANSACTIONS YIELD ",
            status = "running",
        ))
        .await
        .unwrap();

    let names = databases
        .into_stream_as::<Database>(txn.handle())
        .map_ok(|db| db.database)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    assert_eq!(names, vec!["deebee"]);

    txn.commit().await.unwrap();
}
