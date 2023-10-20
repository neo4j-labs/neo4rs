{ 
    //send time without offset as param
    let time = chrono::NaiveTime::from_hms_nano_opt(11, 15, 30, 200).unwrap();
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", time))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: (chrono::NaiveTime, Option<Offset>) = row.get("output").unwrap();
    assert_eq!(t.0.to_string(), "11:15:30.000000200");
    assert_eq!(t.1, None);
    assert!(result.next().await.unwrap().is_none());

    //send time with offset as param
    let time = chrono::NaiveTime::from_hms_nano_opt(11, 15, 30, 200).unwrap();
    let offset = chrono::FixedOffset::east_opt(3 * 3600).unwrap();
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", (time, offset)))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let t: (chrono::NaiveTime, Option<Offset>) = row.get("output").unwrap();
    assert_eq!(t.0.to_string(), "11:15:30.000000200");
    assert_eq!(t.1, Some(Offset(offset)));
    assert!(result.next().await.unwrap().is_none());
}
