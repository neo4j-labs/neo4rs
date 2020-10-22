use crate::error::*;
use crate::types::*;
use bytes::*;

use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x3F;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pull {
    extra: BoltMap,
}

impl Default for Pull {
    fn default() -> Self {
        Pull::new(-1 as i64, -1 as i64)
    }
}

impl Pull {
    pub fn new(n: i64, qid: i64) -> Pull {
        let mut extra = BoltMap::new();
        extra.put("n".into(), n.into());
        extra.put("qid".into(), qid.into());
        Pull { extra }
    }
}

impl TryInto<Bytes> for Pull {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let extra: Bytes = self.extra.try_into()?;
        let mut bytes =
            BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<u8>() + extra.len());
        bytes.put_u8(MARKER);
        bytes.put_u8(SIGNATURE);
        bytes.put(extra);
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn should_serialize_pull_message() {
        let pull = Pull::new(42, 1);
        let bytes: Bytes = pull.try_into().unwrap();
        let (marker_signature, extra) = bytes.split_at(2);
        assert_eq!(marker_signature, &[MARKER, SIGNATURE]);
        let extra: BoltMap = Rc::new(RefCell::new(Bytes::copy_from_slice(extra)))
            .try_into()
            .unwrap();

        assert_eq!(extra.get("n").unwrap(), BoltType::from(42));
        assert_eq!(extra.get("qid").unwrap(), BoltType::from(1));
    }

    #[test]
    fn should_serialize_pull_with_default_value() {
        let pull = Pull::default();
        let bytes: Bytes = pull.try_into().unwrap();
        let (marker_signature, extra) = bytes.split_at(2);
        assert_eq!(marker_signature, &[MARKER, SIGNATURE]);
        let extra: BoltMap = Rc::new(RefCell::new(Bytes::copy_from_slice(extra)))
            .try_into()
            .unwrap();

        assert_eq!(extra.get("n").unwrap(), BoltType::from(255));
        assert_eq!(extra.get("qid").unwrap(), BoltType::from(255));
    }
}
