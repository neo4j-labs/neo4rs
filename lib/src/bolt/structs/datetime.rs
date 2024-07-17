use std::str::FromStr;

use serde::de::{Deserialize, Deserializer};

use super::de::{impl_visitor, impl_visitor_ref};

/// An instant capturing the date, the time, and the time zone.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DateTime {
    seconds: i64,
    nanoseconds: u32,
    tz_offset_seconds: i32,
}

impl DateTime {
    /// Seconds since Unix epoch in UTC, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_offset_seconds(&self) -> i32 {
        self.tz_offset_seconds
    }

    pub fn as_time_datetime(&self) -> Option<time::OffsetDateTime> {
        let (dt, tz) =
            convert_to_time_datetime(self.seconds, self.nanoseconds, self.tz_offset_seconds)?;
        dt.checked_to_offset(tz)
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        let (dt, tz) =
            convert_to_chrono_datetime(self.seconds, self.nanoseconds, self.tz_offset_seconds)?;
        Some(dt.with_timezone(&tz))
    }
}

/// An instant capturing the date, the time, and the time zone specified with a timezone
/// iddentifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DateTimeZoneIdRef<'de> {
    seconds: i64,
    nanoseconds: u32,
    tz_id: &'de str,
}

impl<'de> DateTimeZoneIdRef<'de> {
    /// Seconds since Unix epoch in UTC, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_identifier(&self) -> &'de str {
        self.tz_id
    }

    /// The timezone offset in seconds from UTC according to the IANA Time Zone Database.
    /// If the value could not be parsed or is unknown, None is returned.
    pub fn timezone_offset_seconds(&self) -> Option<i32> {
        let tz = chrono_tz::Tz::from_str(self.tz_id).ok()?;
        let offset =
            chrono::TimeZone::offset_from_utc_datetime(&tz, &chrono::NaiveDateTime::UNIX_EPOCH);
        let offset = chrono::Offset::fix(&offset);
        Some(offset.local_minus_utc())
    }

    pub fn as_time_datetime(&self) -> Option<time::OffsetDateTime> {
        let offset = self.timezone_offset_seconds()?;
        let (dt, tz) = convert_to_time_datetime(self.seconds, self.nanoseconds, offset)?;
        dt.checked_to_offset(tz)
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::DateTime<chrono_tz::Tz>> {
        let tz = chrono_tz::Tz::from_str(self.tz_id).ok()?;
        let datetime = chrono::DateTime::from_timestamp(self.seconds, self.nanoseconds)?;
        Some(datetime.with_timezone(&tz))
    }

    pub fn to_owned(&self) -> DateTimeZoneId {
        DateTimeZoneId {
            seconds: self.seconds,
            nanoseconds: self.nanoseconds,
            tz_id: self.tz_id.to_owned(),
        }
    }
}

/// An instant capturing the date, the time, and the time zone.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LocalDateTime {
    seconds: i64,
    nanoseconds: u32,
}

impl LocalDateTime {
    /// Seconds since Unix epoch, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    pub fn as_time_datetime(&self) -> Option<time::PrimitiveDateTime> {
        let (dt, _tz) = convert_to_time_datetime(self.seconds, self.nanoseconds, 0)?;
        Some(time::PrimitiveDateTime::new(dt.date(), dt.time()))
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::NaiveDateTime> {
        let (dt, _tz) = convert_to_chrono_datetime(self.seconds, self.nanoseconds, 0)?;
        Some(dt.naive_utc())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LegacyDateTime {
    seconds: i64,
    nanoseconds: u32,
    tz_offset_seconds: i32,
}

impl LegacyDateTime {
    /// Seconds since Unix epoch in the timezone of this DateTime, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_offset_seconds(&self) -> i32 {
        self.tz_offset_seconds
    }

    pub fn as_time_datetime(&self) -> Option<time::OffsetDateTime> {
        let (dt, tz) =
            convert_to_time_datetime(self.seconds, self.nanoseconds, self.tz_offset_seconds)?;
        Some(dt.replace_offset(tz))
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        let (dt, tz) =
            convert_to_chrono_datetime(self.seconds, self.nanoseconds, self.tz_offset_seconds)?;
        chrono::TimeZone::from_local_datetime(&tz, &dt.naive_utc()).single()
    }
}

/// An instant capturing the date, the time, and the time zone specified with a timezone
/// iddentifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LegacyDateTimeZoneIdRef<'de> {
    seconds: i64,
    nanoseconds: u32,
    tz_id: &'de str,
}

