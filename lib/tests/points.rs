use neo4rs::*;

mod container;

#[tokio::test]
async fn points() {
    let neo4j = container::Neo4jContainer::new().await;
    let graph = neo4j.graph();

    let distance = if neo4j.version().major >= 5 {
        "point.distance(p1,p2)"
    } else {
        "distance(p1,p2)"
    };

    let qry = format!(
        "WITH point({{ x: 2.3, y: 4.5, crs: 'cartesian' }}) AS p1,
              point({{ x: 1.1, y: 5.4, crs: 'cartesian' }}) AS p2
             RETURN {distance} AS dist, p1, p2",
        distance = distance
    );
    let qry = &qry;

    include!("../include/points.rs");
}
