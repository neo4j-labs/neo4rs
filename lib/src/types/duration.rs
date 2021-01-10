use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
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
}

impl Into<BoltDuration> for std::time::Duration {
    fn into(self) -> BoltDuration {
        let seconds = self.as_secs();
        let nanos = self.subsec_nanos();
        BoltDuration::new(
            0.into(),
            0.into(),
            (seconds as i64).into(),
            (nanos as i64).into(),
        )
    }
}

impl Into<std::time::Duration> for BoltDuration {
    fn into(self) -> std::time::Duration {
        //TODO: clarify month issue
        let seconds =
            self.seconds.value + (self.days.value * 24 * 3600) + (self.months.value * 2_629_800);
        std::time::Duration::new(seconds as u64, self.nanoseconds.value as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;
    use bytes::*;
    use std::cell::RefCell;
    use std::rc::Rc;

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
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB4, 0x45, 0x0C, 0x02, 0x1E, 0xC9, 0x02, 0xBC,
        ])));

        let duration: BoltDuration = BoltDuration::parse(Version::V4_1, bytes).unwrap();

        assert_eq!(duration.months.value, 12);
        assert_eq!(duration.days.value, 2);
        assert_eq!(duration.seconds.value, 30);
        assert_eq!(duration.nanoseconds.value, 700);
    }
}
