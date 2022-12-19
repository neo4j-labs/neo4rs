use crate::errors::Error;
use crate::types::*;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Offset, Timelike};
use neo4rs_macros::BoltStruct;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB3, 0x46)]
pub struct BoltDateTime {
    seconds: BoltInteger,
    nanoseconds: BoltInteger,
    tz_offset_seconds: BoltInteger,
}

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB2, 0x64)]
pub struct BoltLocalDateTime {
    seconds: BoltInteger,
    nanoseconds: BoltInteger,
}

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB3, 0x66)]
pub struct BoltDateTimeZoneId {
    seconds: BoltInteger,
    nanoseconds: BoltInteger,
    tz_id: BoltString,
}

impl From<(NaiveDateTime, &str)> for BoltDateTimeZoneId {
    fn from(val: (NaiveDateTime, &str)) -> Self {
        let seconds = val.0.timestamp().into();
        let nanoseconds = (val.0.timestamp_subsec_nanos() as i64).into();
        BoltDateTimeZoneId {
            seconds,
            nanoseconds,
            tz_id: val.1.into(),
        }
    }
}

impl TryInto<(NaiveDateTime, String)> for BoltDateTimeZoneId {
    type Error = Error;

    fn try_into(self) -> Result<(NaiveDateTime, String)> {
        let datetime =
            NaiveDateTime::from_timestamp_opt(self.seconds.value, self.nanoseconds.value as u32)
                .ok_or_else(|| Error::ConvertError(BoltType::DateTimeZoneId(self.clone())))?;
        Ok((datetime, self.tz_id.into()))
    }
}

impl From<NaiveDateTime> for BoltLocalDateTime {
    fn from(val: NaiveDateTime) -> Self {
        let seconds = val.timestamp().into();
        let nanoseconds = (val.nanosecond() as i64).into();

        BoltLocalDateTime {
            seconds,
            nanoseconds,
        }
    }
}

impl TryInto<NaiveDateTime> for BoltLocalDateTime {
    type Error = Error;

    fn try_into(self) -> Result<NaiveDateTime> {
        NaiveDateTime::from_timestamp_opt(self.seconds.value, self.nanoseconds.value as u32)
            .ok_or_else(|| Error::ConvertError(BoltType::LocalDateTime(self.clone())))
    }
}

impl From<DateTime<FixedOffset>> for BoltDateTime {
    fn from(val: DateTime<FixedOffset>) -> Self {
        let seconds = (val.timestamp() + val.offset().fix().local_minus_utc() as i64).into();
        let nanoseconds = (val.nanosecond() as i64).into();
        let tz_offset_seconds = val.offset().fix().local_minus_utc().into();

        BoltDateTime {
            seconds,
            nanoseconds,
            tz_offset_seconds,
        }
    }
}

impl TryInto<DateTime<FixedOffset>> for BoltDateTime {
    type Error = Error;

    fn try_into(self) -> Result<DateTime<FixedOffset>> {
        let seconds = self.seconds.value - self.tz_offset_seconds.value;
        let datetime = NaiveDateTime::from_timestamp_opt(seconds, self.nanoseconds.value as u32)
            .ok_or_else(|| Error::ConvertError(BoltType::DateTime(self.clone())))?;
        Ok(DateTime::from_utc(
            datetime,
            FixedOffset::east_opt(self.tz_offset_seconds.value as i32)
                .ok_or_else(|| Error::ConvertError(BoltType::DateTime(self.clone())))?,
        ))
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
    fn should_serialize_a_datetime() {
        let date: BoltDateTime = DateTime::parse_from_rfc2822("Wed, 24 Jun 2015 12:50:35 +0100")
            .unwrap()
            .into();

        assert_eq!(
            date.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[
                0xB3, 0x46, 0xCA, 0x55, 0x8A, 0xA7, 0x9B, 0x00, 0xC9, 0x0E, 0x10,
            ])
        );
    }

    #[test]
    fn should_deserialize_a_datetime() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x46, 0xCA, 0x55, 0x8A, 0xA7, 0x9B, 0x00, 0xC9, 0x0E, 0x10,
        ])));

        let datetime: DateTime<FixedOffset> = BoltDateTime::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(datetime.to_rfc2822(), "Wed, 24 Jun 2015 12:50:35 +0100");
    }

    #[test]
    fn should_serialize_a_localdatetime() {
        let date: BoltLocalDateTime =
            NaiveDateTime::parse_from_str("2015-07-01 08:59:60.123", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap()
                .into();
        assert_eq!(
            date.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[
                0xB2, 0x64, 0xCA, 0x55, 0x93, 0xAC, 0x0F, 0xCA, 0x42, 0xEF, 0x9E, 0xC0,
            ])
        );
    }

    #[test]
    fn should_deserialize_a_localdatetime() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB2, 0x64, 0xCA, 0x55, 0x93, 0xAC, 0x0F, 0xCA, 0x42, 0xEF, 0x9E, 0xC0,
        ])));

        let datetime: NaiveDateTime = BoltLocalDateTime::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(datetime.to_string(), "2015-07-01 08:59:60.123");
    }

    #[test]
    fn should_serialize_a_datetime_with_zoneid() {
        let datetime =
            NaiveDateTime::parse_from_str("2015-07-01 08:59:60.123", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

        let date: BoltDateTimeZoneId = (datetime, "Europe/Paris").into();

        assert_eq!(
            date.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[
                0xB3, 0x66, 0xCA, 0x55, 0x93, 0xAC, 0x0F, 0xCA, 0x42, 0xEF, 0x9E, 0xC0, 0x8C, 0x45,
                0x75, 0x72, 0x6F, 0x70, 0x65, 0x2F, 0x50, 0x61, 0x72, 0x69, 0x73,
            ])
        );
    }

    #[test]
    fn should_deserialize_a_datetime_with_zoneid() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB3, 0x66, 0xCA, 0x55, 0x93, 0xAC, 0x0F, 0xCA, 0x42, 0xEF, 0x9E, 0xC0, 0x8C, 0x45,
            0x75, 0x72, 0x6F, 0x70, 0x65, 0x2F, 0x50, 0x61, 0x72, 0x69, 0x73,
        ])));

        let (datetime, zone_id) = BoltDateTimeZoneId::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(datetime.to_string(), "2015-07-01 08:59:60.123");
        assert_eq!(zone_id, "Europe/Paris");
    }
}
