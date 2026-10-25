mod begin;
mod bye;
mod commit;
mod discard;
mod failure;
mod hello;
mod ignore;
mod pull;
mod record;
mod reset;
mod rollback;
mod run;
mod success;

use crate::{
    BoltString, BoltType,
    errors::{Error, Result},
    types::{BoltMap, BoltWireFormat},
    version::Version,
};
use begin::Begin;
use bytes::Bytes;
use failure::Failure;
use ignore::Ignore;
use record::Record;
use run::Run;
pub(crate) use success::Success;

#[derive(Debug, PartialEq, Clone)]
pub enum BoltResponse {
    Success(Success),
    Failure(Failure),
    Ignore(Ignore),
    Record(Record),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]
pub enum BoltRequest {
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Hello` instead.")
    )]
    Hello(hello::Hello),
    Run(Run),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Pull` instead.")
    )]
    Pull(pull::Pull),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Discard` instead.")
    )]
    Discard(discard::Discard),
    Begin(Begin),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Commit` instead.")
    )]
    Commit(commit::Commit),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Rollback` instead.")
    )]
    Rollback(rollback::Rollback),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Reset` instead.")
    )]
    Reset(reset::Reset),
}

#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
pub struct HelloBuilder {
    agent: BoltString,
    principal: BoltString,
    credentials: BoltString,
    routing: Option<BoltMap>,
}

#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
impl HelloBuilder {
    pub fn new(principal: impl Into<BoltString>, credentials: impl Into<BoltString>) -> Self {
        Self {
            agent: "neo4rs".into(),
            principal: principal.into(),
            credentials: credentials.into(),
            routing: None,
        }
    }

    pub fn with_routing(self, routing: impl Into<Option<BoltMap>>) -> Self {
        Self {
            routing: routing.into(),
            ..self
        }
    }

    #[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]
    pub fn build(self, version: Version) -> BoltRequest {
        let HelloBuilder {
            agent,
            principal,
            credentials,
            routing,
        } = self;
        BoltRequest::hello(agent, principal, credentials, routing, version)
    }
}

