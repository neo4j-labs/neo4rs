use std::{collections::HashMap, marker::PhantomData};

use bytes::Bytes;
use chrono::FixedOffset;
use serde::{
    de::{Error, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    types::{
        BoltBoolean, BoltBytes, BoltFloat, BoltInteger, BoltList, BoltMap, BoltNull, BoltString,
        BoltType,
    },
    EndNodeId, Id, Indices, Keys, Labels, Nodes, Offset, Relationships, StartNodeId, Timezone,
    Type,
};

impl<'de> Deserialize<'de> for BoltString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| BoltString { value })
    }
}

impl<'de> Deserialize<'de> for BoltBoolean {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        bool::deserialize(deserializer).map(|value| BoltBoolean { value })
    }
}

impl<'de> Deserialize<'de> for BoltMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        HashMap::<BoltString, BoltType>::deserialize(deserializer).map(|value| BoltMap { value })
    }
}

impl<'de> Deserialize<'de> for BoltNull {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(BoltNull {})
    }
}

impl<'de> Deserialize<'de> for BoltInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(deserializer).map(|value| BoltInteger { value })
    }
}

impl<'de> Deserialize<'de> for BoltFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer).map(|value| BoltFloat { value })
    }
}

impl<'de> Deserialize<'de> for BoltList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<BoltType>::deserialize(deserializer).map(|value| BoltList { value })
    }
}

impl<'de> Deserialize<'de> for BoltBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Bytes::deserialize(deserializer).map(|value| BoltBytes { value })
    }
}

macro_rules! newtype_deser {
    ($($outer:ident$(<$param:ident>)?($inner:ty) => $typ:ty),+ $(,)?) => {
        $(

            impl<'de$(, $param: Deserialize<'de>)?> Deserialize<'de> for $typ {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct TheVisitor$(<$param>(PhantomData<$param>))?;

                    impl<'de$(, $param: Deserialize<'de>)?> Visitor<'de> for TheVisitor$(<$param>)? {
                        type Value = $typ;

                        fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            formatter.write_str(concat!("newtype struct ", stringify!($outer)))
                        }

                        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: Deserializer<'de>,
                        {
                            let value = <$inner>::deserialize(deserializer)?;
                            Ok($outer(value))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: SeqAccess<'de>,
                        {
                            seq.next_element::<$inner>()?
                                .ok_or_else(|| Error::invalid_length(0, &self))
                                .map($outer)
                        }
                    }

                    deserializer.deserialize_newtype_struct(stringify!($outer), TheVisitor$((PhantomData::<$param>))?)
                }
            }

        )+
    };
}

newtype_deser!(
    Id(u64) => Id,
    StartNodeId(u64) => StartNodeId,
    EndNodeId(u64) => EndNodeId,
    Labels<Coll>(Coll) => Labels<Coll>,
    Type<T>(T) => Type<T>,
    Keys<Coll>(Coll) => Keys<Coll>,
    Timezone<T>(T) => Timezone<T>,
    Nodes<T>(Vec<T>) => Nodes<T>,
    Relationships<T>(Vec<T>) => Relationships<T>,
    Indices<T>(Vec<T>) => Indices<T>,
);

impl<'de> Deserialize<'de> for Offset<FixedOffset> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn try_to_offset<T>(v: T) -> Result<FixedOffset, Unexpected<'static>>
        where
            T: Copy + TryInto<i32> + TryInto<i64> + TryInto<u64>,
        {
            match v.try_into().ok().and_then(FixedOffset::east_opt) {
                Some(offset) => Ok(offset),
                None => match v.try_into() {
                    Ok(v) => Err(Unexpected::Signed(v)),
                    Err(_) => match v.try_into() {
                        Ok(v) => Err(Unexpected::Unsigned(v)),
                        Err(_) => Err(Unexpected::Other("big number")),
                    },
                },
            }
        }
        struct OffsetVisitor;

        impl<'de> Visitor<'de> for OffsetVisitor {
            type Value = FixedOffset;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an offset value as i32 within [-86_400, 86_400]")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                try_to_offset(v).map_err(|u| Error::invalid_value(u, &self))
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: Error,
            {
                try_to_offset(v).map_err(|u| Error::invalid_value(u, &self))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                try_to_offset(v).map_err(|u| Error::invalid_value(u, &self))
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: Error,
            {
                try_to_offset(v).map_err(|u| Error::invalid_value(u, &self))
            }
        }

        deserializer.deserialize_i32(OffsetVisitor).map(Offset)
    }
}
