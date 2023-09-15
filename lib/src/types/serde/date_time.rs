use core::fmt;
use std::iter::Peekable;

use serde::de::{
    value::{BorrowedStrDeserializer, MapDeserializer, SeqDeserializer},
    DeserializeSeed, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};

use crate::{
    types::{BoltDateTime, BoltDateTimeZoneId, BoltDuration, BoltInteger},
    DeError,
};

crate::cenum!(Fields {
    Seconds,
    NanoSeconds,
    TzOffsetSeconds,
    TzInfo,
    Datetime,
    NaiveDatetime,
});

pub struct BoltDateTimeVisitor;

impl<'de> Visitor<'de> for BoltDateTimeVisitor {
    type Value = BoltDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("BoltDateTime struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = BoltDateTime {
            seconds: BoltInteger::new(0),
            nanoseconds: BoltInteger::new(0),
            tz_offset_seconds: BoltInteger::new(0),
        };

        while let Some((key, value)) = map.next_entry::<Fields, BoltInteger>()? {
            match key {
                Fields::Seconds => result.seconds = value,
                Fields::NanoSeconds => result.nanoseconds = value,
                Fields::TzOffsetSeconds => result.tz_offset_seconds = value,
                Fields::TzInfo | Fields::Datetime | Fields::NaiveDatetime => {
                    return Err(Error::unknown_field(
                        "tz_info",
                        &["seconds", "nanoseconds", "ts_offset_seconds"],
                    ))
                }
            }
        }

        Ok(result)
    }
}

impl BoltDateTime {
    pub(crate) fn map_access(&self) -> impl MapAccess<'_, Error = DeError> {
        MapDeserializer::new(
            [
                (Fields::Seconds, self.seconds.value),
                (Fields::NanoSeconds, self.nanoseconds.value),
                (Fields::TzOffsetSeconds, self.tz_offset_seconds.value),
            ]
            .into_iter(),
        )
    }
}

impl BoltDuration {
    pub(crate) fn seq_access(&self) -> impl SeqAccess<'_, Error = DeError> {
        SeqDeserializer::new([self.seconds(), self.nanoseconds()].into_iter())
    }
}

impl BoltDateTimeZoneId {
    pub(crate) fn seq_access(&self, as_naive: bool) -> impl SeqAccess<'_, Error = DeError> {
        BoltDateTimeZoneIdAccess::dt_as_string(self, as_naive)
    }

    pub(crate) fn map_access(&self) -> impl MapAccess<'_, Error = DeError> {
        BoltDateTimeZoneIdAccess::fields(self)
    }
}

struct BoltDateTimeZoneIdAccess<'a, const N: usize>(
    &'a BoltDateTimeZoneId,
    Peekable<<[Fields; N] as IntoIterator>::IntoIter>,
);

impl<'a> BoltDateTimeZoneIdAccess<'a, 2> {
    fn dt_as_string(value: &'a BoltDateTimeZoneId, as_naive: bool) -> Self {
        Self(
            value,
            [
                if as_naive {
                    Fields::NaiveDatetime
                } else {
                    Fields::Datetime
                },
                Fields::TzInfo,
            ]
            .into_iter()
            .peekable(),
        )
    }
}

impl<'a> BoltDateTimeZoneIdAccess<'a, 3> {
    fn fields(value: &'a BoltDateTimeZoneId) -> Self {
        Self(
            value,
            [Fields::Seconds, Fields::NanoSeconds, Fields::TzInfo]
                .into_iter()
                .peekable(),
        )
    }
}

impl<'de, const N: usize> SeqAccess<'de> for BoltDateTimeZoneIdAccess<'de, N> {
    type Error = DeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.1.next() {
            Some(Fields::Datetime) => self
                .0
                .try_to_chrono()
                .map(|dt| dt.to_rfc3339().into_deserializer())
                .map_err(|_| Error::custom("Could not convert into chrono::Datetime"))
                .and_then(|dt| seed.deserialize(dt))
                .map(Some),
            Some(Fields::NaiveDatetime) => self
                .0
                .try_to_chrono()
                .map(|dt| format!("{:?}", dt.naive_utc()).into_deserializer())
                .map_err(|_| Error::custom("Could not convert into chrono::Datetime"))
                .and_then(|dt| seed.deserialize(dt))
                .map(Some),
            Some(Fields::Seconds) => seed
                .deserialize(self.0.seconds.value.into_deserializer())
                .map(Some),
            Some(Fields::NanoSeconds) => seed
                .deserialize(self.0.nanoseconds.value.into_deserializer())
                .map(Some),
            Some(Fields::TzInfo) => seed
                .deserialize(BorrowedStrDeserializer::new(self.0.tz_id.value.as_str()))
                .map(Some),
            None => Ok(None),
            _ => Err(Error::custom("invalid field")),
        }
    }
}

impl<'de, const N: usize> MapAccess<'de> for BoltDateTimeZoneIdAccess<'de, N> {
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.1.peek() {
            Some(field) => seed.deserialize(field.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.next_element_seed(seed).map(|opt| opt.unwrap())
    }
}