#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]
impl BoltRequest {
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Hello` instead.")
    )]
    pub fn hello(
        agent: BoltString,
        principal: BoltString,
        credentials: BoltString,
        routing: Option<BoltMap>,
        version: Version,
    ) -> BoltRequest {
        let mut data = BoltMap::default();
        data.put("user_agent".into(), BoltType::String(agent));
        data.put("scheme".into(), "basic".into());
        data.put("principal".into(), BoltType::String(principal));
        data.put("credentials".into(), BoltType::String(credentials));
        if version >= Version::V4_1 {
            if let Some(context) = routing {
                data.put("routing".into(), BoltType::Map(context));
            }
        }
        BoltRequest::Hello(hello::Hello::new(data))
    }

    pub fn run(query: &str, params: BoltMap, extra: BoltMap) -> BoltRequest {
        BoltRequest::Run(Run::new(query.into(), params, extra))
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Pull` instead.")
    )]
    pub fn pull(n: usize, qid: i64) -> BoltRequest {
        BoltRequest::Pull(pull::Pull::new(n as i64, qid))
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Discard` instead.")
    )]
    pub fn discard_all() -> BoltRequest {
        BoltRequest::Discard(discard::Discard::default())
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Discard` instead.")
    )]
    pub fn discard_all_for(query_id: i64) -> BoltRequest {
        BoltRequest::Discard(discard::Discard::new(-1, query_id))
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Begin` instead.")
    )]
    pub fn begin(db: Option<&str>, mut extra: BoltMap) -> BoltRequest {
        if let Some(db) = db {
            extra.put("db".into(), db.into());
        }
        let begin = Begin::new(extra);
        BoltRequest::Begin(begin)
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Commit` instead.")
    )]
    pub fn commit() -> BoltRequest {
        BoltRequest::Commit(commit::Commit::new())
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Rollback` instead.")
    )]
    pub fn rollback() -> BoltRequest {
        BoltRequest::Rollback(rollback::Rollback::new())
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.9.0", note = "Use `crate::bolt::Reset` instead.")
    )]
    pub fn reset() -> BoltRequest {
        BoltRequest::Reset(reset::Reset::new())
    }
}

impl BoltRequest {
    #[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]
    pub fn into_bytes(self, version: Version) -> Result<Bytes> {
        let bytes: Bytes = match self {
            BoltRequest::Hello(hello) => hello.into_bytes(version)?,
            BoltRequest::Run(run) => run.into_bytes(version)?,
            BoltRequest::Pull(pull) => pull.into_bytes(version)?,
            BoltRequest::Discard(discard) => discard.into_bytes(version)?,
            BoltRequest::Begin(begin) => begin.into_bytes(version)?,
            BoltRequest::Commit(commit) => commit.into_bytes(version)?,
            BoltRequest::Rollback(rollback) => rollback.into_bytes(version)?,
            BoltRequest::Reset(reset) => reset.into_bytes(version)?,
        };
        Ok(bytes)
    }
}

impl BoltResponse {
    pub fn parse(version: Version, mut response: Bytes) -> Result<BoltResponse> {
        if Success::can_parse(version, &response) {
            let success = Success::parse(version, &mut response)?;
            return Ok(BoltResponse::Success(success));
        }
        if Failure::can_parse(version, &response) {
            let failure = Failure::parse(version, &mut response)?;
            return Ok(BoltResponse::Failure(failure));
        }
        if Record::can_parse(version, &response) {
            let record = Record::parse(version, &mut response)?;
            return Ok(BoltResponse::Record(record));
        }
        if Ignore::can_parse(version, &response) {
            let ignore = Ignore::parse(version, &mut response)?;
            return Ok(BoltResponse::Ignore(ignore));
        }
        Err(Error::UnknownMessage(response.iter().fold(
            "unknown message ".to_owned(),
            |mut output, byte| {
                use std::fmt::Write;
                let _ = write!(output, "{byte:02X}");
                output
            },
        )))
    }

    pub fn into_error(self, msg: &'static str) -> Error {
        match self {
            BoltResponse::Failure(failure) => Error::Neo4j(failure.into_error()),
            BoltResponse::Ignore(ignore) => Error::Neo4j(ignore.into_error()),
            _ => Error::UnexpectedMessage(format!("unexpected response for {msg}: {self:?}")),
        }
    }
}

#[cfg(all(test, not(feature = "unstable-bolt-protocol-impl-v2")))]
mod begin_options_tests {
    use super::*;

    #[test]
    fn begin_serializes_transaction_options_for_supported_bolt_versions() {
        for version in [Version::V4, Version::V4_1] {
            let mut metadata = BoltMap::new();
            metadata.put("operation".into(), "probe".into());
            metadata.put("request_id".into(), "request-1".into());

            let mut options = BoltMap::new();
            options.put("tx_timeout".into(), 15_000_u32.into());
            options.put("tx_metadata".into(), BoltType::Map(metadata.clone()));

            let bytes = BoltRequest::begin(Some("neo4j"), options)
                .into_bytes(version)
                .unwrap();
            let mut fields = bytes.slice(2..);
            let actual = BoltMap::parse(version, &mut fields).unwrap();
            let expected: BoltMap = [
                ("db".into(), "neo4j".into()),
                ("tx_timeout".into(), 15_000_u32.into()),
                ("tx_metadata".into(), BoltType::Map(metadata)),
            ]
            .into_iter()
            .collect();

            assert_eq!(actual, expected);
            assert!(fields.is_empty());
        }
    }

    #[test]
    fn begin_without_transaction_options_keeps_only_database_selection() {
        let bytes = BoltRequest::begin(Some("neo4j"), BoltMap::new())
            .into_bytes(Version::V4_1)
            .unwrap();
        let mut fields = bytes.slice(2..);
        let actual = BoltMap::parse(Version::V4_1, &mut fields).unwrap();

        assert_eq!(
            actual,
            [("db".into(), "neo4j".into())].into_iter().collect()
        );
        assert!(fields.is_empty());
    }
}
