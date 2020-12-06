use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::rc::Rc;

pub const MARKER: u8 = 0xB3;
pub const SIGNATURE: u8 = 0x4E;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoltNode {
    pub id: BoltInteger,
    pub labels: BoltList,
    pub properties: BoltMap,
}

impl BoltNode {
    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
        let marker = input.borrow()[0];
        let signature = input.borrow()[1];
        return marker == MARKER && signature == SIGNATURE;
    }
}

impl BoltNode {
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.properties.get(key)
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltNode {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltNode> {
        let marker = input.borrow_mut().get_u8();
        let tag = input.borrow_mut().get_u8();
        match (marker, tag) {
            (MARKER, SIGNATURE) => {
                let id: BoltInteger = input.clone().try_into()?;
                let labels: BoltList = input.clone().try_into()?;
                let properties: BoltMap = input.clone().try_into()?;
                Ok(BoltNode {
                    id,
                    labels,
                    properties,
                })
            }
            _ => Err(Error::InvalidTypeMarker {
                detail: format!("invalid node marker/tag ({}, {})", marker, tag),
            }),
        }
    }
}

impl TryInto<Bytes> for BoltNode {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let id: Bytes = self.id.try_into()?;
        let labels: Bytes = self.labels.try_into()?;
        let properties: Bytes = self.properties.try_into()?;

        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>()
                + mem::size_of::<u32>()
                + id.len()
                + labels.len()
                + properties.len(),
        );
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(id);
        bytes.put(labels);
        bytes.put(properties);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize_a_node() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x4E, 0x13, 0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, 0xA1, 0x84, 0x6E,
            0x61, 0x6D, 0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B,
        ])));

        let node: BoltNode = input.try_into().unwrap();

        assert_eq!(node.id, BoltInteger::new(19));
        assert_eq!(node.labels, vec!["Person".into()].into());
        assert_eq!(
            node.properties,
            vec![("name".into(), "Mark".into())].into_iter().collect()
        );
    }

    #[test]
    fn should_serialize_a_node() {
        let id = BoltInteger::new(19);
        let labels = vec!["Person".into()].into();
        let properties = vec![("name".into(), "Mark".into())].into_iter().collect();
        let node = BoltNode {
            id,
            labels,
            properties,
        };

        let bytes: Bytes = node.try_into().unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3, 0x4E, 0x13, 0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, 0xA1, 0x84, 0x6E,
                0x61, 0x6D, 0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B,
            ])
        );
    }
}
