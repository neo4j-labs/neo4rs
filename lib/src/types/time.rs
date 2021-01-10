use crate::types::*;
use chrono::{FixedOffset, NaiveTime, Offset, Timelike};
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB2, 0x54)]
pub struct BoltTime {
    nanoseconds: BoltInteger,
    tz_offset_seconds: BoltInteger,
}

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x74)]
pub struct BoltLocalTime {
    nanoseconds: BoltInteger,
}

impl Into<BoltTime> for (NaiveTime, FixedOffset) {
    fn into(self) -> BoltTime {
        let seconds_from_midnight = self.0.num_seconds_from_midnight() as i64;
        let nanoseconds = seconds_from_midnight * 1_000_000_000 + self.0.nanosecond() as i64;
        BoltTime {
            nanoseconds: nanoseconds.into(),
            tz_offset_seconds: self.1.fix().local_minus_utc().into(),
        }
    }
}

impl Into<(NaiveTime, FixedOffset)> for BoltTime {
    fn into(self) -> (NaiveTime, FixedOffset) {
        let nanos = self.nanoseconds.value;
        let seconds = (nanos / 1_000_000_000) as u32;
        let nanoseconds = (nanos % 1_000_000_000) as u32;
        (
            NaiveTime::from_num_seconds_from_midnight(seconds, nanoseconds),
            FixedOffset::east(self.tz_offset_seconds.value as i32),
        )
    }
}

impl Into<BoltLocalTime> for NaiveTime {
    fn into(self) -> BoltLocalTime {
        let seconds_from_midnight = self.num_seconds_from_midnight() as i64;
        let nanoseconds = seconds_from_midnight * 1_000_000_000 + self.nanosecond() as i64;
        BoltLocalTime {
            nanoseconds: nanoseconds.into(),
        }
    }
}

impl Into<NaiveTime> for BoltLocalTime {
    fn into(self) -> NaiveTime {
        let nanos = self.nanoseconds.value;
        let seconds = (nanos / 1_000_000_000) as u32;
        let nanoseconds = (nanos % 1_000_000_000) as u32;
        NaiveTime::from_num_seconds_from_midnight(seconds, nanoseconds)
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
    fn should_serialize_time() {
        let time = NaiveTime::from_hms_nano_opt(7, 8, 9, 100).unwrap();
        let offset = FixedOffset::east(2 * 3600);

        let time: BoltTime = (time, offset).into();

        assert_eq!(
            time.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[
                0xB2, 0x54, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64, 0xC9, 0x1C, 0x20,
            ])
        );
    }

    #[test]
    fn should_deserialize_time() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB2, 0x54, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64, 0xC9, 0x1C, 0x20,
        ])));

        let (time, offset) = BoltTime::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(time.to_string(), "07:08:09.000000100");
        assert_eq!(offset, FixedOffset::east(2 * 3600));
    }

    #[test]
    fn should_serialize_local_time() {
        let naive_time = NaiveTime::from_hms_nano_opt(7, 8, 9, 100).unwrap();

        let time: BoltLocalTime = naive_time.into();

        assert_eq!(
            time.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[
                0xB1, 0x74, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64,
            ])
        );
    }

    #[test]
    fn should_deserialize_local_time() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB1, 0x74, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64,
        ])));

        let time: NaiveTime = BoltLocalTime::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(time.to_string(), "07:08:09.000000100");
    }
}
