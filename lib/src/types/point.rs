use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB3, 0x58)]
pub struct BoltPoint2D {
    pub sr_id: BoltInteger,
    pub x: BoltFloat,
    pub y: BoltFloat,
}

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
    use crate::version::Version;
    use bytes::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn should_serialize_2d_point() {
        let sr_id = BoltInteger::new(42);
        let x = BoltFloat::new(1.0);
        let y = BoltFloat::new(2.0);

        let point = BoltPoint2D { sr_id, x, y };

        let bytes: Bytes = point.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            &bytes[..],
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

        let point: BoltPoint2D = BoltPoint2D::parse(Version::V4_1, input).unwrap();

        assert_eq!(point.sr_id, BoltInteger::new(42));
        assert_eq!(point.x, BoltFloat::new(1.0));
        assert_eq!(point.y, BoltFloat::new(2.0));
    }

    #[test]
    fn should_serialize_3d_point() {
        let sr_id = BoltInteger::new(42);
        let x = BoltFloat::new(1.0);
        let y = BoltFloat::new(2.0);
        let z = BoltFloat::new(3.0);

        let point = BoltPoint3D { sr_id, x, y, z };

        let bytes: Bytes = point.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            &bytes[..],
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

        let point: BoltPoint3D = BoltPoint3D::parse(Version::V4_1, input).unwrap();

        assert_eq!(point.sr_id, BoltInteger::new(42));
        assert_eq!(point.x, BoltFloat::new(1.0));
        assert_eq!(point.y, BoltFloat::new(2.0));
        assert_eq!(point.z, BoltFloat::new(3.0));
    }
}
