use crate::bolt::{ExpectedResponse, Summary};
use crate::connection::NeoUrl;
use crate::routing::{Route, RoutingTable};
use serde::ser::SerializeStructVariant;
use serde::{Deserialize, Serialize};
use std::fmt::{format, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Response {
    pub(crate) rt: RoutingTable,
}

impl<'a> ExpectedResponse for Route<'a> {
    type Response = Summary<Response>;
}

#[cfg(test)]
mod tests {
    use crate::bolt::request::route::Response;
    use crate::bolt::{Message, MessageResponse};
    use crate::packstream::bolt;
    use crate::routing::{Route, RouteBuilder, Routing};
    use crate::{Database, Version};

    #[test]
    fn serialize() {
        let route = RouteBuilder::new(
            Routing::Yes(vec![("address".into(), "localhost:7687".into())]),
            vec!["bookmark"],
        )
        .with_db(Database::from("neo4j"))
        .build(Version::V4_3);
        let bytes = route.to_bytes().unwrap();

        let expected = bolt()
            .structure(3, 0x66)
            .tiny_map(1)
            .tiny_string("address")
            .tiny_string("localhost:7687")
            .tiny_list(1)
            .tiny_string("bookmark")
            .tiny_string("neo4j")
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn parse() {
        let data = bolt()
            .tiny_map(1)
            .tiny_string("rt")
            .tiny_map(3)
            .tiny_string("ttl")
            .int64(1000)
            .tiny_string("db")
            .tiny_string("neo4j")
            .tiny_string("servers")
            .tiny_list(1)
            .tiny_map(2)
            .tiny_string("addresses")
            .tiny_list(1)
            .tiny_string("localhost:7687")
            .tiny_string("role")
            .tiny_string("ROUTE")
            .build();

        let response = Response::parse(data).unwrap();

        assert_eq!(response.rt.ttl, 1000);
        assert_eq!(response.rt.db.unwrap().as_ref(), "neo4j");
        assert_eq!(response.rt.servers.len(), 1);
    }
}
