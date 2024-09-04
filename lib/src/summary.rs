use std::{fmt, marker::PhantomData, time::Duration};

use serde::{
    de::{self, Visitor},
    Deserialize,
};

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
type MapKey = String;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
type MapValue = crate::bolt::Bolt;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
type Map = std::collections::HashMap<MapKey, MapValue>;

#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
type MapKey = crate::BoltString;
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
type MapValue = create::BoltType;
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
type Map = crate::BoltMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Read,
    Write,
    ReadWrite,
    SchemaOnly,
    Unknown,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NotificationSeverity {
    Information,
    Warning,
    Off,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NotificationClassification {
    Hint,
    Unrecognized,
    Unsupported,
    Performance,
    Deprecation,
    Security,
    Topology,
    Generic,
    Schema,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct InputPosition {
    pub offset: i64,
    pub line: i64,
    pub column: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Notification {
    pub code: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<NotificationSeverity>,
    pub category: Option<NotificationClassification>,
    pub position: Option<InputPosition>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Counters {
    pub nodes_created: u64,
    pub nodes_deleted: u64,
    pub relationships_created: u64,
    pub relationships_deleted: u64,
    pub properties_set: u64,
    pub labels_added: u64,
    pub labels_removed: u64,
    pub indexes_added: u64,
    pub indexes_removed: u64,
    pub constraints_added: u64,
    pub constraints_removed: u64,
    pub system_updates: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Streaming {
    HasMore,
    Done(Box<StreamingSummary>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamingSummary {
    pub bookmark: Option<String>,
    t_first: Option<u64>,
    t_last: Option<u64>,
    pub r#type: Option<Type>,
    pub db: Option<String>,
    pub stats: Counters,
    pub(crate) plan: Option<Map>,
    pub(crate) profile: Option<Map>,
    pub notifications: Vec<Notification>,
}

impl StreamingSummary {
    pub fn available_after(&self) -> Option<Duration> {
        self.t_first.map(Duration::from_millis)
    }

    pub fn consumed_after(&self) -> Option<Duration> {
        self.t_last.map(Duration::from_millis)
    }

    pub fn query_type(&self) -> Type {
        self.r#type.unwrap_or(Type::Unknown)
    }

    pub fn db(&self) -> Option<&str> {
        self.db.as_deref()
    }

    pub fn stats(&self) -> &Counters {
        &self.stats
    }

    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }

    pub fn nodes_created(&self) -> u64 {
        self.stats.nodes_created
    }
    pub fn nodes_deleted(&self) -> u64 {
        self.stats.nodes_deleted
    }
    pub fn relationships_created(&self) -> u64 {
        self.stats.relationships_created
    }
    pub fn relationships_deleted(&self) -> u64 {
        self.stats.relationships_deleted
    }
    pub fn properties_set(&self) -> u64 {
        self.stats.properties_set
    }
    pub fn labels_added(&self) -> u64 {
        self.stats.labels_added
    }
    pub fn labels_removed(&self) -> u64 {
        self.stats.labels_removed
    }
    pub fn indexes_added(&self) -> u64 {
        self.stats.indexes_added
    }
    pub fn indexes_removed(&self) -> u64 {
        self.stats.indexes_removed
    }
    pub fn constraints_added(&self) -> u64 {
        self.stats.constraints_added
    }
    pub fn constraints_removed(&self) -> u64 {
        self.stats.constraints_removed
    }
    pub fn system_updates(&self) -> u64 {
        self.stats.system_updates
    }

    pub(crate) fn set_t_first(&mut self, t_first: i64) {
        self.t_first = u64::try_from(t_first).ok();
    }
}

impl<'de> Deserialize<'de> for Streaming {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let SummaryBuilder {
            has_more,
            bookmark,
            t_last,
            r#type,
            db,
            stats,
            plan,
            profile,
            notifications,
        } = SummaryBuilder::<String, Map, Counters, Notification>::deserialize(deserializer)?;

        if has_more {
            return Ok(Streaming::HasMore);
        }

        let full = StreamingSummary {
            bookmark,
            t_first: None,
            t_last: t_last.map(u64::try_from).and_then(Result::ok),
            r#type,
            db,
            stats: stats.unwrap_or_default(),
            plan,
            profile,
            notifications: notifications.unwrap_or_default(),
        };

        Ok(Streaming::Done(Box::new(full)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamingRef<'de> {
    HasMore,
    Done(Box<StreamingSummaryRef<'de>>),
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingSummaryRef<'de> {
    pub(crate) bookmark: Option<&'de str>,
    pub(crate) t_last: Option<i64>,
    pub(crate) r#type: Option<Type>,
    pub(crate) db: Option<&'de str>,
    pub(crate) stats: Option<std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>>,
    pub(crate) plan: Option<std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>>,
    pub(crate) profile: Option<std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>>,
    pub(crate) notifications:
        Option<Vec<std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>>>,
}

impl<'de> Deserialize<'de> for StreamingRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let SummaryBuilder {
            has_more,
            bookmark,
            t_last,
            r#type,
            db,
            stats,
            plan,
            profile,
            notifications,
        } = SummaryBuilder::<
            &'de str,
            std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>,
            std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>,
            std::collections::HashMap<&'de str, crate::bolt::BoltRef<'de>>,
        >::deserialize(deserializer)?;

        if has_more {
            return Ok(StreamingRef::HasMore);
        }

        let full = StreamingSummaryRef {
            bookmark,
            t_last,
            r#type,
            db,
            stats,
            plan,
            profile,
            notifications,
        };

        Ok(StreamingRef::Done(Box::new(full)))
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[derive(Debug, Clone, PartialEq)]
struct SummaryBuilder<Str, Map, Stats, Note> {
    has_more: bool,
    bookmark: Option<Str>,
    t_last: Option<i64>,
    r#type: Option<Type>,
    db: Option<Str>,
    stats: Option<Stats>,
    plan: Option<Map>,
    profile: Option<Map>,
    notifications: Option<Vec<Note>>,
}

impl<
        'de,
        Key: Deserialize<'de>,
        Map: Deserialize<'de>,
        Stats: Deserialize<'de>,
        Note: Deserialize<'de>,
    > Deserialize<'de> for SummaryBuilder<Key, Map, Stats, Note>
{
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

        struct Visit<Str, Map, Stats, Note>(PhantomData<(Str, Map, Stats, Note)>);

        impl<
                'de,
                Str: Deserialize<'de>,
                Map: Deserialize<'de>,
                Stats: Deserialize<'de>,
                Note: Deserialize<'de>,
            > Visitor<'de> for Visit<Str, Map, Stats, Note>
        {
            type Value = SummaryBuilder<Str, Map, Stats, Note>;

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

                let full = SummaryBuilder {
                    has_more,
                    bookmark,
                    t_last,
                    r#type,
                    db,
                    stats,
                    plan,
                    profile,
                    notifications,
                };

                Ok(full)
            }
        }

        deserializer.deserialize_struct("Response", FIELDS, Visit(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packstream::{bolt, from_bytes, from_bytes_ref, Data};

    #[test]
    fn parse_stream_summary() {
        let data = bolt()
            .tiny_map(1)
            .tiny_string("has_more")
            .bool(true)
            .build();

        let success = from_bytes::<Streaming>(data).unwrap();

        assert!(matches!(success, Streaming::HasMore));
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
            t_first: None,
            t_last: Some(42),
            r#type: Some(Type::ReadWrite),
            db: Some("neo4j".to_owned()),
            stats: Counters {
                labels_added: 1,
                nodes_created: 2,
                properties_set: 3,
                ..Default::default()
            },
            plan: None,
            profile: None,
            notifications: Vec::new(),
        };

        let actual = from_bytes::<Streaming>(data).unwrap();
        let actual = match actual {
            Streaming::Done(actual) => actual,
            _ => panic!("Expected done"),
        };

        assert_eq!(*actual, expected);
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    #[test]
    fn parse_stream_summary_ref() {
        let data = bolt()
            .tiny_map(1)
            .tiny_string("has_more")
            .bool(true)
            .build();

        let mut data = Data::new(data);
        let success = from_bytes_ref::<StreamingRef>(&mut data).unwrap();

        assert!(matches!(success, StreamingRef::HasMore));
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    #[test]
    fn parse_full_summary_ref() {
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

        let expected = StreamingSummaryRef {
            bookmark: Some("FB:kcwQ9vYF5wN+TCaprZQJITJbQnaQ"),
            t_last: Some(42),
            r#type: Some(Type::ReadWrite),
            db: Some("neo4j"),
            stats: Some(std::collections::HashMap::from_iter([
                ("labels-added", crate::bolt::BoltRef::from(1)),
                ("nodes-created", crate::bolt::BoltRef::from(2)),
                ("properties-set", crate::bolt::BoltRef::from(3)),
            ])),
            plan: None,
            profile: None,
            notifications: None,
        };

        let mut data = Data::new(data);
        let actual = from_bytes_ref::<StreamingRef>(&mut data).unwrap();
        let actual = match actual {
            StreamingRef::Done(actual) => actual,
            _ => panic!("Expected done"),
        };

        assert_eq!(*actual, expected);
    }
}
