use crate::error::*;
use crate::types::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x71;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Record {
    pub data: BoltList,
}

impl Record {
    pub fn new(data: BoltList) -> Record {
        Record { data }
    }
}

pub fn is_present(input: Rc<RefCell<Bytes>>) -> bool {
    let marker: u8 = input.borrow()[0];
    let signature: u8 = input.borrow()[1];
    (MARKER..=(MARKER | 0x0F)).contains(&marker) && signature == SIGNATURE
}

impl IntoIterator for Record {
    type Item = BoltType;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl Into<Vec<BoltType>> for Record {
    fn into(self) -> Vec<BoltType> {
        self.data.into()
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for Record {
    type Error = Error;
    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<Record> {
        let marker = input.borrow_mut().get_u8();
        let signature = input.borrow_mut().get_u8();
        Ok(Record {
            data: input.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize_record_message() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            MARKER, SIGNATURE, 0x92, 0x81, 0x61, 0x81, 0x62,
        ])));

        let record: Record = bytes.try_into().unwrap();

        assert_eq!(record.data.len(), 2);
    }
}
