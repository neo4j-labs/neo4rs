use std::{fmt, marker::PhantomData};

use serde::{
    de::{self, VariantAccess as _, Visitor},
    Deserialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Summary<R> {
    Success(Success<R>),
    Ignored,
    Failure(Failure),
}

impl<R: std::fmt::Debug> Summary<R> {
    #[allow(unused)]
    pub fn into_error(self, msg: &'static str) -> crate::errors::Error {
        match self {
            Summary::Failure(f) => f.into_error(msg),
            otherwise => crate::Error::UnexpectedMessage(format!(
                "unexpected response for {}: {:?}",
                msg, otherwise
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Success<R> {
    pub(crate) metadata: R,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Failure {
    pub(crate) code: String,
    pub(crate) message: String,
}

impl Failure {
    #[allow(unused)]
    pub fn into_error(self, msg: &'static str) -> crate::errors::Error {
        let Self { code, message } = self;
        crate::errors::Error::Failure { code, message, msg }
    }
}

impl<'de, R: Deserialize<'de>> Deserialize<'de> for Summary<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<R>(PhantomData<R>);

        impl<'de, R: Deserialize<'de>> de::Visitor<'de> for Visitor<R> {
            type Value = Summary<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A Bolt summary struct")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                let (tag, data) = data.variant::<u8>()?;
                match tag {
                    0x70 => Ok(Summary::Success(data.newtype_variant::<Success<R>>()?)),
                    0x7E => Ok(Summary::Ignored),
                    0x7F => Ok(Summary::Failure(data.newtype_variant::<Failure>()?)),
                    _ => Err(de::Error::invalid_type(
                        // TODO: proper error
                        de::Unexpected::Other(&format!("struct with tag {tag:02X}")),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_enum(
            "Summary",
            &["Success", "Ignore", "Failure"],
            Visitor(PhantomData),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Streaming {
    HasMore,
    Done(Box<StreamingSummary>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Read,
    Write,
    ReadWrite,
    SchemaOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamingSummary {
    pub(crate) bookmark: Option<String>,
    pub(crate) t_last: Option<i64>,
    pub(crate) r#type: Option<Type>,
    pub(crate) db: Option<String>,
    pub(crate) stats: Option<crate::BoltMap>,
    pub(crate) plan: Option<crate::BoltMap>,
    pub(crate) profile: Option<crate::BoltMap>,
    pub(crate) notifications: Option<Vec<crate::BoltMap>>,
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visit;

        impl<'de> Visitor<'de> for Visit {
            type Value = Type;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid type string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "r" => Ok(Type::Read),
                    "w" => Ok(Type::Write),
                    "rw" => Ok(Type::ReadWrite),
                    "s" => Ok(Type::SchemaOnly),
                    _ => Err(E::custom(format!("invalid type string: {}", v))),
                }
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(Visit)
    }
}

impl<'de> Deserialize<'de> for Streaming {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "has_more",
            "bookmark",
            "t_last",
            "type",
            "db",
            "stats",
            "plan",
            "profile",
            "notifications",
        ];

        struct Visit;

        impl<'de> Visitor<'de> for Visit {
            type Value = Streaming;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid streaming response")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                macro_rules! str {
                    (r#type) => {
                        "type"
                    };
                    ($key:ident) => {
                        stringify!($key)
                    };
                }

                macro_rules! set {
                    ($($keys:ident),+ $(,)?) => {

                        $(
                            let mut $keys = None;
                        )+

                        while let Some(key) = map.next_key()? {
                            match key {
                                $(
                                    str!($keys) => {
                                        if $keys.is_some() {
                                            return Err(de::Error::duplicate_field(str!($keys)));
                                        }
                                        $keys = Some(map.next_value()?);
                                    }
                                )+
                                _other => {
                                    // return Err(de::Error::unknown_field(other, FIELDS));
                                    map.next_value::<de::IgnoredAny>()?;
                                }
                            }
                        }
                    };
                }

                set!(
                    has_more,
                    bookmark,
                    t_last,
                    r#type,
                    db,
                    stats,
                    plan,
                    profile,
                    notifications,
                );

                let has_more = has_more.unwrap_or(false);

                if has_more {
                    return Ok(Streaming::HasMore);
                }

                let t_last = t_last.ok_or_else(|| de::Error::missing_field("t_last"))?;
                let r#type = r#type.ok_or_else(|| de::Error::missing_field("type"))?;
                let db = db.ok_or_else(|| de::Error::missing_field("db"))?;

                let full = StreamingSummary {
                    bookmark,
                    t_last,
                    r#type,
                    db,
                    stats,
                    plan,
                    profile,
                    notifications,
                };

                Ok(Streaming::Done(Box::new(full)))
            }
        }

        deserializer.deserialize_struct("Response", FIELDS, Visit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::{packstream::value::bolt, MessageResponse as _},
        BoltMap, BoltString, BoltType,
    };

    #[test]
    fn parse_hello_success() {
        let data = bolt().structure(1, 0x70).null().build();

        let success = Summary::<()>::parse(data).unwrap();

        let Summary::Success(Success { metadata: () }) = success else {
            panic!("Expected success");
        };
    }

    #[test]
    fn parse_ignore() {
        let data = bolt().structure(1, 0x7E).build();

        let success = Summary::<()>::parse(data).unwrap();

        assert_eq!(success, Summary::Ignored);
    }

    #[test]
    fn parse_failure() {
        let data = bolt()
            .structure(1, 0x7F)
            .tiny_map(2)
            .tiny_string("code")
            .string8("Neo.ClientError.Security.Unauthorized")
            .tiny_string("message")
            .string8("The client is unauthorized due to authentication failure.")
            .build();

        let failure = Summary::<()>::parse(data).unwrap();

        let Summary::Failure(failure) = failure else {
            panic!("Expected failure");
        };

        assert_eq!(failure.code, "Neo.ClientError.Security.Unauthorized");
        assert_eq!(
            failure.message,
            "The client is unauthorized due to authentication failure."
        );
    }

    #[test]
    fn parse_stream_summary() {
        let data = bolt()
            .structure(1, 0x70)
            .tiny_map(1)
            .tiny_string("has_more")
            .bool(true)
            .build();

        let success = Summary::<Streaming>::parse(data).unwrap();

        assert!(matches!(
            success,
            Summary::Success(Success {
                metadata: Streaming::HasMore,
            })
        ));
    }

    #[test]
    fn parse_full_summary() {
        let data = bolt()
            .structure(1, 0x70)
            .tiny_map(5)
            .tiny_string("bookmark")
            .string16("FB:kcwQ9vYF5wN+TCaprZQJITJbQnaQ")
            .tiny_string("stats")
            .tiny_map(3)
            .tiny_string("labels-added")
            .tiny_int(1)
            .tiny_string("nodes-created")
            .tiny_int(2)
            .tiny_string("properties-set")
            .tiny_int(3)
            .tiny_string("type")
            .tiny_string("rw")
            .tiny_string("t_last")
            .tiny_int(42)
            .tiny_string("db")
            .tiny_string("neo4j")
            .build();

        let expected = StreamingSummary {
            bookmark: Some("FB:kcwQ9vYF5wN+TCaprZQJITJbQnaQ".to_owned()),
            t_last: Some(42),
            r#type: Some(Type::ReadWrite),
            db: Some("neo4j".to_owned()),
            stats: Some(BoltMap::from_iter([
                (BoltString::from("labels-added"), BoltType::from(1)),
                (BoltString::from("nodes-created"), BoltType::from(2)),
                (BoltString::from("properties-set"), BoltType::from(3)),
            ])),
            plan: None,
            profile: None,
            notifications: None,
        };

        let actual = Summary::<Streaming>::parse(data).unwrap();
        let Summary::Success(actual) = actual else {
            panic!("Expected success");
        };
        let Streaming::Done(actual) = actual.metadata else {
            panic!("Expected done");
        };

        assert_eq!(*actual, expected);
    }
}
