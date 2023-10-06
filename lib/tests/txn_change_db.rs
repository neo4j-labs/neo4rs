use futures::{stream, StreamExt, TryStreamExt};
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

    stream::iter([
        "CREATE DATABASE deebee",
        "START DATABASE deebee",
        "STOP DATABASE neo4j",
        "DROP DATABASE neo4j",
    ])
    .then(|q| graph.run_on("system", query(q)))
    .try_collect::<()>()
    .await
    .unwrap();

    let txn = graph.start_txn().await.unwrap();

    #[derive(Deserialize)]
    struct Database {
        name: String,
    }

    let databases = txn.execute(query("SHOW DATABASES")).await.unwrap();

    let names = stream::unfold(databases, |mut databases| async move {
        let db = databases.next().await.unwrap()?;
        let db = db.to::<Database>().unwrap();
        Some((db.name, databases))
    });

    let mut names = names.collect::<Vec<_>>().await;
    names.sort();

    assert_eq!(names, vec!["deebee", "system"]);

    txn.commit().await.unwrap();
}
