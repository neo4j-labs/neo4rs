use serde::de::{value::MapDeserializer, Error, IntoDeserializer, MapAccess, SeqAccess};

use crate::{
    types::{
        serde::date_time::{DateTimeIsh, DateTimeIshBuilder, Fields},
        BoltDate, BoltInteger, BoltLocalTime, BoltTime,
    },
    DeError,
};

impl BoltTime {
    pub(crate) fn seq_access(&self) -> impl SeqAccess<'_, Error = DeError> {
        BoltTimeSeq {
            nanoseconds: Some(self.nanoseconds.clone()),
            offset: Some(self.tz_offset_seconds.clone()),
        }
    }
}

impl BoltTime {
    pub(crate) fn map_access(&self) -> impl MapAccess<'_, Error = DeError> {
        MapDeserializer::new(
            [
                (Fields::NanoSeconds, self.nanoseconds.value),
                (Fields::TzOffsetSeconds, self.tz_offset_seconds.value),
            ]
            .into_iter(),
        )
    }
}

impl BoltLocalTime {
    pub(crate) fn map_access(&self) -> impl MapAccess<'_, Error = DeError> {
        MapDeserializer::new([(Fields::NanoSeconds, self.nanoseconds.value)].into_iter())
    }
}

impl BoltDate {
    pub(crate) fn map_access(&self) -> impl MapAccess<'_, Error = DeError> {
        MapDeserializer::new([(Fields::Days, self.days.value)].into_iter())
    }
}

struct BoltTimeSeq {
    nanoseconds: Option<BoltInteger>,
    offset: Option<BoltInteger>,
}

impl<'de> SeqAccess<'de> for BoltTimeSeq {
    type Error = DeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.nanoseconds.take() {
            Some(nanoseconds) => seed
                .deserialize(
                    BoltLocalTime { nanoseconds }
                        .to_chrono()
                        .to_string()
                        .into_deserializer(),
                )
                .map(Some),
            None => match self.offset.take() {
                Some(offset) => seed.deserialize(offset.value.into_deserializer()).map(Some),
                None => Ok(None),
            },
        }
    }
}

impl DateTimeIsh for BoltTime {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E> {
        Ok(BoltTime {
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

impl DateTimeIsh for BoltLocalTime {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E> {
        Ok(BoltLocalTime {
            nanoseconds: builder
                .nanoseconds
                .take()
                .ok_or_else(|| Error::missing_field("nanoseconds"))?,
        })
    }
}

impl DateTimeIsh for BoltDate {
    fn build<E: Error>(builder: &mut DateTimeIshBuilder) -> Result<Self, E> {
        Ok(BoltDate {
            days: builder
                .days
                .take()
                .ok_or_else(|| Error::missing_field("days"))?,
        })
    }
}
