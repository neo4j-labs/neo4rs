{ 
    let date = chrono::NaiveDate::from_ymd_opt(1985, 2, 5).unwrap();
    let mut result = graph
        .execute(query("RETURN $d as output").param("d", date))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let d: chrono::NaiveDate = row.get("output").unwrap();
    assert_eq!(d.to_string(), "1985-02-05");
    assert!(result.next().await.unwrap().is_none());
}
