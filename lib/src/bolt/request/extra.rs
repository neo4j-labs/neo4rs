use std::num::TryFromIntError;

use crate::errors::{Error, Result};
use serde::{ser::SerializeStruct as _, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extra {
    n: i64,
    qid: Option<i64>,
}

pub trait WrapExtra: Sized {
    fn all() -> Self {
        Self::new(None, None)
    }

    fn some(n: i64) -> Self {
        Self::new(Some(n), None)
    }

    fn many<T: TryInto<i64, Error = TryFromIntError>>(n: T) -> Result<Self> {
        let n = n.try_into().map_err(|e| Error::IntegerOverflow("n", e))?;
        Ok(Self::new(Some(n), None))
    }

    fn for_query(mut self, query_id: i64) -> Self {
        self.extra_mut().qid = Some(query_id);
        self
    }

    fn for_last_query(self) -> Self {
        self.for_query(-1)
    }

    fn new(how_many: Option<i64>, qid: Option<i64>) -> Self {
        let n = how_many.filter(|i| *i >= 0).unwrap_or(-1);
        Self::create(Extra { n, qid })
    }

    fn create(extra: Extra) -> Self;

    fn extra_mut(&mut self) -> &mut Extra;
}

impl WrapExtra for Extra {
    fn create(extra: Extra) -> Self {
        extra
    }

    fn extra_mut(&mut self) -> &mut Extra {
        self
    }
}

impl Serialize for Extra {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("Extra", 1 + usize::from(self.qid.is_some()))?;
        ser.serialize_field("n", &self.n)?;
        if let Some(qid) = self.qid {
            ser.serialize_field("qid", &qid)?;
        }
        ser.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bolt::Message as _, packstream::bolt};

    #[test]
    fn serialize() {
        let hello = Extra::some(42).for_query(1);
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt()
            .tiny_map(2)
            .tiny_string("n")
            .tiny_int(42)
            .tiny_string("qid")
            .tiny_int(1)
            .build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn serialize_default_values() {
        let hello = Extra::all();
        let bytes = hello.to_bytes().unwrap();

        let expected = bolt().tiny_map(1).tiny_string("n").tiny_int(-1).build();

        assert_eq!(bytes, expected);
    }
}
