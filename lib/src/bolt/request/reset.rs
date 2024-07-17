use crate::bolt::{ExpectedResponse, Summary};
use serde::Serialize;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Reset;

impl ExpectedResponse for Reset {
    type Response = Summary<()>;
}

impl Serialize for Reset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_variant("Request", 0x0F, "RESET")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bolt::Message as _, packstream::bolt};

    #[test]
    fn serialize() {
        let bytes = Reset.to_bytes().unwrap();

        let expected = bolt().structure(0, 0x0F).build();

        assert_eq!(bytes, expected);
    }
}
