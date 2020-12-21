use crate::types::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x71;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x71)]
pub struct Record {
    pub data: BoltList,
}

impl Record {
    fn marker() -> (u8, Option<u8>) {
        (MARKER, Some(SIGNATURE))
    }
}

impl Record {
    pub fn new(data: BoltList) -> Record {
        Record { data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
    use std::cell::RefCell;
    use std::convert::TryInto;
    use std::rc::Rc;

    #[test]
    fn should_deserialize_record_message() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            MARKER, SIGNATURE, 0x92, 0x81, 0x61, 0x81, 0x62,
        ])));

        let record: Record = bytes.try_into().unwrap();

        assert_eq!(record.data.len(), 2);
    }
}
