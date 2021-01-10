use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB3, 0x4E)]
pub struct BoltNode {
    pub id: BoltInteger,
    pub labels: BoltList,
    pub properties: BoltMap,
}

impl BoltNode {
    pub fn new(id: BoltInteger, labels: BoltList, properties: BoltMap) -> Self {
        BoltNode {
            id,
            labels,
            properties,
        }
    }
}

impl BoltNode {
    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.properties.get(key)
    }
}

impl Into<BoltType> for BoltNode {
    fn into(self) -> BoltType {
        BoltType::Node(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn should_deserialize_a_node() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x4E, 0x13, 0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, 0xA1, 0x84, 0x6E,
            0x61, 0x6D, 0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B,
        ])));

        let node: BoltNode = BoltNode::parse(Version::V4_1, input).unwrap();

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

        let bytes: Bytes = node.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3, 0x4E, 0x13, 0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, 0xA1, 0x84, 0x6E,
                0x61, 0x6D, 0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B,
            ])
        );
    }
}
