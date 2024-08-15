use std::fmt;

use serde::{
    de::{self, Visitor},
    Deserialize,
};

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
        // TODO: replace with cenum?
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

                        #[allow(non_camel_case_types)]
                        enum Fields { $($keys),+, __Unknown, }

                        struct FieldsVisitor;

                        impl<'de> ::serde::de::Visitor<'de> for FieldsVisitor {
                            type Value = Fields;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                formatter.write_str("a valid field")
                            }

                            fn visit_str<E: ::serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                                Ok(match v {
                                    $(
                                        str!($keys) => Fields::$keys,
                                    )+
                                    _ => Fields::__Unknown,
                                })
                            }
                        }

                        impl<'de> ::serde::Deserialize<'de> for Fields {
                            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                            where
                                D: de::Deserializer<'de>,
                            {
                                deserializer.deserialize_identifier(FieldsVisitor)
                            }
                        }

                        $(
                            let mut $keys = None;
                        )+

                        while let Some(key) = map.next_key::<Fields>()? {
                            match key {
                                $(
                                    Fields::$keys => {
                                        if $keys.is_some() {
                                            return Err(de::Error::duplicate_field(str!($keys)));
                                        }
                                        $keys = Some(map.next_value()?);
                                    }
                                )+
                                _other => {
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
        packstream::{bolt, from_bytes},
        BoltMap, BoltString, BoltType,
    };

    #[test]
    fn parse_stream_summary() {
        let data = bolt()
            .tiny_map(1)
            .tiny_string("has_more")
            .bool(true)
            .build();

        let success = from_bytes::<Streaming>(data).unwrap();

        assert!(matches!(success, Streaming::HasMore,));
    }

    #[test]
    fn parse_full_summary() {
        let data = bolt()
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

        let actual = from_bytes::<Streaming>(data).unwrap();
        let actual = match actual {
            Streaming::Done(actual) => actual,
            _ => panic!("Expected done"),
        };

        assert_eq!(*actual, expected);
    }
}
