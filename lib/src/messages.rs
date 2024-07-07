mod begin;
mod bye;
mod commit;
mod discard;
mod failure;
mod hello;
mod pull;
mod record;
mod reset;
mod rollback;
mod run;
mod success;

use crate::{
    errors::{Error, Result},
    types::{BoltMap, BoltWireFormat},
    version::Version,
    BoltString, BoltType,
};
use begin::Begin;
use bytes::Bytes;
use failure::Failure;
use record::Record;
use run::Run;
pub(crate) use success::Success;

#[derive(Debug, PartialEq, Clone)]
pub enum BoltResponse {
    Success(Success),
    Failure(Failure),
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

    pub fn run(db: Option<&str>, query: &str, params: BoltMap) -> BoltRequest {
        BoltRequest::Run(Run::new(db.map(Into::into), query.into(), params))
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

    pub fn begin(db: Option<&str>) -> BoltRequest {
        let extra = db.into_iter().map(|db| ("db".into(), db.into())).collect();
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
        Err(Error::UnknownMessage(format!(
            "unknown message {:?}",
            response
        )))
    }

    pub fn into_error(self, msg: &'static str) -> Error {
        match self {
            BoltResponse::Failure(failure) => Error::Neo4j(failure.into_error()),
            _ => Error::UnexpectedMessage(format!("unexpected response for {}: {:?}", msg, self)),
        }
    }
}
