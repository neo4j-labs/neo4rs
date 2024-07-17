use std::time::Duration as StdDuration;

use serde::de::{Deserialize, Deserializer};
use thiserror::Error;

use super::de::impl_visitor;

/// A temporal amount.
/// This captures the difference in time between two instants.
/// It only captures the amount of time between two instants, it does not capture a start time and end time.
///
/// The time is represented as a number of months, days, seconds and nanoseconds.
/// The duration can be negative.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Duration {
    months: i64,
    days: i64,
    seconds: i64,
    nanoseconds: i32,
}

impl Duration {
    /// Months in this duration.
    pub fn months(self) -> i64 {
        self.months
    }

    /// Days in this duration.
    pub fn days(self) -> i64 {
        self.days
    }

    /// Seconds in this duration.
    pub fn seconds(self) -> i64 {
        self.seconds
    }

    /// Nanoseconds in this duration.
    pub fn nanoseconds(self) -> i32 {
        self.nanoseconds
    }

    /// Returns the duration as [`std::time::Duration`], or an error if the conversion failed.
    /// The error can be recovered using [`ConversionError::recover`].
    pub fn as_duration(self) -> Result<StdDuration, ConversionError> {
        if self.months == 0 {
            calculate_duration([0, self.days, self.seconds], self.nanoseconds)
        } else {
            Err(ConversionError::EstimationRequired(DurationUnits {
                months: self.months,
                days: self.days,
                seconds: self.seconds,
                nanoseconds: self.nanoseconds,
            }))
        }
    }

    /// Returns the duration as [`std::time::Duration`], while recovering from any error.
    /// See [`ConversionError::recover`] for more details.
    pub fn force_as_duration(self) -> StdDuration {
        self.as_duration().unwrap_or_else(|e| e.deep_recover())
    }

    // pub fn as_chrono_duration(self) -> chrono::Duration {
    //     chrono::Duration::days(self.days)
    // }
    //
    // pub fn as_chrono_date(self) -> Option<chrono::NaiveDate> {
    //     chrono::NaiveDate::from_yo_opt(1970, 1)?.checked_add_signed(self.as_chrono_duration())
    // }
    //
    // pub fn as_time_duration(self) -> time::Duration {
    //     time::Duration::days(self.days)
    // }
    //
    // pub fn as_time_date(self) -> Option<time::Date> {
    //     time::Date::from_ordinal_date(1970, 1)
    //         .ok()?
    //         .checked_add(self.as_time_duration())
    // }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DurationUnits {
    months: i64,
    days: i64,
    seconds: i64,
    nanoseconds: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ConversionError {
    /// The [`Duration`] contained a month part.
    ///
    /// The length of a month varies by month-of-year.
    /// On an attempt to recover, the following estimation is used:
    ///
    /// ```rust
    /// let seconds_per_day = 86400;
    /// let days_per_year = 365.2425;
    /// let seconds_per_year = (days_per_year * f64::from(seconds_per_day)) as u64;
    /// let months_per_year = 12;
    /// let seconds_per_month = seconds_per_year / months_per_year;
    /// assert_eq!(seconds_per_month, 2629746);
    /// ```
    ///
    /// The recovery can still fail with other errors.
    #[error("Duration must not have a month component since its value can only be estimated.")]
    EstimationRequired(DurationUnits),

    /// The [`Duration`] overflows what can be represented by `std::time::Duration`.
    ///
    /// Attempts to recover will try to use the saturated value and ignore any overflow.
    #[error("Duration overflows what can be represented by `std::time::Duration`.")]
    Overflow(i32),

    /// The [`Duration`] is negative, but `std::time::Duration` can only be positive.
    ///
    /// Attempts to recover will try to use the absolute value and ignore any negative value.
    #[error("Duration is negative, but `std::time::Duration` can only be positive.")]
    Negative(StdDuration),
}

impl ConversionError {
    /// Try to recover from the conversion error and return a `std::time::Duration` if possible.
    /// See [`ConversionError`] for more details.
    ///
    /// For `EstimationRequired`, possible values after recovery are `Overflow` and `Negative`.
    /// For `Overflow`, possible values after recovery are `Negative`.
    /// Recovery will always succeed for `Negative`.
    ///
    /// To recover through all possible values, use [`ConversionError::deep_recover`].
    pub fn recover(self) -> Result<StdDuration, Self> {
        match self {
            ConversionError::EstimationRequired(units) => {
                calculate_duration([units.months, units.days, units.seconds], units.nanoseconds)
            }
            ConversionError::Overflow(sign) => {
                if sign >= 0 {
                    Ok(StdDuration::MAX)
                } else {
                    Err(Self::Negative(StdDuration::MAX))
                }
            }
            ConversionError::Negative(duration) => Ok(duration),
        }
    }

    /// Recover from any conversion error and always produce a `std::time::Duration`.
    pub fn deep_recover(self) -> StdDuration {
        let mut err = self;
        loop {
            match err.recover() {
                Ok(duration) => return duration,
                Err(e) => err = e,
            }
        }
    }
}

fn calculate_duration(
    [months, days, secs]: [i64; 3],
    nanos: i32,
) -> Result<StdDuration, ConversionError> {
    const NANOS_PER_SECOND: i32 = 1_000_000_000;
    const SECONDS_PER_DAY: u32 = 86400;
    const SECONDS_PER_MONTH: u32 = {
        const DAYS_PER_YEAR_X10000: u32 = 3_652_425;
        const SECONDS_PER_DAY_DIV100: u32 = SECONDS_PER_DAY / 100;
        const SECONDS_PER_YEAR_X100: u32 = SECONDS_PER_DAY_DIV100 * DAYS_PER_YEAR_X10000;
        const SECONDS_PER_YEAR: u32 = SECONDS_PER_YEAR_X100 / 100;
        SECONDS_PER_YEAR / 12
    };

    // those cannot overflow in the i128 space
    let seconds = i128::from(months) * i128::from(SECONDS_PER_MONTH)
        + i128::from(days) * i128::from(SECONDS_PER_DAY)
        + i128::from(secs)
        + i128::from(nanos / NANOS_PER_SECOND)
        - i128::from(nanos < 0);

    let subsecond = {
        let ns = nanos % NANOS_PER_SECOND;
        ns.unsigned_abs() + (u32::from(ns < 0) * (NANOS_PER_SECOND as u32))
    };

    let sign = seconds.signum() as i32;
    let seconds =
        u64::try_from(seconds.unsigned_abs()).map_err(|_| ConversionError::Overflow(sign))?;

    let duration = StdDuration::new(seconds, subsecond);

    if sign < 0 {
        Err(ConversionError::Negative(duration))
    } else {
        Ok(duration)
    }
}

impl_visitor!(Duration(months, days, seconds, nanoseconds) == 0x45);

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Duration", &[], Self::visitor())
    }
}

#[cfg(test)]
mod tests {
    use crate::packstream::{bolt, from_bytes_ref, BoltBytesBuilder, Data};

    use super::*;

    #[test]
    fn deserialize() {
        let data = bolt_duration()
            .tiny_int(42)
            .int16(1337)
            .int16(420)
            .int32(420_013_370)
            .build();
        let mut data = Data::new(data);
        let duration: Duration = from_bytes_ref(&mut data).unwrap();

        assert_eq!(duration.months(), 42);
        assert_eq!(duration.days(), 1337);
        assert_eq!(duration.seconds(), 420);
        assert_eq!(duration.nanoseconds(), 420_013_370);
    }

    // TODO: many more test cases (testkit also)

    fn bolt_duration() -> BoltBytesBuilder {
        bolt().structure(4, 0x45)
    }
}
