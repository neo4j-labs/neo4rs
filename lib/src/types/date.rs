use crate::errors::Error;
use crate::types::*;
use chrono::{Duration, NaiveDate};
use neo4rs_macros::BoltStruct;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x44)]
pub struct BoltDate {
    days: BoltInteger,
}

impl Into<BoltDate> for NaiveDate {
    fn into(self) -> BoltDate {
        let epoch = NaiveDate::from_ymd(1970, 1, 1);
        let days = (self - epoch).num_days().into();
        BoltDate { days }
    }
}

impl TryInto<NaiveDate> for BoltDate {
    type Error = Error;

    fn try_into(self) -> Result<NaiveDate> {
        let epoch = NaiveDate::from_ymd(1970, 1, 1);
        let days = Duration::days(self.days.value);
        epoch
            .checked_add_signed(days)
            .ok_or(Error::ConverstionError)
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
    fn should_serialize_a_date() {
        let date: BoltDate = NaiveDate::from_ymd(2010, 1, 1).into();
        assert_eq!(
            date.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[0xB1, 0x44, 0xC9, 0x39, 0x12])
        );
    }

    #[test]
    fn should_deserialize_a_date() {
        let bytes = Rc::new(RefCell::new(Bytes::from_static(&[
            0xB1, 0x44, 0xC9, 0x39, 0x12,
        ])));

        let date: NaiveDate = BoltDate::parse(Version::V4_1, bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(date.to_string(), "2010-01-01");
    }
}
