use crate::types::*;
use std::convert::TryInto;

#[derive(Debug)]
pub struct Row {
    attributes: BoltMap,
}

#[derive(Debug)]
pub struct Node {
    inner: BoltNode,
}

#[derive(Debug)]
pub struct Path {
    inner: BoltPath,
}

#[derive(Debug)]
pub struct Relation {
    inner: BoltRelation,
}

#[derive(Debug)]
pub struct UnboundedRelation {
    inner: BoltUnboundedRelation,
}

pub struct Point2D {
    inner: BoltPoint2D,
}

pub struct Point3D {
    inner: BoltPoint3D,
}

impl Path {
    pub fn new(inner: BoltPath) -> Self {
        Path { inner }
    }

    pub fn ids(&self) -> Vec<i64> {
        let bolt_ids = self.inner.ids();
        bolt_ids.into_iter().map(|id| id.value).collect()
    }

    pub fn nodes(&self) -> Vec<Node> {
        let nodes = self.inner.nodes();
        nodes.into_iter().map(|n| Node::new(n)).collect()
    }

    pub fn rels(&self) -> Vec<UnboundedRelation> {
        let rels = self.inner.rels();
        rels.into_iter()
            .map(|r| UnboundedRelation::new(r))
            .collect()
    }
}

impl Point2D {
    pub fn new(inner: BoltPoint2D) -> Self {
        Point2D { inner }
    }

    pub fn sr_id(&self) -> i64 {
        self.inner.sr_id.value
    }

    pub fn x(&self) -> f64 {
        self.inner.x.value
    }

    pub fn y(&self) -> f64 {
        self.inner.y.value
    }
}

impl Point3D {
    pub fn new(inner: BoltPoint3D) -> Self {
        Point3D { inner }
    }

    pub fn sr_id(&self) -> i64 {
        self.inner.sr_id.value
    }

    pub fn x(&self) -> f64 {
        self.inner.x.value
    }

    pub fn y(&self) -> f64 {
        self.inner.y.value
    }

    pub fn z(&self) -> f64 {
        self.inner.z.value
    }
}

impl Row {
    pub fn new(fields: BoltList, data: BoltList) -> Self {
        let mut attributes = BoltMap::with_capacity(fields.len());
        for (field, value) in fields.into_iter().zip(data.into_iter()) {
            if let Ok(key) = field.try_into() {
                attributes.put(key, value);
            }
        }
        Row { attributes }
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.attributes.get(key)
    }
}

impl Node {
    pub fn new(inner: BoltNode) -> Self {
        Node { inner }
    }

    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    pub fn labels(&self) -> Vec<String> {
        self.inner.labels.iter().map(|l| l.to_string()).collect()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }
}

impl Relation {
    pub fn new(inner: BoltRelation) -> Self {
        Relation { inner }
    }

    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    pub fn start_node_id(&self) -> i64 {
        self.inner.start_node_id.value
    }

    pub fn end_node_id(&self) -> i64 {
        self.inner.end_node_id.value
    }

    pub fn typ(&self) -> String {
        self.inner.typ.value.clone()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }
}

impl UnboundedRelation {
    pub fn new(inner: BoltUnboundedRelation) -> Self {
        UnboundedRelation { inner }
    }

    pub fn id(&self) -> i64 {
        self.inner.id.value
    }

    pub fn typ(&self) -> String {
        self.inner.typ.value.clone()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.inner.get(key)
    }
}
