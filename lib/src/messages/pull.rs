use crate::types::*;
use neo4rs_macros::BoltStruct;

pub const MARKER: u8 = 0xB1;
pub const SIGNATURE: u8 = 0x3F;

#[derive(Debug, PartialEq, Clone, BoltStruct)]
#[signature(0xB1, 0x3F)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::*;
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

        assert_eq!(extra.get::<i64>("n").unwrap(), 42.into());
        assert_eq!(extra.get::<i64>("qid").unwrap(), 1.into());
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

        assert_eq!(extra.get::<i64>("n").unwrap(), 255.into());
        assert_eq!(extra.get::<i64>("qid").unwrap(), 255.into());
    }
}
