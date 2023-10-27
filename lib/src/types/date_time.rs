use crate::errors::Error;
use crate::types::*;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Offset, Timelike};
use neo4rs_macros::BoltStruct;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB3, 0x46)]
pub struct BoltDateTime {
    pub(crate) seconds: BoltInteger,
    pub(crate) nanoseconds: BoltInteger,
    pub(crate) tz_offset_seconds: BoltInteger,
}

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB2, 0x64)]
pub struct BoltLocalDateTime {
    pub(crate) seconds: BoltInteger,
    pub(crate) nanoseconds: BoltInteger,
}

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB3, 0x66)]
pub struct BoltDateTimeZoneId {
    pub(crate) seconds: BoltInteger,
    pub(crate) nanoseconds: BoltInteger,
    pub(crate) tz_id: BoltString,
}

impl BoltDateTime {
    pub(crate) fn try_to_chrono(&self) -> Result<DateTime<FixedOffset>> {
        self.try_into()
    }
}

impl BoltLocalDateTime {
    pub(crate) fn try_to_chrono(&self) -> Result<NaiveDateTime> {
        self.try_into()
    }
}

impl BoltDateTimeZoneId {
    pub(crate) fn try_to_chrono(&self) -> Result<DateTime<FixedOffset>> {
        self.try_into()
    }

    pub fn tz_id(&self) -> &str {
        &self.tz_id.value
    }
}

impl From<(NaiveDateTime, &str)> for BoltDateTimeZoneId {
    fn from(value: (NaiveDateTime, &str)) -> Self {
        let seconds = value.0.timestamp().into();
        let nanoseconds = (value.0.timestamp_subsec_nanos() as i64).into();
        BoltDateTimeZoneId {
            seconds,
            nanoseconds,
            tz_id: value.1.into(),
        }
    }
}

impl TryInto<(NaiveDateTime, String)> for BoltDateTimeZoneId {
    type Error = Error;

    fn try_into(self) -> Result<(NaiveDateTime, String)> {
        NaiveDateTime::from_timestamp_opt(self.seconds.value, self.nanoseconds.value as u32)
            .map(|datetime| (datetime, self.tz_id.into()))
            .ok_or(Error::ConversionError)
    }
}

impl TryFrom<&BoltDateTimeZoneId> for NaiveDateTime {
    type Error = Error;

    fn try_from(value: &BoltDateTimeZoneId) -> Result<Self, Self::Error> {
        NaiveDateTime::from_timestamp_opt(value.seconds.value, value.nanoseconds.value as u32)
            .ok_or(Error::ConversionError)
    }
}

impl TryFrom<&BoltDateTimeZoneId> for DateTime<FixedOffset> {
    type Error = Error;

    fn try_from(value: &BoltDateTimeZoneId) -> std::result::Result<Self, Self::Error> {
        let tz: chrono_tz::Tz = value
            .tz_id
            .value
            .parse()
            .map_err(|_| Error::ConversionError)?;

        let seconds = value.seconds.value;
        let nanoseconds = value.nanoseconds.value as u32;

        let dt = NaiveDateTime::from_timestamp_opt(seconds, nanoseconds)
            .ok_or(Error::ConversionError)?
            .and_local_timezone(tz)
            .earliest()
            .ok_or(Error::ConversionError)?;

        let dt = dt.with_timezone(&dt.offset().fix());

        Ok(dt)
    }
}

impl From<NaiveDateTime> for BoltLocalDateTime {
    fn from(value: NaiveDateTime) -> Self {
        let seconds = value.timestamp().into();
        let nanoseconds = (value.nanosecond() as i64).into();

        BoltLocalDateTime {
            seconds,
            nanoseconds,
        }
    }
}

impl TryInto<NaiveDateTime> for BoltLocalDateTime {
    type Error = Error;

    fn try_into(self) -> Result<NaiveDateTime> {
        (&self).try_into()
    }
}

impl TryFrom<&BoltLocalDateTime> for NaiveDateTime {
    type Error = Error;

    fn try_from(value: &BoltLocalDateTime) -> std::result::Result<Self, Self::Error> {
        NaiveDateTime::from_timestamp_opt(value.seconds.value, value.nanoseconds.value as u32)
            .ok_or(Error::ConversionError)
    }
}

impl From<DateTime<FixedOffset>> for BoltDateTime {
    fn from(value: DateTime<FixedOffset>) -> Self {
        let seconds = (value.timestamp() + value.offset().fix().local_minus_utc() as i64).into();
        let nanoseconds = (value.nanosecond() as i64).into();
        let tz_offset_seconds = value.offset().fix().local_minus_utc().into();

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
        (&self).try_into()
    }
}

impl TryFrom<&BoltDateTime> for DateTime<FixedOffset> {
    type Error = Error;

    fn try_from(
        BoltDateTime {
            seconds,
            nanoseconds,
            tz_offset_seconds,
        }: &BoltDateTime,
    ) -> std::result::Result<Self, Self::Error> {
        let seconds = seconds.value - tz_offset_seconds.value;
        let offset =
            FixedOffset::east_opt(tz_offset_seconds.value as i32).ok_or(Error::ConversionError)?;
        let datetime = NaiveDateTime::from_timestamp_opt(seconds, nanoseconds.value as u32)
            .ok_or(Error::ConversionError)?;

        Ok(DateTime::from_utc(datetime, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::Bytes;

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
        let mut bytes = Bytes::from_static(&[
            0xB3, 0x46, 0xCA, 0x55, 0x8A, 0xA7, 0x9B, 0x00, 0xC9, 0x0E, 0x10,
        ]);

        let datetime: DateTime<FixedOffset> = BoltDateTime::parse(Version::V4_1, &mut bytes)
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
        let mut bytes = Bytes::from_static(&[
            0xB2, 0x64, 0xCA, 0x55, 0x93, 0xAC, 0x0F, 0xCA, 0x42, 0xEF, 0x9E, 0xC0,
        ]);

        let datetime: NaiveDateTime = BoltLocalDateTime::parse(Version::V4_1, &mut bytes)
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
        let mut bytes = Bytes::from_static(&[
            0xB3, 0x66, 0xCA, 0x55, 0x93, 0xAC, 0x0F, 0xCA, 0x42, 0xEF, 0x9E, 0xC0, 0x8C, 0x45,
            0x75, 0x72, 0x6F, 0x70, 0x65, 0x2F, 0x50, 0x61, 0x72, 0x69, 0x73,
        ]);

        let (datetime, zone_id) = BoltDateTimeZoneId::parse(Version::V4_1, &mut bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(datetime.to_string(), "2015-07-01 08:59:60.123");
        assert_eq!(zone_id, "Europe/Paris");
    }
}
