{ 
    //send datetime as parameter in the query
    let datetime = chrono::DateTime::parse_from_rfc2822("Tue, 01 Jul 2003 10:52:37 +0200").unwrap();

    let mut result = graph
        .execute(query("RETURN $d as output").param("d", datetime))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: chrono::DateTime<chrono::FixedOffset> = row.get("output").unwrap();
    assert_eq!(t.to_string(), "2003-07-01 10:52:37 +02:00");
    assert!(result.next().await.unwrap().is_none());

    //send NaiveDateTime as parameter in the query
    let localdatetime =
    chrono::NaiveDateTime::parse_from_str("2015-07-01 08:55:59.123", "%Y-%m-%d %H:%M:%S%.f")
        .unwrap();

    let mut result = graph
        .execute(query("RETURN $d as output").param("d", localdatetime))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: chrono::NaiveDateTime = row.get("output").unwrap();
    assert_eq!(t.to_string(), "2015-07-01 08:55:59.123");
    assert!(result.next().await.unwrap().is_none());

    //send NaiveDateTime with timezone id as parameter in the query
    let datetime =
    chrono::NaiveDateTime::parse_from_str("2015-07-03 08:55:59.555", "%Y-%m-%d %H:%M:%S%.f")
        .unwrap();
    let timezone = "Europe/Paris";

    let mut result = graph
        .execute(query("RETURN $d as output").param("d", (datetime, timezone)))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let (time, zone): (chrono::NaiveDateTime, String) = row.get("output").unwrap();
    assert_eq!(time.to_string(), "2015-07-03 08:55:59.555");
    assert_eq!(zone, "Europe/Paris");
    assert!(result.next().await.unwrap().is_none());
}
