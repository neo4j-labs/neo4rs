use std::marker::PhantomData;

use ::serde::{
    de::{self, VariantAccess as _},
    Deserialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail<R> {
    Record(Record<R>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Record<R> {
    pub(crate) record: R,
}

impl<'de, R: Deserialize<'de>> Deserialize<'de> for Detail<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<R>(PhantomData<R>);

        impl<'de, R: Deserialize<'de>> de::Visitor<'de> for Visitor<R> {
            type Value = Detail<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A Bolt detail struct")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                let (tag, data) = data.variant::<u8>()?;
                match tag {
                    0x71 => Ok(Detail::Record(data.newtype_variant::<Record<R>>()?)),
                    _ => Err(de::Error::invalid_type(
                        // TODO: proper error
                        de::Unexpected::Other(&format!("struct with tag {tag:02X}")),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_enum("Detail", &["Record"], Visitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use serde::de::DeserializeOwned;

    use super::*;

    use crate::{bolt::MessageResponse as _, packstream::bolt};

    #[test]
    fn parse_empty_record() {
        let data = bolt().structure(1, 0x71).tiny_list(0).build();

        let detail = Detail::<()>::parse(data).unwrap();
        let Detail::Record(Record { record: () }) = detail;
    }

    #[test]
    fn parse_record_tuple() {
        test_tupleish((42, "1337".to_string(), 84.21));
    }

    #[test]
    fn parse_record_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Foo {
            a: i8,
            b: String,
            c: f32,
        }

        test_tupleish(Foo {
            a: 42,
            b: "1337".to_string(),
            c: 84.21,
        });
    }

    #[test]
    fn parse_record_tuple_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Foo(u16, String, f32);

        test_tupleish(Foo(42, "1337".to_string(), 84.21));
    }

    #[test]
    fn parse_record_enum() {
        #[derive(Debug, PartialEq, Deserialize)]
        #[serde(untagged)]
        enum Val {
            Int(i32),
            Str(String),
            Float(f32),
        }

        test_tupleish(vec![
            Val::Int(42),
            Val::Str("1337".to_string()),
            Val::Float(84.21),
        ]);
    }

    fn test_tupleish<T: DeserializeOwned + PartialEq + Debug>(expected: T) {
        let data = bolt()
            .structure(1, 0x71)
            .tiny_list(3)
            .tiny_int(42)
            .tiny_string("1337")
            .float(84.21)
            .build();

        let detail = Detail::<T>::parse(data).unwrap();
        let Detail::Record(Record { record }) = detail;
        assert_eq!(record, expected);
    }
}
