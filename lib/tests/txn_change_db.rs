use futures::TryStreamExt;
use neo4rs::*;
use serde::Deserialize;

mod container;

#[tokio::test]
async fn txn_changes_db() {
    let neo4j = match container::Neo4jContainerBuilder::new()
        .modify_config(|c| c.db("deebee"))
        .with_enterprise_edition()
        .start()
        .await
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
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

    let mut txn = graph.start_txn().await.unwrap();
    let databases = txn
        .execute(
            query(concat!(
                "SHOW TRANSACTIONS YIELD * WHERE username = $username AND currentQuery ",
                "STARTS WITH $query AND currentQueryStatus = $status RETURN database"
            ))
            .param("username", "neo4j")
            .param("query", "SHOW TRANSACTIONS YIELD ")
            .param("status", "running"),
        )
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
