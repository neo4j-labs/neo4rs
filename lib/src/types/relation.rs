use crate::types::*;
use neo4rs_macros::BoltStruct;

pub const MARKER_REL: u8 = 0xB5;
pub const SIGNATURE_REL: u8 = 0x52;

pub const MARKER_UNBOUNDED_REL: u8 = 0xB3;
pub const SIGNATURE_UNBOUNDED_REL: u8 = 0x72;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
pub struct BoltRelation {
    pub id: BoltInteger,
    pub start_node_id: BoltInteger,
    pub end_node_id: BoltInteger,
    pub typ: BoltString,
    pub properties: BoltMap,
}

impl BoltRelation {
    fn marker() -> (u8, Option<u8>) {
        (MARKER_REL, Some(SIGNATURE_REL))
    }
}

#[derive(Debug, PartialEq, Clone, BoltStruct)]
pub struct BoltUnboundedRelation {
    pub id: BoltInteger,
    pub typ: BoltString,
    pub properties: BoltMap,
}

impl BoltUnboundedRelation {
    fn marker() -> (u8, Option<u8>) {
        (MARKER_UNBOUNDED_REL, Some(SIGNATURE_UNBOUNDED_REL))
    }
}

impl BoltUnboundedRelation {
    pub fn new(id: BoltInteger, typ: BoltString, properties: BoltMap) -> Self {
        BoltUnboundedRelation {
            id,
            typ,
            properties,
        }
    }
}

impl BoltRelation {
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.properties.get(key)
    }
}

impl BoltUnboundedRelation {
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.properties.get(key)
    }
}

impl Into<BoltType> for BoltRelation {
    fn into(self) -> BoltType {
        BoltType::Relation(self)
    }
}

impl Into<BoltType> for BoltUnboundedRelation {
    fn into(self) -> BoltType {
        BoltType::UnboundedRelation(self)
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
    fn should_serialize_a_relation() {
        let id = BoltInteger::new(42);
        let start_node_id = BoltInteger::new(1);
        let end_node_id = BoltInteger::new(2);
        let typ = BoltString::new("rel");
        let properties = vec![("name".into(), "Mark".into())].into_iter().collect();

        let relation = BoltRelation {
            id,
            start_node_id,
            end_node_id,
            typ,
            properties,
        };

        let bytes: Bytes = relation.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB5, 0x52, 0x2A, 0x01, 0x02, 0x83, 0x72, 0x65, 0x6C, 0xA1, 0x84, 0x6E, 0x61, 0x6D,
                0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B,
            ])
        );
    }

    #[test]
    fn should_deserialize_a_relation() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB5, 0x52, 0x2A, 0x01, 0x02, 0x83, 0x72, 0x65, 0x6C, 0xA1, 0x84, 0x6E, 0x61, 0x6D,
            0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B,
        ])));

        let relation: BoltRelation = input.try_into().unwrap();

        assert_eq!(relation.id, BoltInteger::new(42));
        assert_eq!(relation.start_node_id, BoltInteger::new(1));
        assert_eq!(relation.end_node_id, BoltInteger::new(2));
        assert_eq!(relation.typ, BoltString::new("rel"));
        assert_eq!(
            relation.properties,
            vec![("name".into(), "Mark".into())].into_iter().collect()
        );
    }

    #[test]
    fn should_serialize_an_unbounded_relation() {
        let id = BoltInteger::new(42);
        let typ = BoltString::new("rel");
        let properties = vec![("name".into(), "Mark".into())].into_iter().collect();
        let relation = BoltUnboundedRelation::new(id, typ, properties);

        let bytes: Bytes = relation.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3, 0x72, 0x2A, 0x83, 0x72, 0x65, 0x6C, 0xA1, 0x84, 0x6E, 0x61, 0x6D, 0x65, 0x84,
                0x4D, 0x61, 0x72, 0x6B,
            ])
        );
    }

    #[test]
    fn should_deserialize_an_unbounded_relation() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x72, 0x2A, 0x83, 0x72, 0x65, 0x6C, 0xA1, 0x84, 0x6E, 0x61, 0x6D, 0x65, 0x84,
            0x4D, 0x61, 0x72, 0x6B,
        ])));

        let relation: BoltUnboundedRelation = input.try_into().unwrap();

        assert_eq!(relation.id, BoltInteger::new(42));
        assert_eq!(relation.typ, BoltString::new("rel"));
        assert_eq!(
            relation.properties,
            vec![("name".into(), "Mark".into())].into_iter().collect()
        );
    }
}