impl<'de> LegacyDateTimeZoneIdRef<'de> {
    /// Seconds since Unix epoch in the timezone of this DateTime, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_identifier(&self) -> &'de str {
        self.tz_id
    }

    /// The timezone offset in seconds from UTC according to the IANA Time Zone Database.
    /// If the value could not be parsed or is unknown, None is returned.
    pub fn timezone_offset_seconds(&self) -> Option<i32> {
        let tz = chrono_tz::Tz::from_str(self.tz_id).ok()?;
        let offset =
            chrono::TimeZone::offset_from_utc_datetime(&tz, &chrono::NaiveDateTime::UNIX_EPOCH);
        let offset = chrono::Offset::fix(&offset);
        Some(offset.local_minus_utc())
    }

    pub fn as_time_datetime(&self) -> Option<time::OffsetDateTime> {
        let offset = self.timezone_offset_seconds()?;
        let (dt, tz) = convert_to_time_datetime(self.seconds, self.nanoseconds, offset)?;
        Some(dt.replace_offset(tz))
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::DateTime<chrono_tz::Tz>> {
        let tz = chrono_tz::Tz::from_str(self.tz_id).ok()?;
        let dt = chrono::DateTime::from_timestamp(self.seconds, self.nanoseconds)?;
        chrono::TimeZone::from_local_datetime(&tz, &dt.naive_utc()).single()
    }

    pub fn to_owned(&self) -> LegacyDateTimeZoneId {
        LegacyDateTimeZoneId {
            seconds: self.seconds,
            nanoseconds: self.nanoseconds,
            tz_id: self.tz_id.to_owned(),
        }
    }
}

fn convert_to_time_datetime(
    seconds: i64,
    nanoseconds: u32,
    tz_offset_seconds: i32,
) -> Option<(time::OffsetDateTime, time::UtcOffset)> {
    let nanos_since_epoch = i128::from(seconds).checked_mul(1_000_000_000)?;
    let nanos_since_epoch = nanos_since_epoch.checked_add(i128::from(nanoseconds))?;
    let datetime = time::OffsetDateTime::from_unix_timestamp_nanos(nanos_since_epoch).ok()?;
    let timezone = time::UtcOffset::from_whole_seconds(tz_offset_seconds).ok()?;
    Some((datetime, timezone))
}

fn convert_to_chrono_datetime(
    seconds: i64,
    nanoseconds: u32,
    tz_offset_seconds: i32,
) -> Option<(chrono::DateTime<chrono::Utc>, chrono::FixedOffset)> {
    let datetime = chrono::DateTime::from_timestamp(seconds, nanoseconds)?;
    let timezone = chrono::FixedOffset::east_opt(tz_offset_seconds)?;
    Some((datetime, timezone))
}

