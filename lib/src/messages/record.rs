use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x71)]
pub struct Record {
    pub data: BoltList,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn should_deserialize_record_message() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB1, 0x71, 0x92, 0x81, 0x61, 0x81, 0x62,
        ])));

        let record: Record = Record::parse(Version::V4_1, bytes).unwrap();

        assert_eq!(record.data.len(), 2);
    }
}
