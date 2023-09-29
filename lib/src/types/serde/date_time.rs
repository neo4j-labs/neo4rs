use core::fmt;
use std::{iter::Peekable, marker::PhantomData};

use serde::de::{
    value::{BorrowedStrDeserializer, MapDeserializer, SeqDeserializer},
    DeserializeSeed, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};

use crate::types::{serde::builder::SetOnce, BoltLocalDateTime, BoltString};
use crate::{
    types::{BoltDateTime, BoltDateTimeZoneId, BoltDuration, BoltInteger},
    DeError,
};

crate::cenum!(Fields {
    Seconds,
    NanoSeconds,
    TzOffsetSeconds,
    TzInfo,
    Days,
    Datetime,
    NaiveDatetime,
});

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

impl BoltLocalDateTime {
    pub(crate) fn map_access(&self) -> impl MapAccess<'_, Error = DeError> {
        MapDeserializer::new(
            [
                (Fields::Seconds, self.seconds.value),
                (Fields::NanoSeconds, self.nanoseconds.value),
            ]
            .into_iter(),
        )
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

impl BoltDuration {
    pub(crate) fn seq_access(&self) -> impl SeqAccess<'_, Error = DeError> {
        SeqDeserializer::new([self.seconds(), self.nanoseconds()].into_iter())
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
                .map(|dt| format!("{:?}", dt.naive_local()).into_deserializer())
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

pub struct BoltDateTimeVisitor<A>(PhantomData<A>);

impl<A> BoltDateTimeVisitor<A> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T: DateTimeIsh> Visitor<'de> for BoltDateTimeVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::any::type_name::<T>())?;
        formatter.write_str(" struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut builder = DateTimeIshBuilder::default();

        while let Some(key) = map.next_key::<Fields>()? {
            match key {
                Fields::Seconds => builder.seconds(|| map.next_value())?,
                Fields::NanoSeconds => builder.nanoseconds(|| map.next_value())?,
                Fields::TzOffsetSeconds => builder.tz_offset_seconds(|| map.next_value())?,
                Fields::TzInfo => builder.tz_id(|| map.next_value())?,
                Fields::Days => builder.days(|| map.next_value())?,
                Fields::Datetime | Fields::NaiveDatetime => {
                    return Err(Error::unknown_field(
                        "datetime",
                        &[
                            "seconds",
                            "nanoseconds",
                            "ts_offset_seconds",
                            "tz_id",
                            "days",
                        ],
                    ))
                }
            }
        }

        let res = T::build(&mut builder)?;

        if builder.seconds.is_set() {
            return Err(Error::unknown_field(
                "seconds",
                &["nanoseconds", "ts_offset_seconds", "tz_id", "days"],
            ));
        }
        if builder.nanoseconds.is_set() {
            return Err(Error::unknown_field(
                "nanoseconds",
                &["seconds", "ts_offset_seconds", "tz_id", "days"],
            ));
        }
        if builder.tz_offset_seconds.is_set() {
            return Err(Error::unknown_field(
                "tz_offset_seconds",
                &["seconds", "nanoseconds", "tz_id", "days"],
            ));
        }
        if builder.tz_id.is_set() {
            return Err(Error::unknown_field(
                "tz_id",
                &["seconds", "nanoseconds", "tz_offset_seconds", "days"],
            ));
        }
        if builder.days.is_set() {
            return Err(Error::unknown_field(
                "days",
                &["seconds", "nanoseconds", "tz_offset_seconds", "tz_id"],
            ));
        }

        Ok(res)
    }
}

#[derive(Default)]
pub(crate) struct DateTimeIshBuilder {
    pub(crate) seconds: SetOnce<BoltInteger>,
    pub(crate) nanoseconds: SetOnce<BoltInteger>,
    pub(crate) tz_offset_seconds: SetOnce<BoltInteger>,
    pub(crate) tz_id: SetOnce<BoltString>,
    pub(crate) days: SetOnce<BoltInteger>,
}

impl DateTimeIshBuilder {
    fn seconds<E: Error>(&mut self, f: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        self.seconds
            .try_insert_with(f)
            .map_or_else(|_| Err(Error::duplicate_field("seconds")), |_| Ok(()))
    }

    fn nanoseconds<E: Error>(
        &mut self,
        f: impl FnOnce() -> Result<BoltInteger, E>,
    ) -> Result<(), E> {
        self.nanoseconds
            .try_insert_with(f)
            .map_or_else(|_| Err(Error::duplicate_field("nanoseconds")), |_| Ok(()))
    }

    fn tz_offset_seconds<E: Error>(
        &mut self,
        f: impl FnOnce() -> Result<BoltInteger, E>,
    ) -> Result<(), E> {
        self.tz_offset_seconds.try_insert_with(f).map_or_else(
            |_| Err(Error::duplicate_field("tz_offset_seconds")),
            |_| Ok(()),
        )
    }

    fn tz_id<E: Error>(&mut self, f: impl FnOnce() -> Result<BoltString, E>) -> Result<(), E> {
        self.tz_id
            .try_insert_with(f)
            .map_or_else(|_| Err(Error::duplicate_field("tz_id")), |_| Ok(()))
    }

    fn days<E: Error>(&mut self, f: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        self.days
            .try_insert_with(f)
            .map_or_else(|_| Err(Error::duplicate_field("days")), |_| Ok(()))
    }
}

pub(crate) trait DateTimeIsh: Sized {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E>;
}

impl DateTimeIsh for BoltDateTime {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E> {
        Ok(BoltDateTime {
            seconds: builder
                .seconds
                .take()
                .ok_or_else(|| Error::missing_field("seconds"))?,
            nanoseconds: builder
                .nanoseconds
                .take()
                .ok_or_else(|| Error::missing_field("nanoseconds"))?,
            tz_offset_seconds: builder
                .tz_offset_seconds
                .take()
                .ok_or_else(|| Error::missing_field("tz_offset_seconds"))?,
        })
    }
}

impl DateTimeIsh for BoltLocalDateTime {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E> {
        Ok(BoltLocalDateTime {
            seconds: builder
                .seconds
                .take()
                .ok_or_else(|| Error::missing_field("seconds"))?,
            nanoseconds: builder
                .nanoseconds
                .take()
                .ok_or_else(|| Error::missing_field("nanoseconds"))?,
        })
    }
}

impl DateTimeIsh for BoltDateTimeZoneId {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E> {
        Ok(BoltDateTimeZoneId {
            seconds: builder
                .seconds
                .take()
                .ok_or_else(|| Error::missing_field("seconds"))?,
            nanoseconds: builder
                .nanoseconds
                .take()
                .ok_or_else(|| Error::missing_field("nanoseconds"))?,
            tz_id: builder
                .tz_id
                .take()
                .ok_or_else(|| Error::missing_field("tz_id"))?,
        })
    }
}
