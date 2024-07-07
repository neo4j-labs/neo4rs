use crate::{
    bolt::{
        request::extra::{Extra, WrapExtra},
        ExpectedResponse, Summary,
    },
    errors::Result,
    summary::Streaming,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pull {
    extra: Extra,
}

impl WrapExtra for Pull {
    fn create(extra: Extra) -> Self {
        Self { extra }
    }

    fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}

impl Serialize for Pull {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_variant("Request", 0x3F, "PULL", &self.extra)
    }
}

impl ExpectedResponse for Pull {
    type Response = Summary<Streaming>;
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
        let hello = Pull::some(42).for_query(1);
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x3F)
            .tiny_map(2)
            .tiny_string("n")
            .tiny_int(42)
            .tiny_string("qid")
            .tiny_int(1)
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn serialize_default_values() {
        let hello = Pull::all();
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x3F)
            .tiny_map(1)
            .tiny_string("n")
            .tiny_int(-1)
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn parse() {
        let data = bolt()
            .tiny_map(1)
            .tiny_string("has_more")
            .bool(true)
            .build();

        let response = Streaming::parse(data).unwrap();

        assert_eq!(response, Streaming::HasMore);
    }
}
