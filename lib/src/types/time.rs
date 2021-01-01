use crate::types::*;
use chrono::{NaiveTime, Timelike};
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, Hash, BoltStruct)]
#[signature(0xB2, 0x54)]
pub struct BoltTime {
    nanoseconds: BoltInteger,
    tz_offset_seconds: BoltInteger,
}

impl Into<BoltTime> for NaiveTime {
    fn into(self) -> BoltTime {
        let seconds_from_midnight = self.num_seconds_from_midnight() as i64;
        let nanoseconds = seconds_from_midnight * 1_000_000_000 + self.nanosecond() as i64;
        //No tz offset as the time is already in utc
        BoltTime {
            nanoseconds: nanoseconds.into(),
            tz_offset_seconds: 0.into(),
        }
    }
}

impl Into<NaiveTime> for BoltTime {
    fn into(self) -> NaiveTime {
        let nanos = self.nanoseconds.value - (self.tz_offset_seconds.value * 1_000_000_000);
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
        let utc_time = NaiveTime::from_hms_nano_opt(7, 8, 9, 100).unwrap();

        let time: BoltTime = utc_time.into();

        println!(
            "{:#04X?}",
            time.clone().to_bytes(Version::V4_1).unwrap().bytes()
        );

        assert_eq!(
            time.to_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[
                0xB2, 0x54, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64, 0x00,
            ])
        );
    }

    #[test]
    fn should_deserialize_time() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB2, 0x54, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64, 0x00,
        ])));

        let time: NaiveTime = BoltTime::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(time.to_string(), "07:08:09.000000100");
    }
}
