use std::time::Duration;

use serde::de::{Deserialize, Deserializer};

use super::de::impl_visitor;

/// An instant capturing the time of day, and the timezone, but not the date.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Time {
    nanoseconds: u64,
    tz_offset_seconds: i32,
}

impl Time {
    /// Nanoseconds since midnight in the timezone of this time, not in UTC.
    pub fn nanoseconds_since_midnight(self) -> u64 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_offset_seconds(self) -> i32 {
        self.tz_offset_seconds
    }

    /// Returns the duration since midnight in the timezone of this time, not in UTC.
    pub fn as_duration(self) -> Duration {
        Duration::from_nanos(self.nanoseconds_since_midnight())
    }

    pub fn as_time_time(self) -> Option<time::OffsetDateTime> {
        Some(
            time::OffsetDateTime::from_unix_timestamp_nanos(self.nanoseconds.into())
                .ok()?
                .replace_offset(time::UtcOffset::from_whole_seconds(self.tz_offset_seconds).ok()?),
        )
    }

    pub fn as_chrono_time(self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        let time = chrono::NaiveTime::from_num_seconds_from_midnight_opt(
            u32::try_from(self.nanoseconds / 1_000_000_000).ok()?,
            u32::try_from(self.nanoseconds % 1_000_000_000).ok()?,
        )?;
        chrono::NaiveDateTime::new(chrono::NaiveDate::from_yo_opt(1970, 1).unwrap(), time)
            .and_local_timezone(chrono::FixedOffset::east_opt(self.tz_offset_seconds)?)
            .single()
    }
}

impl_visitor!(Time(nanoseconds, tz_offset_seconds) == 0x54);

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Time", &[], Self::visitor())
    }
}

/// An instant capturing the time of day, but neither the date nor the time zone.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LocalTime {
    nanoseconds: i64,
}

impl LocalTime {
    /// Nanoseconds since midnight.
    pub fn nanoseconds_since_midnight(self) -> u64 {
        self.nanoseconds.unsigned_abs()
    }

    /// Returns the duration since midnight.
    pub fn as_duration(self) -> Duration {
        Duration::from_nanos(self.nanoseconds_since_midnight())
    }

    pub fn as_time_time(self) -> Option<time::Time> {
        let nanos = self.nanoseconds_since_midnight();
        let hours = u8::try_from(nanos / 3_600_000_000_000).ok()?;
        let minutes = u8::try_from((nanos % 3_600_000_000_000) / 60_000_000_000).ok()?;
        let secs = u8::try_from((nanos % 60_000_000_000) / 1_000_000_000).ok()?;
        let nanos = (nanos % 1_000_000_000) as u32; // safe because mod 1e9

        time::Time::from_hms_nano(hours, minutes, secs, nanos).ok()
    }

    pub fn as_chrono_time(self) -> Option<chrono::NaiveTime> {
        let nanos = self.nanoseconds_since_midnight();
        chrono::NaiveTime::from_num_seconds_from_midnight_opt(
            u32::try_from(nanos / 1_000_000_000).ok()?,
            u32::try_from(nanos % 1_000_000_000).ok()?,
        )
    }
}

impl_visitor!(LocalTime(nanoseconds) == 0x74);

impl<'de> Deserialize<'de> for LocalTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("LocalTime", &[], Self::visitor())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use chrono::{Datelike, FixedOffset, Timelike};
    use time::UtcOffset;

    use crate::packstream::{bolt, from_bytes_ref, Data};

    use super::*;

    #[test]
    fn deserialize_time() {
        let data = bolt_time();
        let mut data = Data::new(data);
        let time: Time = from_bytes_ref(&mut data).unwrap();

        assert_eq!(time.nanoseconds_since_midnight(), 1337000420000_u64);
        assert_eq!(time.timezone_offset_seconds(), -7200_i32);
        assert_eq!(
            time.as_duration(),
            Duration::from_secs(1337).saturating_add(Duration::from_nanos(420_000))
        );

        let ch: chrono::DateTime<FixedOffset> = time.as_chrono_time().unwrap();
        assert_eq!(ch.num_seconds_from_midnight(), 1337);
        assert_eq!(ch.second(), 17);
        assert_eq!(ch.minute(), 22);
        assert_eq!(ch.hour(), 0);
        assert_eq!(ch.day(), 1);
        assert_eq!(ch.month(), 1);
        assert_eq!(ch.year(), 1970);
        assert_eq!(ch.nanosecond(), 420_000);
        assert_eq!(ch.timezone(), FixedOffset::west_opt(7200).unwrap());

        let tm: time::OffsetDateTime = time.as_time_time().unwrap();
        assert_eq!(tm.second(), 17);
        assert_eq!(tm.minute(), 22);
        assert_eq!(tm.hour(), 0);
        assert_eq!(tm.day(), 1);
        assert_eq!(tm.month(), time::Month::January);
        assert_eq!(tm.year(), 1970);
        assert_eq!(tm.nanosecond(), 420_000);
        assert_eq!(tm.offset(), UtcOffset::from_whole_seconds(-7200).unwrap());
    }

    fn bolt_time() -> Bytes {
        bolt()
            .structure(2, 0x54)
            .int64(1_337_000_420_000)
            .int16(-7200)
            .build()
    }

    #[test]
    fn deserialize_local_time() {
        let data = bolt_local_time();
        let mut data = Data::new(data);
        let time: LocalTime = from_bytes_ref(&mut data).unwrap();

        assert_eq!(time.nanoseconds_since_midnight(), 1337000420000_u64);
        assert_eq!(
            time.as_duration(),
            Duration::from_secs(1337).saturating_add(Duration::from_nanos(420_000))
        );

        let ch: chrono::NaiveTime = time.as_chrono_time().unwrap();
        assert_eq!(ch.num_seconds_from_midnight(), 1337);
        assert_eq!(ch.second(), 17);
        assert_eq!(ch.minute(), 22);
        assert_eq!(ch.hour(), 0);
        assert_eq!(ch.nanosecond(), 420_000);

        let tm: time::Time = time.as_time_time().unwrap();
        assert_eq!(tm.second(), 17);
        assert_eq!(tm.minute(), 22);
        assert_eq!(tm.hour(), 0);
        assert_eq!(tm.nanosecond(), 420_000);
    }

    fn bolt_local_time() -> Bytes {
        bolt().structure(1, 0x74).int64(1_337_000_420_000).build()
    }
}
