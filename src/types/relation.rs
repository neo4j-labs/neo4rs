use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::rc::Rc;

pub const MARKER: u8 = 0xB5;
pub const SIGNATURE: u8 = 0x52;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoltRelation {
    pub id: BoltInteger,
    pub start_node_id: BoltInteger,
    pub end_node_id: BoltInteger,
    pub typ: BoltString,
    pub properties: BoltMap,
}

impl BoltRelation {
    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
        let input = input.borrow();
        input.len() > 1 && input[0] == MARKER && input[1] == SIGNATURE
    }
}

impl BoltRelation {
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.properties.get(key)
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltRelation {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltRelation> {
        let marker = input.borrow_mut().get_u8();
        let tag = input.borrow_mut().get_u8();
        match (marker, tag) {
            (MARKER, SIGNATURE) => {
                let id: BoltInteger = input.clone().try_into()?;
                let start_node_id: BoltInteger = input.clone().try_into()?;
                let end_node_id: BoltInteger = input.clone().try_into()?;
                let typ: BoltString = input.clone().try_into()?;
                let properties: BoltMap = input.clone().try_into()?;
                Ok(BoltRelation {
                    id,
                    start_node_id,
                    end_node_id,
                    typ,
                    properties,
                })
            }
            _ => Err(Error::InvalidTypeMarker {
                detail: format!("invalid node marker/tag ({}, {})", marker, tag),
            }),
        }
    }
}

impl TryInto<Bytes> for BoltRelation {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let id: Bytes = self.id.try_into()?;
        let start_node_id: Bytes = self.start_node_id.try_into()?;
        let end_node_id: Bytes = self.end_node_id.try_into()?;
        let typ: Bytes = self.typ.try_into()?;
        let properties: Bytes = self.properties.try_into()?;

        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>()
                + mem::size_of::<u32>()
                + id.len()
                + start_node_id.len()
                + end_node_id.len()
                + typ.len()
                + properties.len(),
        );
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(id);
        bytes.put(start_node_id);
        bytes.put(end_node_id);
        bytes.put(typ);
        bytes.put(properties);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            relation.properties,
            vec![("name".into(), "Mark".into())].into_iter().collect()
        );
    }
}
