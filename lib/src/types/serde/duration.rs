use core::fmt;

use serde::de::{value::SeqDeserializer, Error, MapAccess, SeqAccess, Visitor};

use crate::{
    types::{serde::builder::SetOnce, BoltDuration, BoltInteger},
    DeError,
};

crate::cenum!(Fields {
    Months,
    Days,
    Seconds,
    NanoSeconds,
});

impl BoltDuration {
    pub(crate) fn seq_access_bolt(&self) -> impl SeqAccess<'_, Error = DeError> {
        SeqDeserializer::new(
            [
                self.months.value,
                self.days.value,
                self.seconds.value,
                self.nanoseconds.value,
            ]
            .into_iter(),
        )
    }
    pub(crate) fn seq_access_external(&self) -> impl SeqAccess<'_, Error = DeError> {
        SeqDeserializer::new([self.seconds(), self.nanoseconds.value].into_iter())
    }
}

pub struct BoltDurationVisitor;

impl<'de> Visitor<'de> for BoltDurationVisitor {
    type Value = BoltDuration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("BoltDuration struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut builder = DurationBuilder::default();

        while let Some(key) = map.next_key::<Fields>()? {
            match key {
                Fields::Months => builder.months(|| map.next_value())?,
                Fields::Days => builder.days(|| map.next_value())?,
                Fields::Seconds => builder.seconds(|| map.next_value())?,
                Fields::NanoSeconds => builder.nanoseconds(|| map.next_value())?,
            }
        }

        builder.build()
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        const FIELDS: [Fields; 4] = [
            Fields::Months,
            Fields::Days,
            Fields::Seconds,
            Fields::NanoSeconds,
        ];

        let mut require_next = |field| {
            seq.next_element()
                .and_then(|value| value.ok_or_else(|| Error::missing_field(field)))
        };

        let mut builder = DurationBuilder::default();

        for field in FIELDS {
            match field {
                Fields::Months => builder.months(|| require_next("months"))?,
                Fields::Days => builder.days(|| require_next("days"))?,
                Fields::Seconds => builder.seconds(|| require_next("seconds"))?,
                Fields::NanoSeconds => builder.nanoseconds(|| require_next("nanoseconds"))?,
            }
        }

        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(0, &"4"));
        }

        builder.build()
    }
}

#[derive(Default)]
pub(crate) struct DurationBuilder {
    pub(crate) months: SetOnce<BoltInteger>,
    pub(crate) days: SetOnce<BoltInteger>,
    pub(crate) seconds: SetOnce<BoltInteger>,
    pub(crate) nanoseconds: SetOnce<BoltInteger>,
}

impl DurationBuilder {
    fn months<E: Error>(&mut self, f: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        self.months
            .try_insert_with(f)
            .map_or_else(|_| Err(Error::duplicate_field("months")), |_| Ok(()))
    }

    fn days<E: Error>(&mut self, f: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        self.days
            .try_insert_with(f)
            .map_or_else(|_| Err(Error::duplicate_field("days")), |_| Ok(()))
    }

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

    fn build<E: Error>(mut self: DurationBuilder) -> Result<BoltDuration, E> {
        Ok(BoltDuration {
            months: self
                .months
                .take()
                .ok_or_else(|| Error::missing_field("months"))?,
            days: self
                .days
                .take()
                .ok_or_else(|| Error::missing_field("days"))?,
            seconds: self
                .seconds
                .take()
                .ok_or_else(|| Error::missing_field("seconds"))?,
            nanoseconds: self
                .nanoseconds
                .take()
                .ok_or_else(|| Error::missing_field("nanoseconds"))?,
        })
    }
}
