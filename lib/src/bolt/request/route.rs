use crate::bolt::{ExpectedResponse, Summary};
use crate::connection::{NeoUrl, Routing};
use crate::routing::{Extra, Route, RoutingTable};
use serde::ser::{SerializeMap, SerializeStructVariant};
use serde::{Deserialize, Serialize};
use std::fmt::{format, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Response {
    pub rt: RoutingTable,
}

impl<'a> ExpectedResponse for Route<'a> {
    type Response = Summary<Response>;
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl Serialize for Routing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Routing::No => serializer.serialize_none(),
            Routing::Yes(routing) => {
                let mut map = serializer.serialize_map(Some(routing.len()))?;
                for (k, v) in routing {
                    map.serialize_entry(k.to_string().as_str(), v.to_string().as_str())?;
                }
                map.end()
            }
        }
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl Serialize for Route<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut structure = serializer.serialize_struct_variant("Request", 0x66, "ROUTE", 3)?;
        structure.serialize_field("routing", &self.routing)?;
        structure.serialize_field("bookmarks", &self.bookmarks)?;
        if let Some(extra) = &self.extra {
            structure.serialize_field("extra", extra)?;
        } else if let Some(db) = &self.db {
            structure.serialize_field("db", &db.to_string())?;
        } else {
            structure.skip_field("db")?; // Render a null value
        }
        structure.end()
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
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
    use crate::connection::Routing;
    use crate::packstream::bolt;
    use crate::routing::{Route, RouteBuilder};
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
    fn serialize_no_db() {
        let builder = RouteBuilder::new(
            Routing::Yes(vec![("address".into(), "localhost:7687".into())]),
            vec!["bookmark"],
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
            Routing::Yes(vec![("address".into(), "localhost:7687".into())]),
            vec!["bookmark"],
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
