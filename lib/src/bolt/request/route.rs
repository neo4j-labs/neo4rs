use crate::bolt::{ExpectedResponse, Summary};
use crate::connection::Routing;
use crate::routing::{Extra, Route, RouteExtra, RoutingTable};
use serde::ser::{SerializeMap, SerializeStructVariant};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Response {
    pub rt: RoutingTable,
}

impl ExpectedResponse for Route {
    type Response = Summary<Response>;
}

impl Serialize for Routing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Routing::No => serializer.serialize_none(),
            Routing::Yes(routing) => {
                let mut map = serializer.serialize_map(Some(routing.len()))?;
                for (k, v) in routing.iter() {
                    map.serialize_entry(k.value.as_str(), v.value.as_str())?;
                }
                map.end()
            }
        }
    }
}

impl Serialize for Route {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut structure = serializer.serialize_struct_variant("Request", 0x66, "ROUTE", 3)?;
        structure.serialize_field("routing", &self.routing)?;
        structure.serialize_field("bookmarks", &self.bookmarks)?;
        match self.extra {
            RouteExtra::V4_3(ref db) => {
                structure.serialize_field("db", db)?;
            }
            RouteExtra::V4_4(ref extra) => {
                structure.serialize_field("extra", extra)?;
            }
        }
        structure.end()
    }
}

impl Serialize for Extra {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("db", &self.db)?;
        map.serialize_entry("imp_user", &self.imp_user)?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::bolt::request::route::Response;
    use crate::bolt::{Message, MessageResponse};
    use crate::config::ImpersonateUser;
    use crate::connection::Routing;
    use crate::packstream::bolt;
    use crate::routing::RouteBuilder;
    use crate::{Database, Version};

    #[test]
    fn serialize() {
        let route = RouteBuilder::new(
            Routing::Yes([("address".into(), "localhost:7687".into())].into()),
            vec!["bookmark".into()],
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
    fn serialize_no_db() {
        let builder = RouteBuilder::new(
            Routing::Yes([("address".into(), "localhost:7687".into())].into()),
            vec!["bookmark".into()],
        );
        let route = builder.build(Version::V4_3);
        let serialized = route.to_bytes().unwrap();

        let expected = bolt()
            .structure(3, 0x66)
            .tiny_map(1)
            .tiny_string("address")
            .tiny_string("localhost:7687")
            .tiny_list(1)
            .tiny_string("bookmark")
            .null()
            .build();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_no_db_v4_4() {
        let builder = RouteBuilder::new(
            Routing::Yes([("address".into(), "localhost:7687".into())].into()),
            vec!["bookmark".into()],
        );
        let route = builder.build(Version::V4_4);
        let serialized = route.to_bytes().unwrap();

        let expected = bolt()
            .structure(3, 0x66)
            .tiny_map(1)
            .tiny_string("address")
            .tiny_string("localhost:7687")
            .tiny_list(1)
            .tiny_string("bookmark")
            .tiny_map(2)
            .tiny_string("db")
            .null()
            .tiny_string("imp_user")
            .null()
            .build();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_with_db_v4_4() {
        let builder = RouteBuilder::new(
            Routing::Yes([("address".into(), "localhost:7687".into())].into()),
            vec!["bookmark".into()],
        );
        let route = builder
            .with_db("neo4j".into())
            .with_imp_user(ImpersonateUser::from("Foo"))
            .build(Version::V4_4);
        let serialized = route.to_bytes().unwrap();

        let expected = bolt()
            .structure(3, 0x66)
            .tiny_map(1)
            .tiny_string("address")
            .tiny_string("localhost:7687")
            .tiny_list(1)
            .tiny_string("bookmark")
            .tiny_map(2)
            .tiny_string("db")
            .tiny_string("neo4j")
            .tiny_string("imp_user")
            .tiny_string("Foo")
            .build();

        assert_eq!(serialized, expected);
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
