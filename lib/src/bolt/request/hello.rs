use crate::bolt::{ExpectedResponse, Summary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello<'a> {
    metadata: Meta<'a>,
}

impl<'a> Hello<'a> {
    pub fn new(user_agent: &'a str, principal: &'a str, credentials: &'a str) -> Self {
        let metadata = Meta {
            user_agent,
            scheme: "basic",
            principal,
            credentials,
        };
        Hello { metadata }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Meta<'a> {
    scheme: &'a str,
    principal: &'a str,
    credentials: &'a str,
    user_agent: &'a str,
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
        let hello = Hello::new("foo", "user", "pass");
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
            .tiny_string("foo")
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
