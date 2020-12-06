use crate::types::*;
use std::convert::TryInto;

#[derive(Debug)]
pub struct Row {
    attributes: BoltMap,
}

#[derive(Debug)]
pub struct Node {
    data: BoltNode,
}

impl Node {
    pub fn new(data: BoltNode) -> Self {
        Node { data }
    }

    pub fn id(&self) -> i64 {
        self.data.id.value
    }

    pub fn labels(&self) -> Vec<String> {
        self.data.labels.iter().map(|l| l.to_string()).collect()
    }

    pub fn get<T: std::convert::TryFrom<BoltType>>(&self, key: &str) -> Option<T> {
        self.data.get(key)
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
