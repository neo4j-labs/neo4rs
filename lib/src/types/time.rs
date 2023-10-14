use crate::types::BoltInteger;
use chrono::{FixedOffset, NaiveTime, Offset, Timelike};
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB2, 0x54)]
pub struct BoltTime {
    pub(crate) nanoseconds: BoltInteger,
    pub(crate) tz_offset_seconds: BoltInteger,
}

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB1, 0x74)]
pub struct BoltLocalTime {
    pub(crate) nanoseconds: BoltInteger,
}

impl From<(NaiveTime, FixedOffset)> for BoltTime {
    fn from(value: (NaiveTime, FixedOffset)) -> Self {
        let seconds_from_midnight = value.0.num_seconds_from_midnight() as i64;
        let nanoseconds = seconds_from_midnight * 1_000_000_000 + value.0.nanosecond() as i64;
        BoltTime {
            nanoseconds: nanoseconds.into(),
            tz_offset_seconds: value.1.fix().local_minus_utc().into(),
        }
    }
}

impl From<NaiveTime> for BoltLocalTime {
    fn from(value: NaiveTime) -> Self {
        let seconds_from_midnight = value.num_seconds_from_midnight() as i64;
        let nanoseconds = seconds_from_midnight * 1_000_000_000 + value.nanosecond() as i64;
        BoltLocalTime {
            nanoseconds: nanoseconds.into(),
        }
    }
}

impl BoltTime {
    pub(crate) fn to_chrono(&self) -> (NaiveTime, FixedOffset) {
        self.into()
    }
}

impl BoltLocalTime {
    pub(crate) fn to_chrono(&self) -> NaiveTime {
        self.into()
    }
}

impl From<&BoltTime> for (NaiveTime, FixedOffset) {
    fn from(value: &BoltTime) -> Self {
        let time = BoltLocalTime {
            nanoseconds: value.nanoseconds.clone(),
        }
        .to_chrono();

        let offset = FixedOffset::east_opt(value.tz_offset_seconds.value as i32)
            .unwrap_or_else(|| panic!("invald timezone offset {}", value.tz_offset_seconds.value));

        (time, offset)
    }
}

impl From<BoltTime> for (NaiveTime, FixedOffset) {
    fn from(value: BoltTime) -> Self {
        (&value).into()
    }
}

impl From<&BoltLocalTime> for NaiveTime {
    fn from(value: &BoltLocalTime) -> Self {
        let nanos = value.nanoseconds.value;
        let seconds = (nanos / 1_000_000_000) as u32;
        let nanoseconds = (nanos % 1_000_000_000) as u32;
        NaiveTime::from_num_seconds_from_midnight_opt(seconds, nanoseconds).unwrap_or_else(|| {
            panic!(
                "invalid number of seconds {} and nanoseconds {}",
                seconds, nanoseconds
            )
        })
    }
}

impl From<BoltLocalTime> for NaiveTime {
    fn from(value: BoltLocalTime) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::BoltWireFormat, version::Version};
    use bytes::Bytes;

    #[test]
    fn should_serialize_time() {
        let time = NaiveTime::from_hms_nano_opt(7, 8, 9, 100).unwrap();
        let offset = FixedOffset::east_opt(2 * 3600).unwrap();

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
        let mut bytes = Bytes::from_static(&[
            0xB2, 0x54, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64, 0xC9, 0x1C, 0x20,
        ]);

        let (time, offset) = BoltTime::parse(Version::V4_1, &mut bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(time.to_string(), "07:08:09.000000100");
        assert_eq!(offset, FixedOffset::east_opt(2 * 3600).unwrap());
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
        let mut bytes = Bytes::from_static(&[
            0xB1, 0x74, 0xCB, 0x00, 0x00, 0x17, 0x5D, 0x2F, 0xB8, 0x3A, 0x64,
        ]);

        let time: NaiveTime = BoltLocalTime::parse(Version::V4_1, &mut bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(time.to_string(), "07:08:09.000000100");
    }
}
