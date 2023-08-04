{ 
    //Parse NaiveDateTime from result
    let mut result = graph
        .execute(query(
            "WITH localdatetime('2015-06-24T12:50:35.556') AS t RETURN t",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: chrono::NaiveDateTime = row.get("t").unwrap();
    assert_eq!(t.to_string(), "2015-06-24 12:50:35.556");
    assert!(result.next().await.unwrap().is_none());

    //Parse DateTime from result
    let mut result = graph
        .execute(query(
            "WITH datetime('2015-06-24T12:50:35.777+0100') AS t RETURN t",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: chrono::DateTime<chrono::FixedOffset> = row.get("t").unwrap();
    assert_eq!(t.to_string(), "2015-06-24 12:50:35.777 +01:00");
    assert!(result.next().await.unwrap().is_none());

    //Parse NaiveDateTime with zone id from result
    let mut result = graph
        .execute(query(
            "WITH datetime({ year:1984, month:11, day:11, hour:12, minute:31, second:14, nanosecond: 645876123, timezone:'Europe/Stockholm' }) AS d return d",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let (datetime, zone_id): (chrono::NaiveDateTime, String) = row.get("d").unwrap();
    assert_eq!(datetime.to_string(), "1984-11-11 12:31:14.645876123");
    assert_eq!(zone_id, "Europe/Stockholm");
    assert!(result.next().await.unwrap().is_none());
}
