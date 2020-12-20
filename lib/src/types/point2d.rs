use crate::types::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB3;
pub const SIGNATURE: u8 = 0x58;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
pub struct BoltPoint2D {
    pub sr_id: BoltInteger,
    pub x: BoltFloat,
    pub y: BoltFloat,
}

impl BoltPoint2D {
    fn marker() -> (u8, Option<u8>) {
        (MARKER, Some(SIGNATURE))
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
    fn should_serialize_2d_point() {
        let sr_id = BoltInteger::new(42);
        let x = BoltFloat::new(1.0);
        let y = BoltFloat::new(2.0);

        let point = BoltPoint2D { sr_id, x, y };

        let bytes: Bytes = point.try_into().unwrap();

        println!("{:#04X?}", bytes.bytes());

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3, 0x58, 0x2A, 0xC1, 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0x40,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        );
    }

    #[test]
    fn should_deserialize_2d_point() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x58, 0x2A, 0xC1, 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0x40,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ])));

        let point: BoltPoint2D = input.try_into().unwrap();

        assert_eq!(point.sr_id, BoltInteger::new(42));
        assert_eq!(point.x, BoltFloat::new(1.0));
        assert_eq!(point.y, BoltFloat::new(2.0));
    }
}
