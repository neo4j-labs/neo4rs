use crate::errors::*;
use crate::types::*;
use bytes::*;
use std::convert::TryInto;
use std::mem;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x2F;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Discard {
    extra: BoltMap,
}

impl Default for Discard {
    fn default() -> Self {
        Discard::new(-1 as i64, -1 as i64)
    }
}

impl Discard {
    pub fn new(n: i64, qid: i64) -> Discard {
        let mut extra = BoltMap::new();
        extra.put("n".into(), n.into());
        extra.put("qid".into(), qid.into());
        Discard { extra }
    }
}

impl TryInto<Bytes> for Discard {
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
    fn should_serialize_discard_message() {
        let discard = Discard::new(42, 1);
        let bytes: Bytes = discard.try_into().unwrap();
        let (marker_signature, extra) = bytes.split_at(2);
        assert_eq!(marker_signature, &[MARKER, SIGNATURE]);
        let extra: BoltMap = Rc::new(RefCell::new(Bytes::copy_from_slice(extra)))
            .try_into()
            .unwrap();

        assert_eq!(extra.get("n").unwrap(), 42.into());
        assert_eq!(extra.get("qid").unwrap(), 1.into());
    }

    #[test]
    fn should_serialize_discard_with_default_value() {
        let discard = Discard::default();
        let bytes: Bytes = discard.try_into().unwrap();
        let (marker_signature, extra) = bytes.split_at(2);
        assert_eq!(marker_signature, &[MARKER, SIGNATURE]);
        let extra: BoltMap = Rc::new(RefCell::new(Bytes::copy_from_slice(extra)))
            .try_into()
            .unwrap();

        assert_eq!(extra.get("n").unwrap(), 255.into());
        assert_eq!(extra.get("qid").unwrap(), 255.into());
    }
}
