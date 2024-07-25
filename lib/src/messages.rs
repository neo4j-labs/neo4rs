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
};
use begin::Begin;
use bytes::Bytes;
use failure::Failure;
use pull::Pull;
use record::Record;
use run::Run;
use success::Success;

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
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Hello` instead.")
    )]
    Hello(hello::Hello),
    Run(Run),
    Pull(Pull),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Discard` instead.")
    )]
    Discard(discard::Discard),
    Begin(Begin),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Commit` instead.")
    )]
    Commit(commit::Commit),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Rollback` instead.")
    )]
    Rollback(rollback::Rollback),
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Reset` instead.")
    )]
    Reset(reset::Reset),
}

#[cfg_attr(feature = "unstable-bolt-protocol-impl-v2", allow(deprecated))]
impl BoltRequest {
    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Hello` instead.")
    )]
    pub fn hello(agent: &str, principal: &str, credentials: &str) -> BoltRequest {
        let mut data = BoltMap::default();
        data.put("user_agent".into(), agent.into());
        data.put("scheme".into(), "basic".into());
        data.put("principal".into(), principal.into());
        data.put("credentials".into(), credentials.into());
        BoltRequest::Hello(hello::Hello::new(data))
    }

    pub fn run(db: &str, query: &str, params: BoltMap) -> BoltRequest {
        BoltRequest::Run(Run::new(db.into(), query.into(), params))
    }

    pub fn pull(n: usize, qid: i64) -> BoltRequest {
        BoltRequest::Pull(Pull::new(n as i64, qid))
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Discard` instead.")
    )]
    pub fn discard() -> BoltRequest {
        BoltRequest::Discard(discard::Discard::default())
    }

    pub fn begin(db: &str) -> BoltRequest {
        let begin = Begin::new([("db".into(), db.into())].into_iter().collect());
        BoltRequest::Begin(begin)
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Commit` instead.")
    )]
    pub fn commit() -> BoltRequest {
        BoltRequest::Commit(commit::Commit::new())
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Rollback` instead.")
    )]
    pub fn rollback() -> BoltRequest {
        BoltRequest::Rollback(rollback::Rollback::new())
    }

    #[cfg_attr(
        feature = "unstable-bolt-protocol-impl-v2",
        deprecated(since = "0.8.0", note = "Use `crate::bolt::Reset` instead.")
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
            BoltResponse::Failure(failure) => Error::Failure {
                code: failure.code().to_string(),
                message: failure.message().to_string(),
                msg,
            },
            _ => Error::UnexpectedMessage(format!("unexpected response for {}: {:?}", msg, self)),
        }
    }
}
