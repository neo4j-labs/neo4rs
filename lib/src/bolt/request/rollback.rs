use serde::Serialize;

use crate::bolt::{ExpectedResponse, Summary};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Rollback;

impl Serialize for Rollback {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_variant("Request", 0x13, "ROLLBACK")
    }
}

impl ExpectedResponse for Rollback {
    type Response = Summary<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bolt::Message as _, packstream::bolt};

    #[test]
    fn serialize() {
        let rollback = Rollback;
        let bytes = rollback.to_bytes().unwrap();

        let expected = bolt().structure(0, 0x13).build();

        assert_eq!(bytes, expected);
    }
}
