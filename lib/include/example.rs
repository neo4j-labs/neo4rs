{ 
    let id = uuid::Uuid::new_v4().to_string();
    graph
        .run(query("CREATE (p:Person {id: $id})").param("id", id.clone()))
        .await
        .unwrap();

    let mut handles = Vec::new();
    let count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    for _ in 1..=42 {
        let graph = graph.clone();
        let id = id.clone();
        let count = count.clone();
        let handle = tokio::spawn(async move {
            let mut result = graph
                .execute(query("MATCH (p:Person {id: $id}) RETURN p").param("id", id))
                .await
                .unwrap();
            while let Ok(Some(_row)) = result.next().await {
                count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    futures::future::join_all(handles).await;
    assert_eq!(count.load(std::sync::atomic::Ordering::Relaxed), 42);
}
