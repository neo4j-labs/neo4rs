use serde::Serialize;

use crate::bolt::{ExpectedResponse, Summary};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Commit;

impl Serialize for Commit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_variant("Request", 0x12, "COMMIT")
    }
}

impl ExpectedResponse for Commit {
    type Response = Summary<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bolt::Message as _, packstream::bolt};

    #[test]
    fn serialize() {
        let commit = Commit;
        let bytes = commit.to_bytes().unwrap();

        let expected = bolt().structure(0, 0x12).build();

        assert_eq!(bytes, expected);
    }
}
