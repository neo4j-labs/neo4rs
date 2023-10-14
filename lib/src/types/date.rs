use crate::{
    errors::Error,
    types::{BoltInteger, Result},
};
use chrono::{Days, NaiveDate, NaiveDateTime};
use neo4rs_macros::BoltStruct;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq, Clone, BoltStruct)]
#[signature(0xB1, 0x44)]
pub struct BoltDate {
    pub(crate) days: BoltInteger,
}

impl From<NaiveDate> for BoltDate {
    fn from(value: NaiveDate) -> Self {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let days = (value - epoch).num_days().into();
        BoltDate { days }
    }
}

impl BoltDate {
    pub(crate) fn try_to_chrono(&self) -> Result<NaiveDate> {
        self.try_into()
    }
}

impl TryFrom<&BoltDate> for NaiveDate {
    type Error = Error;

    fn try_from(value: &BoltDate) -> Result<Self> {
        let epoch = NaiveDateTime::from_timestamp_opt(0, 0).expect("UNIX epoch is always valid");
        let days = Days::new(value.days.value.unsigned_abs());
        if value.days.value >= 0 {
            epoch.checked_add_days(days)
        } else {
            epoch.checked_sub_days(days)
        }
        .map_or(Err(Error::ConversionError), |o| Ok(o.date()))
    }
}

impl TryInto<NaiveDate> for BoltDate {
    type Error = Error;

    fn try_into(self) -> Result<NaiveDate> {
        (&self).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::BoltWireFormat, version::Version};
    use bytes::Bytes;

    #[test]
    fn should_serialize_a_date() {
        let date: BoltDate = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap().into();
        assert_eq!(
            date.into_bytes(Version::V4_1).unwrap(),
            Bytes::from_static(&[0xB1, 0x44, 0xC9, 0x39, 0x12])
        );
    }

    #[test]
    fn should_deserialize_a_date() {
        let mut bytes = Bytes::from_static(&[0xB1, 0x44, 0xC9, 0x39, 0x12]);

        let date: NaiveDate = BoltDate::parse(Version::V4_1, &mut bytes)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(date.to_string(), "2010-01-01");
    }

    #[test]
    fn convert_to_chrono() {
        let date = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();

        let bolt: BoltDate = date.into();
        let actual: NaiveDate = bolt.try_into().unwrap();

        assert_eq!(actual, date);
    }

    #[test]
    fn convert_to_chrono_negative() {
        let date = NaiveDate::from_ymd_opt(1910, 1, 1).unwrap();

        let bolt: BoltDate = date.into();
        let actual: NaiveDate = bolt.try_into().unwrap();

        assert_eq!(actual, date);
    }
}
