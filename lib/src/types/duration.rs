use crate::types::*;
use neo4rs_macros::BoltStruct;

#[derive(Debug, PartialEq, Clone, Hash, BoltStruct)]
#[signature(0xB4, 0x45)]
pub struct BoltDuration {
    pub months: BoltInteger,
    pub days: BoltInteger,
    pub seconds: BoltInteger,
    pub nanoseconds: BoltInteger,
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

    pub fn as_secs(&self) -> u64 {
        (self.days.value * 86400) as u64 + (self.seconds.value as u64)
    }

    pub fn subsec_nanos(&self) -> u32 {
        self.nanoseconds.value as u32
    }
}

impl Into<BoltDuration> for std::time::Duration {
    fn into(self) -> BoltDuration {
        let days = self.as_secs() / 86400;
        let seconds = self.as_secs() % 86400;
        let nanos = self.subsec_nanos();
        BoltDuration::new(
            0.into(),
            (days as i64).into(),
            (seconds as i64).into(),
            (nanos as i64).into(),
        )
    }
}

impl Into<std::time::Duration> for BoltDuration {
    fn into(self) -> std::time::Duration {
        let seconds = self.as_secs();
        let nanos = self.subsec_nanos();
        std::time::Duration::new(seconds, nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
    use std::cell::RefCell;
    use std::convert::TryInto;
    use std::rc::Rc;

    #[test]
    fn should_serialize_a_duration() {
        let duration = BoltDuration::new(12.into(), 2.into(), 30.into(), 700.into());

        let bytes: Bytes = duration.try_into().unwrap();

        println!("{:#04X?}", bytes.bytes());

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

        let duration: BoltDuration = bytes.try_into().unwrap();

        assert_eq!(duration.months.value, 12);
        assert_eq!(duration.days.value, 2);
        assert_eq!(duration.seconds.value, 30);
        assert_eq!(duration.nanoseconds.value, 700);
    }
}
