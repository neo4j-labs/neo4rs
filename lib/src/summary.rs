use std::{fmt, time::Duration};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Read,
    Write,
    ReadWrite,
    SchemaOnly,
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NotificationSeverity {
    Information,
    Warning,
    Off,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Default, Deserialize)]
#[serde(from = "NotificationWire")]
pub struct Notification {
    pub code: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "SummaryBuilder")]
pub enum Streaming {
    HasMore,
    Done(Box<ResultSummary>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResultSummary {
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

impl ResultSummary {
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

enum Optional<T> {
    Some(T),
    None,
}

impl<T> Optional<T> {
    fn into_option(self) -> Option<T> {
        match self {
            Optional::Some(v) => Some(v),
            Optional::None => None,
        }
    }
}

#[derive(Deserialize)]
struct NotificationWire {
    code: Option<String>,
    title: Option<String>,
    description: Option<String>,
    severity: Option<Optional<NotificationSeverity>>,
    category: Option<Optional<NotificationClassification>>,
    position: Option<InputPosition>,
}

impl From<NotificationWire> for Notification {
    fn from(value: NotificationWire) -> Self {
        Notification {
            code: value.code,
            title: value.title,
            description: value.description,
            severity: value.severity.and_then(Optional::into_option),
            category: value.category.and_then(Optional::into_option),
            position: value.position,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SummaryBuilder {
    has_more: Option<bool>,
    bookmark: Option<String>,
    t_last: Option<i64>,
    r#type: Option<Type>,
    db: Option<String>,
    stats: Option<Counters>,
    plan: Option<Map>,
    profile: Option<Map>,
    notifications: Option<Vec<Notification>>,
}

impl From<SummaryBuilder> for Streaming {
    fn from(value: SummaryBuilder) -> Self {
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
        } = value;

        if has_more.unwrap_or(false) {
            Streaming::HasMore
        } else {
            Streaming::Done(Box::new(ResultSummary {
                bookmark,
                t_first: None,
                t_last: t_last.map(u64::try_from).and_then(Result::ok),
                r#type,
                db,
                stats: stats.unwrap_or_default(),
                plan,
                profile,
                notifications: notifications.unwrap_or_default(),
            }))
        }
    }
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

impl<'de> Deserialize<'de> for NotificationSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct TheVisitor;

        impl<'de> Visitor<'de> for TheVisitor {
            type Value = NotificationSeverity;
            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a valid NotificationSeverity")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                if v.eq_ignore_ascii_case("information") {
                    Ok(NotificationSeverity::Information)
                } else if v.eq_ignore_ascii_case("warning") {
                    Ok(NotificationSeverity::Warning)
                } else if v.eq_ignore_ascii_case("off") {
                    Ok(NotificationSeverity::Off)
                } else {
                    Err(de::Error::unknown_variant(
                        v,
                        &["INFORMATION", "WARNING", "OFF"],
                    ))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(v)
            }
        }

        deserializer.deserialize_any(TheVisitor)
    }
}

impl<'de> Deserialize<'de> for NotificationClassification {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct TheVisitor;

        impl<'de> Visitor<'de> for TheVisitor {
            type Value = NotificationClassification;
            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str("a valid NotificationClassification")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                if v.eq_ignore_ascii_case("hint") {
                    Ok(NotificationClassification::Hint)
                } else if v.eq_ignore_ascii_case("unrecognized") {
                    Ok(NotificationClassification::Unrecognized)
                } else if v.eq_ignore_ascii_case("unsupported") {
                    Ok(NotificationClassification::Unsupported)
                } else if v.eq_ignore_ascii_case("performance") {
                    Ok(NotificationClassification::Performance)
                } else if v.eq_ignore_ascii_case("deprecation") {
                    Ok(NotificationClassification::Deprecation)
                } else if v.eq_ignore_ascii_case("security") {
                    Ok(NotificationClassification::Security)
                } else if v.eq_ignore_ascii_case("topology") {
                    Ok(NotificationClassification::Topology)
                } else if v.eq_ignore_ascii_case("generic") {
                    Ok(NotificationClassification::Generic)
                } else if v.eq_ignore_ascii_case("schema") {
                    Ok(NotificationClassification::Schema)
                } else {
                    Err(de::Error::unknown_variant(
                        v,
                        &[
                            "HINT",
                            "UNRECOGNIZED",
                            "UNSUPPORTED",
                            "PERFORMANCE",
                            "DEPRECATION",
                            "SECURITY",
                            "TOPOLOGY",
                            "GENERIC",
                            "SCHEMA",
                        ],
                    ))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(v)
            }
        }

        deserializer.deserialize_any(TheVisitor)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Optional<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        T::deserialize(deserializer)
            .map(Optional::Some)
            .or_else(|_| Ok(Optional::None))
    }
}

impl<'de> Deserialize<'de> for SummaryBuilder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visit;

        impl<'de> Visitor<'de> for Visit {
            type Value = SummaryBuilder;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid result summary")
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

                Ok(SummaryBuilder {
                    has_more,
                    bookmark,
                    t_last,
                    r#type,
                    db,
                    stats,
                    plan,
                    profile,
                    notifications,
                })
            }
        }

        deserializer.deserialize_struct(
            "Response",
            &[
                "has_more",
                "bookmark",
                "t_last",
                "type",
                "db",
                "stats",
                "plan",
                "profile",
                "notifications",
            ],
            Visit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packstream::{bolt, from_bytes};

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

        let expected = ResultSummary {
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

    #[test]
    fn parse_severity() {
        let data = bolt().tiny_string("WARNING").build();
        let actual = from_bytes::<NotificationSeverity>(data).unwrap();

        assert_eq!(actual, NotificationSeverity::Warning);
    }

    #[test]
    fn parse_classification() {
        let data = bolt().tiny_string("UNRECOGNIZED").build();
        let actual = from_bytes::<NotificationClassification>(data).unwrap();

        assert_eq!(actual, NotificationClassification::Unrecognized);
    }

    #[test]
    fn parse_notification() {
        let data = bolt()
            .tiny_map(6)
            .tiny_string("code")
            .string8("Neo.ClientError.Security.Unauthorized")
            .tiny_string("title")
            .string8("Unauthorized")
            .tiny_string("description")
            .string8("The client is unauthorized due to authentication failure.")
            .tiny_string("severity")
            .tiny_string("WARNING")
            .tiny_string("category")
            .tiny_string("UNRECOGNIZED")
            .tiny_string("position")
            .tiny_map(3)
            .tiny_string("offset")
            .tiny_int(42)
            .tiny_string("line")
            .int16(1337)
            .tiny_string("column")
            .tiny_int(84)
            .build();

        let expected = Notification {
            code: Some("Neo.ClientError.Security.Unauthorized".to_owned()),
            title: Some("Unauthorized".to_owned()),
            description: Some(
                "The client is unauthorized due to authentication failure.".to_owned(),
            ),
            severity: Some(NotificationSeverity::Warning),
            category: Some(NotificationClassification::Unrecognized),
            position: Some(InputPosition {
                offset: 42,
                line: 1337,
                column: 84,
            }),
        };

        let actual = from_bytes::<Notification>(data).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_notification_missing_data() {
        let data = bolt().tiny_map(0).build();

        let expected = Notification {
            code: None,
            title: None,
            description: None,
            severity: None,
            category: None,
            position: None,
        };

        let actual = from_bytes::<Notification>(data).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_notification_invalid_data() {
        let data = bolt()
            .tiny_map(2)
            .tiny_string("severity")
            .tiny_string("FROBNICATE")
            .tiny_string("category")
            .tiny_string("FOOBAR")
            .build();

        let expected = Notification::default();

        let actual = from_bytes::<Notification>(data).unwrap();

        assert_eq!(actual, expected);
    }
}
