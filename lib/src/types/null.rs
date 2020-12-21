use bytes::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xC0;

#[derive(Debug, PartialEq, Eq, Hash, Clone, BoltStruct)]
#[signature(0xC0)]
pub struct BoltNull;

impl BoltNull {
    pub fn new() -> BoltNull {
        BoltNull {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn should_serialize_null() {
        let null = BoltNull::new();
        let b: Bytes = null.try_into().unwrap();
        assert_eq!(b.bytes(), &[0xC0]);
    }
}
