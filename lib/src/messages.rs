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
use crate::errors::*;
use crate::types::*;
use crate::version::Version;
use begin::Begin;
use bytes::*;
use commit::Commit;
use discard::Discard;
use failure::Failure;
use hello::Hello;
use pull::Pull;
use record::Record;
use reset::Reset;
use rollback::Rollback;
use run::Run;
use std::cell::RefCell;
use std::rc::Rc;
use success::Success;

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

impl BoltRequest {
    pub fn hello(agent: &str, principal: &str, credentials: &str) -> BoltRequest {
        let mut data = BoltMap::default();
        data.put("user_agent".into(), agent.into());
        data.put("scheme".into(), "basic".into());
        data.put("principal".into(), principal.into());
        data.put("credentials".into(), credentials.into());
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

    pub fn begin() -> BoltRequest {
        BoltRequest::Begin(Begin::new(BoltMap::default()))
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
    pub fn parse(version: Version, response: Bytes) -> Result<BoltResponse> {
        match Rc::new(RefCell::new(response)) {
            input if Success::can_parse(version, input.clone()) => {
                Ok(BoltResponse::Success(Success::parse(version, input)?))
            }
            input if Failure::can_parse(version, input.clone()) => {
                Ok(BoltResponse::Failure(Failure::parse(version, input)?))
            }
            input if Record::can_parse(version, input.clone()) => {
                Ok(BoltResponse::Record(Record::parse(version, input)?))
            }
            msg => Err(Error::UnknownMessage(format!("unknown message {:?}", msg))),
        }
    }
}
