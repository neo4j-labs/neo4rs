use std::{borrow::Borrow, collections::HashMap};

use crate::{
    bolt::{ExpectedResponse, Summary},
    Version,
};
use serde::{ser::SerializeMap, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello<'a> {
    metadata: Meta<'a>,
}

pub struct HelloBuilder<'a> {
    scheme: &'a str,
    principal: &'a str,
    credentials: &'a str,
    user_agent: &'a str,
    routing: ServerRouting<'a>,
}

impl<'a> HelloBuilder<'a> {
    pub fn new(principal: &'a str, credentials: &'a str) -> Self {
        Self {
            scheme: "basic",
            principal,
            credentials,
            user_agent: "neo4rs",
            routing: ServerRouting::No,
        }
    }

    pub fn with_routing(
        self,
        context: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> HelloBuilder<'a> {
        let context = context.into_iter().collect::<Box<[_]>>();
        let routing = if context.is_empty() {
            ServerRouting::Yes
        } else {
            ServerRouting::WithContext { context }
        };
        HelloBuilder { routing, ..self }
    }

    pub fn build(self, version: Version) -> Hello<'a> {
        let Self {
            scheme,
            principal,
            credentials,
            user_agent,
            mut routing,
        } = self;

        if version < Version::V4_1 {
            routing = ServerRouting::No;
        }

        let metadata = Meta {
            user_agent,
            scheme,
            principal,
            credentials,
            routing,
        };
        Hello { metadata }
    }
}

impl<'a> Hello<'a> {
    pub fn builder(principal: &'a str, credentials: &'a str) -> HelloBuilder<'a> {
        HelloBuilder::new(principal, credentials)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Meta<'a> {
    scheme: &'a str,
    principal: &'a str,
    credentials: &'a str,
    user_agent: &'a str,
    #[serde(skip_serializing_if = "ServerRouting::is_none")]
    routing: ServerRouting<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServerRouting<'a> {
    No,
    Yes,
    WithContext { context: Box<[(&'a str, &'a str)]> },
}

impl<'a> ServerRouting<'a> {
    fn is_none(&self) -> bool {
        matches!(self, ServerRouting::No)
    }
}

impl<'a> Serialize for ServerRouting<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ServerRouting::No => serializer.serialize_none(),
            ServerRouting::Yes => serializer.serialize_map(Some(0))?.end(),
            ServerRouting::WithContext { context } => {
                let mut map = serializer.serialize_map(Some(context.len()))?;
                for (k, v) in &**context {
                    map.serialize_entry(*k, *v)?;
                }
                map.end()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Response {
    pub(crate) server: String,
    pub(crate) connection_id: String,
}

impl<'a> ExpectedResponse for Hello<'a> {
    type Response = Summary<Response>;
}

impl<'a> Serialize for Hello<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_variant("Request", 0x01, "HELLO", &self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::{Message as _, MessageResponse as _},
        packstream::bolt,
    };

    #[test]
    fn serialize() {
        let hello = Hello::builder("user", "pass").build(Version::V4_1);
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x01)
            .tiny_map(4)
            .tiny_string("scheme")
            .tiny_string("basic")
            .tiny_string("principal")
            .tiny_string("user")
            .tiny_string("credentials")
            .tiny_string("pass")
            .tiny_string("user_agent")
            .tiny_string("neo4rs")
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn serialize_with_server_side_routing() {
        let hello = Hello::builder("user", "pass")
            .with_routing([])
            .build(Version::V4_1);
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x01)
            .tiny_map(5)
            .tiny_string("scheme")
            .tiny_string("basic")
            .tiny_string("principal")
            .tiny_string("user")
            .tiny_string("credentials")
            .tiny_string("pass")
            .tiny_string("user_agent")
            .tiny_string("neo4rs")
            .tiny_string("routing")
            .tiny_map(0)
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn serialize_with_routing_context() {
        let hello = Hello::builder("user", "pass")
            .with_routing([("region", "eu-west-1"), ("zone", "a")])
            .build(Version::V4_1);
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x01)
            .tiny_map(5)
            .tiny_string("scheme")
            .tiny_string("basic")
            .tiny_string("principal")
            .tiny_string("user")
            .tiny_string("credentials")
            .tiny_string("pass")
            .tiny_string("user_agent")
            .tiny_string("neo4rs")
            .tiny_string("routing")
            .tiny_map(2)
            .tiny_string("region")
            .tiny_string("eu-west-1")
            .tiny_string("zone")
            .tiny_string("a")
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn serialize_routing_based_on_version() {
        let hello = Hello::builder("user", "pass")
            .with_routing([])
            .build(Version::V4);
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x01)
            .tiny_map(4)
            .tiny_string("scheme")
            .tiny_string("basic")
            .tiny_string("principal")
            .tiny_string("user")
            .tiny_string("credentials")
            .tiny_string("pass")
            .tiny_string("user_agent")
            .tiny_string("neo4rs")
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn parse() {
        let data = bolt()
            .tiny_map(2)
            .tiny_string("server")
            .tiny_string("Neo4j/4.1.4")
            .tiny_string("connection_id")
            .tiny_string("bolt-31")
            .build();

        let response = Response::parse(data).unwrap();

        assert_eq!(response.server, "Neo4j/4.1.4");
        assert_eq!(response.connection_id, "bolt-31");
    }
}
