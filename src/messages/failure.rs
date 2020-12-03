use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x7F;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Failure {
    metadata: BoltMap,
}

impl Failure {
    pub fn new(metadata: BoltMap) -> Failure {
        Failure { metadata }
    }

    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
        let marker: u8 = input.borrow()[0];
        let signature: u8 = input.borrow()[1];
        (MARKER..=(MARKER | 0x0F)).contains(&marker) && signature == SIGNATURE
    }
}

impl Failure {
    pub fn code(&self) -> String {
        self.metadata.get("code").unwrap().try_into().unwrap() //TODO: unwrap
    }

    pub fn message(&self) -> String {
        self.metadata.get("message").unwrap().try_into().unwrap() //TODO: unwrap
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for Failure {
    type Error = Error;
    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<Failure> {
        let marker = input.borrow_mut().get_u8();
        let signature = input.borrow_mut().get_u8();
        Ok(Failure {
            metadata: input.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_deserialize_success() {
        let data = Bytes::from_static(&[
            0xB1, 0x7F, 0xA2, 0x84, 0x63, 0x6F, 0x64, 0x65, 0xD0, 0x25, 0x4E, 0x65, 0x6F, 0x2E,
            0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x45, 0x72, 0x72, 0x6F, 0x72, 0x2E, 0x53, 0x65,
            0x63, 0x75, 0x72, 0x69, 0x74, 0x79, 0x2E, 0x55, 0x6E, 0x61, 0x75, 0x74, 0x68, 0x6F,
            0x72, 0x69, 0x7A, 0x65, 0x64, 0x87, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0xD0,
            0x39, 0x54, 0x68, 0x65, 0x20, 0x63, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x20, 0x69, 0x73,
            0x20, 0x75, 0x6E, 0x61, 0x75, 0x74, 0x68, 0x6F, 0x72, 0x69, 0x7A, 0x65, 0x64, 0x20,
            0x64, 0x75, 0x65, 0x20, 0x74, 0x6F, 0x20, 0x61, 0x75, 0x74, 0x68, 0x65, 0x6E, 0x74,
            0x69, 0x63, 0x61, 0x74, 0x69, 0x6F, 0x6E, 0x20, 0x66, 0x61, 0x69, 0x6C, 0x75, 0x72,
            0x65, 0x2E,
        ]);

        let failure: Failure = Rc::new(RefCell::new(data)).try_into().unwrap();

        assert_eq!(failure.code(), "Neo.ClientError.Security.Unauthorized");
        assert_eq!(
            failure.message(),
            "The client is unauthorized due to authentication failure."
        );
    }
}
