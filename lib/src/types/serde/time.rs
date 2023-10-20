use serde::{
    de::{value::MapDeserializer, Error, IntoDeserializer, MapAccess, SeqAccess},
    forward_to_deserialize_any, Deserializer,
};

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
            offset: if self.tz_offset_seconds.value == 0 {
                None
            } else {
                Some(self.tz_offset_seconds.clone())
            },
        }
    }

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
    pub(crate) fn seq_access(&self) -> impl SeqAccess<'_, Error = DeError> {
        BoltTimeSeq {
            nanoseconds: Some(self.nanoseconds.clone()),
            offset: None,
        }
    }

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
        struct OptOffsetDeserializer(Option<BoltInteger>);

        impl<'de> Deserializer<'de> for OptOffsetDeserializer {
            type Error = DeError;

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self.0 {
                    Some(offset) => visitor.visit_i64(offset.value),
                    None => visitor.visit_none(),
                }
            }

            fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self.0.as_ref() {
                    Some(_) => visitor.visit_some(self),
                    None => visitor.visit_none(),
                }
            }

            forward_to_deserialize_any! {
                bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
                unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier
                ignored_any
            }
        }

        match self.nanoseconds.take() {
            Some(nanoseconds) => seed
                .deserialize(
                    BoltLocalTime { nanoseconds }
                        .to_chrono()
                        .to_string()
                        .into_deserializer(),
                )
                .map(Some),
            None => seed
                .deserialize(OptOffsetDeserializer(self.offset.take()))
                .map(Some),
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
