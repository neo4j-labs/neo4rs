use crate::types::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB4;
pub const SIGNATURE: u8 = 0x59;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB4, 0x59)]
pub struct BoltPoint3D {
    pub sr_id: BoltInteger,
    pub x: BoltFloat,
    pub y: BoltFloat,
    pub z: BoltFloat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
    use std::cell::RefCell;
    use std::convert::TryInto;
    use std::rc::Rc;

    #[test]
    fn should_serialize_3d_point() {
        let sr_id = BoltInteger::new(42);
        let x = BoltFloat::new(1.0);
        let y = BoltFloat::new(2.0);
        let z = BoltFloat::new(3.0);

        let point = BoltPoint3D { sr_id, x, y, z };

        let bytes: Bytes = point.try_into().unwrap();

        println!("{:#04X?}", bytes.bytes());

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB4, 0x59, 0x2A, 0xC1, 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0x40,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0x40, 0x08, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ])
        );
    }

    #[test]
    fn should_deserialize_3d_point() {
        let input = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB4, 0x59, 0x2A, 0xC1, 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0x40,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0x40, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])));

        let point: BoltPoint3D = input.try_into().unwrap();

        assert_eq!(point.sr_id, BoltInteger::new(42));
        assert_eq!(point.x, BoltFloat::new(1.0));
        assert_eq!(point.y, BoltFloat::new(2.0));
        assert_eq!(point.z, BoltFloat::new(3.0));
    }
}
