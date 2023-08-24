use core::fmt;

use serde::de::{value::MapDeserializer, Error, MapAccess, Visitor};

use crate::{
    types::{BoltDateTime, BoltInteger},
    DeError,
};

crate::cenum!(Fields {
    Seconds,
    NanoSeconds,
    TzOffsetSeconds,
    TzInfo,
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
                Fields::TzInfo => {
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
    pub(crate) fn map_access<'de>(&'de self) -> impl MapAccess<'de, Error = DeError> {
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
