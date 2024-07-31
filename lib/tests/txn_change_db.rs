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
        "DROP DATABASE deebee IF EXISTS",
        "CREATE DATABASE deebee",
        "START DATABASE deebee",
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap();

    #[derive(Deserialize)]
    struct Database {
        name: String,
    }

    let mut txn = graph.start_txn().await.unwrap();
    let databases = txn.execute(query("SHOW DATABASES")).await.unwrap();

    let mut names = databases
        .into_stream_as::<Database>(txn.handle())
        .map_ok(|db| db.name)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    names.sort();

    assert_eq!(names, vec!["deebee", "neo4j", "system"]);

    txn.commit().await.unwrap();
}
