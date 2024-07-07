use std::time::Duration;

use serde::de::{Deserialize, Deserializer};

use super::de::impl_visitor;

/// A date without a time-zone in the ISO-8601 calendar system.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Date {
    days: i64,
}

impl Date {
    /// Days since Unix epoch, e.g. 0 represents 1970-01-01 and 1 represents 1970-01-02.
    pub fn days(self) -> i64 {
        self.days
    }

    /// Returns the duration to the Unix epoch, wrapped in an enum to indicate
    /// if the duration is after (`AfterEpoch`) or before (`BeforeEpoch`) the
    /// Unix epoch.
    /// Returns `None` if the calculation overflows.
    pub fn as_duration(self) -> Option<DateDuration> {
        const SECONDS_PER_DAY: i64 = 86400;
        let seconds = self.days.checked_mul(SECONDS_PER_DAY)?.unsigned_abs();
        let duration = Duration::from_secs(seconds);
        Some(if self.days > 0 {
            DateDuration::AfterEpoch(duration)
        } else {
            DateDuration::BeforeEpoch(duration)
        })
    }

    pub fn as_chrono_duration(self) -> Option<chrono::Duration> {
        chrono::Duration::try_days(self.days)
    }

    pub fn as_chrono_date(self) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::from_yo_opt(1970, 1)?.checked_add_signed(self.as_chrono_duration()?)
    }

    pub fn as_time_duration(self) -> time::Duration {
        time::Duration::days(self.days)
    }

    pub fn as_time_date(self) -> Option<time::Date> {
        time::Date::from_ordinal_date(1970, 1)
            .ok()?
            .checked_add(self.as_time_duration())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateDuration {
    BeforeEpoch(Duration),
    AfterEpoch(Duration),
}

impl DateDuration {
    pub fn is_before_epoch(self) -> bool {
        match self {
            DateDuration::BeforeEpoch(_) => true,
            DateDuration::AfterEpoch(_) => false,
        }
    }

    pub fn is_after_epoch(self) -> bool {
        match self {
            DateDuration::BeforeEpoch(_) => false,
            DateDuration::AfterEpoch(_) => true,
        }
    }

    pub fn abs_duration(self) -> Duration {
        match self {
            DateDuration::BeforeEpoch(d) => d,
            DateDuration::AfterEpoch(d) => d,
        }
    }
}

impl_visitor!(Date(days) == 0x44);

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Date", &[], Self::visitor())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use chrono::Datelike;

    use crate::packstream::{bolt, from_bytes_ref, Data};

    use super::*;

    #[test]
    fn deserialize_date() {
        let data = bolt_date(false);
        let mut data = Data::new(data);
        let date: Date = from_bytes_ref(&mut data).unwrap();

        assert_eq!(date.days(), 1337);
        assert_eq!(
            date.as_duration(),
            Some(DateDuration::AfterEpoch(Duration::from_secs(1337 * 86400)))
        );

        let ch: chrono::Duration = date.as_chrono_duration().unwrap();
        assert_eq!(ch.num_days(), 1337);

        let ch: chrono::NaiveDate = date.as_chrono_date().unwrap();
        assert_eq!(ch.year(), 1973);
        assert_eq!(ch.month0(), 7);
        assert_eq!(ch.day0(), 29);

        let tm: time::Duration = date.as_time_duration();
        assert_eq!(tm.whole_days(), 1337);

        let tm: time::Date = date.as_time_date().unwrap();
        assert_eq!(tm.year(), 1973);
        assert_eq!(tm.month(), time::Month::August);
        assert_eq!(tm.day(), 30);
    }

    #[test]
    fn deserialize_negative_date() {
        let data = bolt_date(true);
        let mut data = Data::new(data);
        let date: Date = from_bytes_ref(&mut data).unwrap();

        assert_eq!(date.days(), -1337);
        assert_eq!(
            date.as_duration(),
            Some(DateDuration::BeforeEpoch(Duration::from_secs(1337 * 86400)))
        );

        let ch: chrono::Duration = date.as_chrono_duration().unwrap();
        assert_eq!(ch.num_days(), -1337);

        let ch: chrono::NaiveDate = date.as_chrono_date().unwrap();
        assert_eq!(ch.year(), 1966);
        assert_eq!(ch.month0(), 4);
        assert_eq!(ch.day0(), 4);

        let tm: time::Duration = date.as_time_duration();
        assert_eq!(tm.whole_days(), -1337);

        let tm: time::Date = date.as_time_date().unwrap();
        assert_eq!(tm.year(), 1966);
        assert_eq!(tm.month(), time::Month::May);
        assert_eq!(tm.day(), 5);
    }

    fn bolt_date(negative: bool) -> Bytes {
        bolt()
            .structure(1, 0x44)
            .int16(if negative { -1337 } else { 1337 })
            .build()
    }
}
