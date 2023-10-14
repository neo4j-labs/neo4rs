use crate::types::BoltInteger;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB4, 0x45)]
pub struct BoltDuration {
    months: BoltInteger,
    days: BoltInteger,
    seconds: BoltInteger,
    nanoseconds: BoltInteger,
}

impl BoltDuration {
    pub fn new(
        months: BoltInteger,
        days: BoltInteger,
        seconds: BoltInteger,
        nanoseconds: BoltInteger,
    ) -> Self {
        BoltDuration {
            months,
            days,
            seconds,
            nanoseconds,
        }
    }

    pub(crate) fn seconds(&self) -> i64 {
        self.seconds
            .value
            .saturating_add(self.days.value.saturating_mul(24 * 3600))
            .saturating_add(self.months.value.saturating_mul(2_629_800))
    }

    pub(crate) fn nanoseconds(&self) -> i64 {
        self.nanoseconds.value
    }
}

impl From<std::time::Duration> for BoltDuration {
    fn from(value: std::time::Duration) -> Self {
        let seconds = value.as_secs();
        let nanos = value.subsec_nanos();
        BoltDuration::new(
            0.into(),
            0.into(),
            (seconds as i64).into(),
            (nanos as i64).into(),
        )
    }
}

impl From<BoltDuration> for std::time::Duration {
    fn from(value: BoltDuration) -> Self {
        //TODO: clarify month issue
        let seconds =
            value.seconds.value + (value.days.value * 24 * 3600) + (value.months.value * 2_629_800);
        std::time::Duration::new(seconds as u64, value.nanoseconds.value as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::BoltWireFormat, version::Version};
    use bytes::Bytes;

    #[test]
    fn should_serialize_a_duration() {
        let duration = BoltDuration::new(12.into(), 2.into(), 30.into(), 700.into());

        let bytes: Bytes = duration.into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[0xB4, 0x45, 0x0C, 0x02, 0x1E, 0xC9, 0x02, 0xBC,])
        );
    }

    #[test]
    fn should_deserialize_a_duration() {
        let mut bytes = Bytes::from_static(&[0xB4, 0x45, 0x0C, 0x02, 0x1E, 0xC9, 0x02, 0xBC]);

        let duration: BoltDuration = BoltDuration::parse(Version::V4_1, &mut bytes).unwrap();

        assert_eq!(duration.months.value, 12);
        assert_eq!(duration.days.value, 2);
        assert_eq!(duration.seconds.value, 30);
        assert_eq!(duration.nanoseconds.value, 700);
    }
}
