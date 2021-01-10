use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB3, 0x50)]
pub struct BoltPath {
    pub nodes: BoltList,
    pub rels: BoltList,
    pub ids: BoltList,
}

impl BoltPath {
    pub fn nodes(&self) -> Vec<BoltNode> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for bolt_type in self.nodes.iter() {
            if let BoltType::Node(node) = bolt_type {
                nodes.push(node.clone());
            }
        }
        nodes
    }

    pub fn rels(&self) -> Vec<BoltUnboundedRelation> {
        let mut rels = Vec::with_capacity(self.rels.len());
        for bolt_type in self.rels.iter() {
            if let BoltType::UnboundedRelation(rel) = bolt_type {
                rels.push(rel.clone());
            }
        }
        rels
    }

    pub fn ids(&self) -> Vec<BoltInteger> {
        let mut ids = Vec::with_capacity(self.ids.len());
        for bolt_type in self.ids.iter() {
            if let BoltType::Integer(id) = bolt_type {
                ids.push(id.clone());
            }
        }
        ids
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
    fn should_serialize_a_path() {
        let mark = BoltNode::new(
            42.into(),
            vec!["Person".into()].into(),
            vec![("name".into(), "Mark".into())].into_iter().collect(),
        );
        let james = BoltNode::new(
            43.into(),
            vec!["Person".into()].into(),
            vec![("name".into(), "James".into())].into_iter().collect(),
        );
        let friend = BoltUnboundedRelation::new(
            22.into(),
            "friend".into(),
            vec![("key".into(), "value".into())].into_iter().collect(),
        );

        let path = BoltPath {
            nodes: vec![mark.into(), james.into()].into(),
            rels: vec![friend.into()].into(),
            ids: vec![22.into(), 42.into()].into(),
        };

        let bytes: Bytes = path.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3, 0x50, 0x92, 0xB3, 0x4E, 0x2A, 0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E,
                0xA1, 0x84, 0x6E, 0x61, 0x6D, 0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B, 0xB3, 0x4E, 0x2B,
                0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, 0xA1, 0x84, 0x6E, 0x61, 0x6D, 0x65,
                0x85, 0x4A, 0x61, 0x6D, 0x65, 0x73, 0x91, 0xB3, 0x72, 0x16, 0x86, 0x66, 0x72, 0x69,
                0x65, 0x6E, 0x64, 0xA1, 0x83, 0x6B, 0x65, 0x79, 0x85, 0x76, 0x61, 0x6C, 0x75, 0x65,
                0x92, 0x16, 0x2A,
            ])
        );
    }

    #[test]
    fn should_deserialize_a_path() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x50, 0x92, 0xB3, 0x4E, 0x2A, 0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E,
            0xA1, 0x84, 0x6E, 0x61, 0x6D, 0x65, 0x84, 0x4D, 0x61, 0x72, 0x6B, 0xB3, 0x4E, 0x2B,
            0x91, 0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, 0xA1, 0x84, 0x6E, 0x61, 0x6D, 0x65,
            0x85, 0x4A, 0x61, 0x6D, 0x65, 0x73, 0x91, 0xB3, 0x72, 0x16, 0x86, 0x66, 0x72, 0x69,
            0x65, 0x6E, 0x64, 0xA1, 0x83, 0x6B, 0x65, 0x79, 0x85, 0x76, 0x61, 0x6C, 0x75, 0x65,
            0x92, 0x16, 0x2A,
        ])));

        let path: BoltPath = BoltPath::parse(Version::V4_1, input).unwrap();

        let nodes = path.nodes();
        let rels = path.rels();
        let ids = path.ids();
        assert_eq!(nodes.len(), 2);
        assert_eq!(rels.len(), 1);
        assert_eq!(ids.len(), 2);
        assert_eq!(
            nodes,
            vec![
                BoltNode::new(
                    42.into(),
                    vec!["Person".into()].into(),
                    vec![("name".into(), "Mark".into())].into_iter().collect(),
                ),
                BoltNode::new(
                    43.into(),
                    vec!["Person".into()].into(),
                    vec![("name".into(), "James".into())].into_iter().collect(),
                )
            ]
        );
        assert_eq!(
            rels,
            vec![BoltUnboundedRelation::new(
                22.into(),
                "friend".into(),
                vec![("key".into(), "value".into())].into_iter().collect(),
            )]
        );
        assert_eq!(ids, vec![22.into(), 42.into()]);
    }
}
