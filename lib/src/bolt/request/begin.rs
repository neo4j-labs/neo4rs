use crate::bolt::{ExpectedResponse, Hello, Summary};
use crate::routing::Route;
use crate::{Database, Version};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Begin<'a> {
    metadata: BeginMeta<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeginExtra<'a> {
    V4(Option<&'a str>),
    V4_4(Extra<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Extra<'a> {
    pub(crate) db: Option<&'a str>,
    pub(crate) imp_user: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxMetadata(Vec<(String, String)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeginMeta<'a> {
    pub(crate) bookmarks: Vec<String>,
    pub(crate) tx_timeout: Option<u32>,
    pub(crate) tx_metadata: Option<TxMetadata>,
    pub(crate) mode: &'a str,
    pub(crate) extra: BeginExtra<'a>,
    // To be added when implementing protocol version 5.2
    // pub(crate) notifications_minimum_severity: &'a str,
    // pub(crate) notifications_disabled_categories: Vec<String>
}

pub struct BeginBuilder<'a> {
    bookmarks: Vec<String>,
    tx_timeout: Option<u32>,
    tx_metadata: Option<TxMetadata>,
    mode: &'a str,
    db: Option<&'a str>,
    imp_user: Option<&'a str>,
}

impl<'a> BeginBuilder<'a> {
    pub fn new(db: Option<&'a str>) -> Self {
        Self {
            bookmarks: Vec::new(),
            tx_timeout: None,
            tx_metadata: None,
            mode: "w", // default is write mode
            db,
            imp_user: None,
        }
    }

    pub fn with_bookmarks(mut self, bookmarks: Vec<impl Display>) -> Self {
        self.bookmarks = bookmarks
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<String>>();
        self
    }

    pub fn with_tx_timeout(mut self, tx_timeout: u32) -> Self {
        self.tx_timeout = Some(tx_timeout);
        self
    }

    pub fn with_tx_metadata(mut self, tx_metadata: Vec<(String, String)>) -> Self {
        self.tx_metadata = Some(TxMetadata(tx_metadata));
        self
    }

    pub fn with_mode(mut self, mode: &'a str) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_imp_user(mut self, imp_user: &'a str) -> Self {
        self.imp_user = Some(imp_user);
        self
    }

    pub fn build(self, version: Version) -> Begin<'a> {
        match version.cmp(&Version::V4_4) {
            std::cmp::Ordering::Less => Begin {
                metadata: BeginMeta {
                    bookmarks: self.bookmarks,
                    tx_timeout: self.tx_timeout,
                    tx_metadata: self.tx_metadata,
                    mode: self.mode,
                    extra: BeginExtra::V4(self.db),
                },
            },
            _ => Begin {
                metadata: BeginMeta {
                    bookmarks: self.bookmarks,
                    tx_timeout: self.tx_timeout,
                    tx_metadata: self.tx_metadata,
                    mode: self.mode,
                    extra: BeginExtra::V4_4(Extra {
                        db: self.db,
                        imp_user: self.imp_user,
                    }),
                },
            },
        }
    }
}

impl<'a> Begin<'a> {
    pub fn builder(db: Option<&'a str>) -> BeginBuilder<'a> {
        BeginBuilder::new(db)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Response {
    pub(crate) db: Option<Database>,
}

impl ExpectedResponse for Begin<'_> {
    type Response = Summary<Response>;
}

impl Serialize for Begin<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_variant("Request", 0x11, "BEGIN", &self.metadata)
    }
}

impl Serialize for TxMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl Serialize for BeginMeta<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut fields_count = 2; // minimum number of fields for the map
        if self.tx_metadata.is_some() {
            fields_count += 1;
        }
        if self.tx_timeout.is_some() {
            fields_count += 1;
        }

        match &self.extra {
            BeginExtra::V4(e) => {
                if e.is_some() {
                    fields_count += 1;
                }
            }
            BeginExtra::V4_4(e) => {
                if e.db.is_some() {
                    fields_count += 1;
                }
                if e.imp_user.is_some() {
                    fields_count += 1;
                }
            }
        }

        let mut map = serializer.serialize_map(Some(fields_count))?;
        map.serialize_entry("bookmarks", &self.bookmarks)?;
        map.serialize_entry("mode", &self.mode)?;
        if let Some(tx_timeout) = self.tx_timeout {
            map.serialize_entry("tx_timeout", &tx_timeout)?;
        }
        if let Some(tx_metadata) = self.tx_metadata.as_ref() {
            map.serialize_entry("tx_metadata", tx_metadata)?;
        }
        match &self.extra {
            BeginExtra::V4(db) => {
                if let Some(db) = db {
                    map.serialize_entry("db", db)?;
                }
            }
            BeginExtra::V4_4(extra) => {
                if let Some(db) = extra.db.as_ref() {
                    map.serialize_entry("db", db)?;
                }
                if let Some(imp_user) = extra.imp_user.as_ref() {
                    map.serialize_entry("imp_user", imp_user)?;
                }
            }
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::Begin;
    use crate::bolt::Message;
    use crate::packstream::bolt;
    use crate::{Database, Version};
    use std::collections::HashMap;

    #[test]
    fn serialize() {
        let begin = Begin::builder(None)
            .with_bookmarks(vec!["example-bookmark:1", "example-bookmark:2"])
            .with_tx_metadata(
                [
                    ("user".to_string(), "alice".to_string()),
                    ("action".to_string(), "data_import".to_string()),
                ]
                .to_vec(),
            )
            .build(Version::V4);
        let bytes = begin.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x11)
            .tiny_map(3)
            .tiny_string("bookmarks")
            .tiny_list(2)
            .string8("example-bookmark:1")
            .string8("example-bookmark:2")
            .tiny_string("mode")
            .tiny_string("w")
            .tiny_string("tx_metadata")
            .tiny_map(2)
            .tiny_string("user")
            .tiny_string("alice")
            .tiny_string("action")
            .tiny_string("data_import")
            .build();

        assert_eq!(bytes, expected);

        let db = Some(Database::from("neo4j"));
        let begin = Begin::builder(db.as_deref())
            .with_bookmarks(vec!["example-bookmark:1", "example-bookmark:2"])
            .with_imp_user("my_user")
            .build(Version::V4_4);
        let bytes = begin.to_bytes().unwrap();

        let expected = bolt()
            .structure(1, 0x11)
            .tiny_map(4)
            .tiny_string("bookmarks")
            .tiny_list(2)
            .string8("example-bookmark:1")
            .string8("example-bookmark:2")
            .tiny_string("mode")
            .tiny_string("w")
            .tiny_string("db")
            .tiny_string("neo4j")
            .tiny_string("imp_user")
            .tiny_string("my_user")
            .build();

        assert_eq!(bytes, expected);
    }
}
