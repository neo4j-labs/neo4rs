{ 
    //Parse time without offset
    let mut result = graph
        .execute(query(
            " WITH time({hour:10, minute:15, second:30, nanosecond: 200}) AS t RETURN t",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: (chrono::NaiveTime, Option<Offset>) = row.get("t").unwrap();
    assert_eq!(t.0.to_string(), "10:15:30.000000200");
    assert_eq!(t.1, None);
    assert!(result.next().await.unwrap().is_none());

    //Parse time with timezone information
    let mut result = graph
        .execute(query(
            " WITH time({hour:10, minute:15, second:33, nanosecond: 200, timezone: '+01:00'}) AS t RETURN t",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: (chrono::NaiveTime, Option<Offset>) = row.get("t").unwrap();
    assert_eq!(t.0.to_string(), "10:15:33.000000200");
    assert_eq!(t.1, Some(Offset(chrono::FixedOffset::east_opt(3600).unwrap())));
    assert!(result.next().await.unwrap().is_none());
}
