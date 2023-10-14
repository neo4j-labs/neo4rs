{
    let before = graph
        .execute(query("MATCH (n:MyNode) RETURN COUNT(n) AS n"))
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap()
        .get::<i64>("n")
        .unwrap();

    // use `run` for fire-and-forget queries, that are being executed on the server
    graph
        .run(query("CREATE (n:MyNode {p: 'prop'})"))
        .await
        .unwrap();

    // using `execute` without consuming the result will do nothing
    // This will trigger a `unused_must_use` warning
    graph
        .execute(query("CREATE (n:MyNode {p: 'prop'})"))
        .await
        .unwrap();

    // consuming the result stream of`execute` will run the query on the server
    graph
        .execute(query("CREATE (n:MyNode {p: 'prop'})"))
        .await
        .unwrap()
        .next()
        .await
        .unwrap();

    let after = graph
        .execute(query("MATCH (n:MyNode) RETURN COUNT(n) AS n"))
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap()
        .get::<i64>("n")
        .unwrap();

    assert_eq!(after, before + 2);
}
