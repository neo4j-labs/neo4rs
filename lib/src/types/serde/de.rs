use std::{collections::HashMap, marker::PhantomData};

use bytes::Bytes;
use serde::{
    de::{Error, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    types::{
        BoltBoolean, BoltBytes, BoltFloat, BoltInteger, BoltList, BoltMap, BoltNull, BoltString,
        BoltType,
    },
    EndNodeId, Id, Indices, Keys, Labels, Nodes, Relationships, StartNodeId, Timezone, Type,
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
