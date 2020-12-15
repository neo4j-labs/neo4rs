use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::rc::Rc;

pub const MARKER: u8 = 0xB4;
pub const SIGNATURE: u8 = 0x59;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BoltPoint3D {
    pub sr_id: BoltInteger,
    pub x: BoltInteger,
    pub y: BoltInteger,
    pub z: BoltInteger,
}

impl BoltPoint3D {
    pub fn can_parse(input: Rc<RefCell<Bytes>>) -> bool {
        let input = input.borrow();
        input.len() > 1 && input[0] == MARKER && input[1] == SIGNATURE
    }
}

impl TryFrom<Rc<RefCell<Bytes>>> for BoltPoint3D {
    type Error = Error;

    fn try_from(input: Rc<RefCell<Bytes>>) -> Result<BoltPoint3D> {
        let marker = input.borrow_mut().get_u8();
        let tag = input.borrow_mut().get_u8();
        match (marker, tag) {
            (MARKER, SIGNATURE) => {
                let sr_id: BoltInteger = input.clone().try_into()?;
                let x: BoltInteger = input.clone().try_into()?;
                let y: BoltInteger = input.clone().try_into()?;
                let z: BoltInteger = input.clone().try_into()?;
                Ok(BoltPoint3D { sr_id, x, y, z })
            }
            _ => Err(Error::InvalidTypeMarker(format!(
                "invalid point3d marker/tag ({}, {})",
                marker, tag
            ))),
        }
    }
}

impl TryInto<Bytes> for BoltPoint3D {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let sr_id: Bytes = self.sr_id.try_into()?;
        let x: Bytes = self.x.try_into()?;
        let y: Bytes = self.y.try_into()?;
        let z: Bytes = self.z.try_into()?;

        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>()
                + mem::size_of::<u32>()
                + sr_id.len()
                + x.len()
                + y.len()
                + z.len(),
        );
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(sr_id);
        bytes.put(x);
        bytes.put(y);
        bytes.put(z);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_3d_point() {
        let sr_id = BoltInteger::new(42);
        let x = BoltInteger::new(1);
        let y = BoltInteger::new(2);
        let z = BoltInteger::new(3);

        let point = BoltPoint3D { sr_id, x, y, z };

        let bytes: Bytes = point.try_into().unwrap();

        println!("{:#04X?}", bytes.bytes());

        assert_eq!(
            bytes,
            Bytes::from_static(&[0xB4, 0x59, 0x2A, 0x01, 0x02, 0x03])
        );
    }

    #[test]
    fn should_deserialize_3d_point() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB4, 0x59, 0x2A, 0x01, 0x02, 0x03,
        ])));

        let point: BoltPoint3D = input.try_into().unwrap();

        assert_eq!(point.sr_id, BoltInteger::new(42));
        assert_eq!(point.x, BoltInteger::new(1));
        assert_eq!(point.y, BoltInteger::new(2));
        assert_eq!(point.z, BoltInteger::new(3));
    }
}