impl_visitor!(DateTime(seconds, nanoseconds, tz_offset_seconds) == 0x49);
impl_visitor_ref!(DateTimeZoneIdRef<'de>(seconds, nanoseconds, tz_id) == 0x69);
impl_visitor!(LocalDateTime(seconds, nanoseconds) == 0x64);
impl_visitor!(LegacyDateTime(seconds, nanoseconds, tz_offset_seconds) == 0x46);
impl_visitor_ref!(LegacyDateTimeZoneIdRef<'de>(seconds, nanoseconds, tz_id) == 0x66);

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("DateTime", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for DateTimeZoneIdRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("DateTimeZoneId", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for LocalDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("LocalDateTime", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for LegacyDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("LegacyDateTime", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for LegacyDateTimeZoneIdRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("LegacyDateTimeZoneId", &[], Self::visitor())
    }
}

/// An instant capturing the date, the time, and the time zone specified with a timezone
/// iddentifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTimeZoneId {
    seconds: i64,
    nanoseconds: u32,
    tz_id: String,
}

impl DateTimeZoneId {
    /// Seconds since Unix epoch in UTC, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_identifier(&self) -> &str {
        &self.tz_id
    }

    /// The timezone offset in seconds from UTC according to the IANA Time Zone Database.
    /// If the value could not be parsed or is unknown, None is returned.
    pub fn timezone_offset_seconds(&self) -> Option<i32> {
        let tz = chrono_tz::Tz::from_str(&self.tz_id).ok()?;
        let offset =
            chrono::TimeZone::offset_from_utc_datetime(&tz, &chrono::NaiveDateTime::UNIX_EPOCH);
        let offset = chrono::Offset::fix(&offset);
        Some(offset.local_minus_utc())
    }

    pub fn as_time_datetime(&self) -> Option<time::OffsetDateTime> {
        let offset = self.timezone_offset_seconds()?;
        let (dt, tz) = convert_to_time_datetime(self.seconds, self.nanoseconds, offset)?;
        dt.checked_to_offset(tz)
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::DateTime<chrono_tz::Tz>> {
        let tz = chrono_tz::Tz::from_str(&self.tz_id).ok()?;
        let datetime = chrono::DateTime::from_timestamp(self.seconds, self.nanoseconds)?;
        Some(datetime.with_timezone(&tz))
    }
}

/// An instant capturing the date, the time, and the time zone specified with a timezone
/// iddentifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyDateTimeZoneId {
    seconds: i64,
    nanoseconds: u32,
    tz_id: String,
}

impl LegacyDateTimeZoneId {
    /// Seconds since Unix epoch in the timezone of this DateTime, e.g. 0 represents 1970-01-01T00:00:01 and 1 represents 1970-01-01T00:00:02.
    pub fn seconds_since_epoch(&self) -> i64 {
        self.seconds
    }

    /// Nanoseconds since the last whole second.
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    /// The timezone offset in seconds from UTC.
    pub fn timezone_identifier(&self) -> &str {
        &self.tz_id
    }

    /// The timezone offset in seconds from UTC according to the IANA Time Zone Database.
    /// If the value could not be parsed or is unknown, None is returned.
    pub fn timezone_offset_seconds(&self) -> Option<i32> {
        let tz = chrono_tz::Tz::from_str(&self.tz_id).ok()?;
        let offset =
            chrono::TimeZone::offset_from_utc_datetime(&tz, &chrono::NaiveDateTime::UNIX_EPOCH);
        let offset = chrono::Offset::fix(&offset);
        Some(offset.local_minus_utc())
    }

    pub fn as_time_datetime(&self) -> Option<time::OffsetDateTime> {
        let offset = self.timezone_offset_seconds()?;
        let (dt, tz) = convert_to_time_datetime(self.seconds, self.nanoseconds, offset)?;
        Some(dt.replace_offset(tz))
    }

    pub fn as_chrono_datetime(&self) -> Option<chrono::DateTime<chrono_tz::Tz>> {
        let tz = chrono_tz::Tz::from_str(&self.tz_id).ok()?;
        let dt = chrono::DateTime::from_timestamp(self.seconds, self.nanoseconds)?;
        chrono::TimeZone::from_local_datetime(&tz, &dt.naive_utc()).single()
    }
}

impl_visitor!(DateTimeZoneId(seconds, nanoseconds, tz_id) == 0x69);
impl_visitor!(LegacyDateTimeZoneId(seconds, nanoseconds, tz_id) == 0x66);

impl<'de> Deserialize<'de> for DateTimeZoneId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("DateTimeZoneId", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for LegacyDateTimeZoneId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("LegacyDateTimeZoneId", &[], Self::visitor())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, FixedOffset, Timelike};

    use crate::packstream::{bolt, from_bytes_ref, Data};

    use super::*;

    #[test]
    fn deserialize_datetime() {
        let data = bolt()
            .structure(3, 0x49)
            .int16(4500)
            .tiny_int(42)
            .int16(3600)
            .build();
        let mut data = Data::new(data);
        let date: DateTime = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::DateTime<FixedOffset> = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1970);
        assert_eq!(ch.month0(), 0);
        assert_eq!(ch.day0(), 0);
        assert_eq!(ch.hour(), 2);
        assert_eq!(ch.minute(), 15);
        assert_eq!(ch.second(), 0);
        assert_eq!(ch.nanosecond(), 42);
        assert_eq!(ch.timezone().local_minus_utc(), 3600);

        let tm: time::OffsetDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1970);
        assert_eq!(tm.month(), time::Month::January);
        assert_eq!(tm.day(), 1);
        assert_eq!(tm.hour(), 2);
        assert_eq!(tm.minute(), 15);
        assert_eq!(tm.second(), 0);
        assert_eq!(tm.nanosecond(), 42);
        assert_eq!(tm.offset().as_hms(), (1, 0, 0));
    }

    #[test]
    fn deserialize_datetime_zoneid() {
        let data = bolt()
            .structure(3, 0x69)
            .int16(4500)
            .tiny_int(42)
            .tiny_string("Europe/Paris")
            .build();
        let mut data = Data::new(data);
        let date: DateTimeZoneIdRef = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::DateTime<chrono_tz::Tz> = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1970);
        assert_eq!(ch.month0(), 0);
        assert_eq!(ch.day0(), 0);
        assert_eq!(ch.hour(), 2);
        assert_eq!(ch.minute(), 15);
        assert_eq!(ch.second(), 0);
        assert_eq!(ch.nanosecond(), 42);
        assert_eq!(ch.timezone().name(), "Europe/Paris");
        assert_eq!(ch.fixed_offset().timezone().local_minus_utc(), 3600);

        let tm: time::OffsetDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1970);
        assert_eq!(tm.month(), time::Month::January);
        assert_eq!(tm.day(), 1);
        assert_eq!(tm.hour(), 2);
        assert_eq!(tm.minute(), 15);
        assert_eq!(tm.second(), 0);
        assert_eq!(tm.nanosecond(), 42);
        assert_eq!(tm.offset().as_hms(), (1, 0, 0));
    }

    #[test]
    fn deserialize_local_datetime() {
        let data = bolt().structure(2, 0x64).int16(4500).tiny_int(42).build();
        let mut data = Data::new(data);
        let date: LocalDateTime = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::NaiveDateTime = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1970);
        assert_eq!(ch.month0(), 0);
        assert_eq!(ch.day0(), 0);
        assert_eq!(ch.hour(), 1);
        assert_eq!(ch.minute(), 15);
        assert_eq!(ch.second(), 0);
        assert_eq!(ch.nanosecond(), 42);

        let tm: time::PrimitiveDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1970);
        assert_eq!(tm.month(), time::Month::January);
        assert_eq!(tm.day(), 1);
        assert_eq!(tm.hour(), 1);
        assert_eq!(tm.minute(), 15);
        assert_eq!(tm.second(), 0);
        assert_eq!(tm.nanosecond(), 42);
    }

    #[test]
    fn deserialize_legacy_datetime() {
        let data = bolt()
            .structure(3, 0x46)
            .int16(8100)
            .tiny_int(42)
            .int16(3600)
            .build();
        let mut data = Data::new(data);
        let date: LegacyDateTime = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::DateTime<FixedOffset> = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1970);
        assert_eq!(ch.month0(), 0);
        assert_eq!(ch.day0(), 0);
        assert_eq!(ch.hour(), 2);
        assert_eq!(ch.minute(), 15);
        assert_eq!(ch.second(), 0);
        assert_eq!(ch.nanosecond(), 42);
        assert_eq!(ch.timezone().local_minus_utc(), 3600);

        let tm: time::OffsetDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1970);
        assert_eq!(tm.month(), time::Month::January);
        assert_eq!(tm.day(), 1);
        assert_eq!(tm.hour(), 2);
        assert_eq!(tm.minute(), 15);
        assert_eq!(tm.second(), 0);
        assert_eq!(tm.nanosecond(), 42);
        assert_eq!(tm.offset().as_hms(), (1, 0, 0));
    }

    #[test]
    fn deserialize_legacy_datetime_zoneid() {
        let data = bolt()
            .structure(3, 0x66)
            .int16(8100)
            .tiny_int(42)
            .tiny_string("Europe/Paris")
            .build();
        let mut data = Data::new(data);
        let date: LegacyDateTimeZoneIdRef = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::DateTime<chrono_tz::Tz> = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1970);
        assert_eq!(ch.month0(), 0);
        assert_eq!(ch.day0(), 0);
        assert_eq!(ch.hour(), 2);
        assert_eq!(ch.minute(), 15);
        assert_eq!(ch.second(), 0);
        assert_eq!(ch.nanosecond(), 42);
        assert_eq!(ch.timezone().name(), "Europe/Paris");
        assert_eq!(ch.fixed_offset().timezone().local_minus_utc(), 3600);

        let tm: time::OffsetDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1970);
        assert_eq!(tm.month(), time::Month::January);
        assert_eq!(tm.day(), 1);
        assert_eq!(tm.hour(), 2);
        assert_eq!(tm.minute(), 15);
        assert_eq!(tm.second(), 0);
        assert_eq!(tm.nanosecond(), 42);
        assert_eq!(tm.offset().as_hms(), (1, 0, 0));
    }

    #[test]
    fn deserialize_positive_datetime() {
        let data = bolt()
            .structure(3, 0x49)
            .int32(946_695_599)
            .int32(420_000)
            .int16(-10800)
            .build();
        let mut data = Data::new(data);
        let date: DateTime = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::DateTime<FixedOffset> = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1999);
        assert_eq!(ch.month0(), 11);
        assert_eq!(ch.day0(), 30);
        assert_eq!(ch.hour(), 23);
        assert_eq!(ch.minute(), 59);
        assert_eq!(ch.second(), 59);
        assert_eq!(ch.nanosecond(), 420_000);
        assert_eq!(ch.timezone().local_minus_utc(), -10800);

        let tm: time::OffsetDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1999);
        assert_eq!(tm.month(), time::Month::December);
        assert_eq!(tm.day(), 31);
        assert_eq!(tm.hour(), 23);
        assert_eq!(tm.minute(), 59);
        assert_eq!(tm.second(), 59);
        assert_eq!(tm.nanosecond(), 420_000);
        assert_eq!(tm.offset().as_hms(), (-3, 0, 0));
    }

    #[test]
    fn deserialize_negative_datetime() {
        let data = bolt()
            .structure(3, 0x49)
            .int64(-16_302_076_758)
            .int32(420_000)
            .int16(10800)
            .build();
        let mut data = Data::new(data);

        let date: DateTime = from_bytes_ref(&mut data).unwrap();

        let ch: chrono::DateTime<FixedOffset> = date.as_chrono_datetime().unwrap();
        assert_eq!(ch.year(), 1453);
        assert_eq!(ch.month0(), 4);
        assert_eq!(ch.day0(), 28);
        assert_eq!(ch.hour(), 16);
        assert_eq!(ch.minute(), 20);
        assert_eq!(ch.second(), 42);
        assert_eq!(ch.nanosecond(), 420_000);
        assert_eq!(ch.timezone().local_minus_utc(), 10800);

        let tm: time::OffsetDateTime = date.as_time_datetime().unwrap();
        assert_eq!(tm.year(), 1453);
        assert_eq!(tm.month(), time::Month::May);
        assert_eq!(tm.day(), 29);
        assert_eq!(tm.hour(), 16);
        assert_eq!(tm.minute(), 20);
        assert_eq!(tm.second(), 42);
        assert_eq!(tm.nanosecond(), 420_000);
        assert_eq!(tm.offset().as_hms(), (3, 0, 0));
    }
}
