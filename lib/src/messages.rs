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
use commit::Commit;
use discard::Discard;
use failure::Failure;
use hello::Hello;
use pull::Pull;
use record::Record;
use reset::Reset;
use rollback::Rollback;
use run::Run;
pub(crate) use success::Success;

#[derive(Debug, PartialEq, Clone)]
pub enum BoltResponse {
    Success(Success),
    Failure(Failure),
    Record(Record),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoltRequest {
    Hello(Hello),
    Run(Run),
    Pull(Pull),
    Discard(Discard),
    Begin(Begin),
    Commit(Commit),
    Rollback(Rollback),
    Reset(Reset),
}

pub struct HelloBuilder {
    agent: BoltString,
    principal: BoltString,
    credentials: BoltString,
    routing: Option<BoltMap>,
    version: Version,
}

impl HelloBuilder {
    pub fn new(principal: impl Into<BoltString>, credentials: impl Into<BoltString>) -> Self {
        Self {
            agent: "neo4rs".into(),
            principal: principal.into(),
            credentials: credentials.into(),
            routing: None,
            version: Version::V4,
        }
    }

    pub fn with_routing(self, routing: impl Into<Option<BoltMap>>) -> Self {
        Self {
            routing: routing.into(),
            ..self
        }
    }

    pub fn with_version(self, version: Version) -> Self {
        Self { version, ..self }
    }

    pub fn build(self) -> BoltRequest {
        let HelloBuilder {
            agent,
            principal,
            credentials,
            routing,
            version,
        } = self;
        BoltRequest::hello(agent, principal, credentials, routing, version)
    }
}

impl BoltRequest {
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
        BoltRequest::Hello(Hello::new(data))
    }

    pub fn run(db: &str, query: &str, params: BoltMap) -> BoltRequest {
        BoltRequest::Run(Run::new(db.into(), query.into(), params))
    }

    pub fn pull(n: usize, qid: i64) -> BoltRequest {
        BoltRequest::Pull(Pull::new(n as i64, qid))
    }

    pub fn discard() -> BoltRequest {
        BoltRequest::Discard(Discard::default())
    }

    pub fn begin(db: &str) -> BoltRequest {
        let begin = Begin::new([("db".into(), db.into())].into_iter().collect());
        BoltRequest::Begin(begin)
    }

    pub fn commit() -> BoltRequest {
        BoltRequest::Commit(Commit::new())
    }

    pub fn rollback() -> BoltRequest {
        BoltRequest::Rollback(Rollback::new())
    }

    pub fn reset() -> BoltRequest {
        BoltRequest::Reset(Reset::new())
    }
}

impl BoltRequest {
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
}
