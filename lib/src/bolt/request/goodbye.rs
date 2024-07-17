use serde::Serialize;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Goodbye;

impl Serialize for Goodbye {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_variant("Request", 0x02, "GOODBYE")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bolt::Message as _, packstream::bolt};

    #[test]
    fn serialize() {
        let goodbye = Goodbye;
        let bytes = goodbye.to_bytes().unwrap();

        let expected = bolt().structure(0, 0x02).build();

        assert_eq!(bytes, expected);
    }
}
