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
    SuccessMessage(Success),
    FailureMessage(Failure),
    RecordMessage(Record),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoltRequest {
    HelloMessage(Hello),
    RunMessage(Run),
    PullMessage(Pull),
    DiscardMessage(Discard),
    BeginMessage(Begin),
    CommitMessage(Commit),
    RollbackMessage(Rollback),
    ResetMessage(Reset),
}

impl BoltRequest {
    pub fn hello(agent: &str, principal: String, credentials: String) -> BoltRequest {
        let mut data = BoltMap::default();
        data.put("user_agent".into(), agent.into());
        data.put("scheme".into(), "basic".into());
        data.put("principal".into(), principal.into());
        data.put("credentials".into(), credentials.into());
        BoltRequest::HelloMessage(Hello::new(data))
    }

    pub fn run(db: &str, query: &str, params: BoltMap) -> BoltRequest {
        BoltRequest::RunMessage(Run::new(db.into(), query.into(), params))
    }

    pub fn pull(n: usize, qid: i64) -> BoltRequest {
        BoltRequest::PullMessage(Pull::new(n as i64, qid))
    }

    pub fn discard() -> BoltRequest {
        BoltRequest::DiscardMessage(Discard::default())
    }

    pub fn begin() -> BoltRequest {
        BoltRequest::BeginMessage(Begin::new(BoltMap::default()))
    }

    pub fn commit() -> BoltRequest {
        BoltRequest::CommitMessage(Commit::new())
    }

    pub fn rollback() -> BoltRequest {
        BoltRequest::RollbackMessage(Rollback::new())
    }

    pub fn reset() -> BoltRequest {
        BoltRequest::ResetMessage(Reset::new())
    }
}

impl BoltRequest {
    pub fn into_bytes(self, version: Version) -> Result<Bytes> {
        let bytes: Bytes = match self {
            BoltRequest::HelloMessage(hello) => hello.into_bytes(version)?,
            BoltRequest::RunMessage(run) => run.into_bytes(version)?,
            BoltRequest::PullMessage(pull) => pull.into_bytes(version)?,
            BoltRequest::DiscardMessage(discard) => discard.into_bytes(version)?,
            BoltRequest::BeginMessage(begin) => begin.into_bytes(version)?,
            BoltRequest::CommitMessage(commit) => commit.into_bytes(version)?,
            BoltRequest::RollbackMessage(rollback) => rollback.into_bytes(version)?,
            BoltRequest::ResetMessage(reset) => reset.into_bytes(version)?,
        };
        Ok(bytes)
    }
}

impl BoltResponse {
    pub fn parse(version: Version, response: Bytes) -> Result<BoltResponse> {
        match Rc::new(RefCell::new(response)) {
            input if Success::can_parse(version, input.clone()) => Ok(
                BoltResponse::SuccessMessage(Success::parse(version, input)?),
            ),
            input if Failure::can_parse(version, input.clone()) => Ok(
                BoltResponse::FailureMessage(Failure::parse(version, input)?),
            ),
            input if Record::can_parse(version, input.clone()) => {
                Ok(BoltResponse::RecordMessage(Record::parse(version, input)?))
            }
            msg => Err(Error::UnknownMessage(format!("unknown message {:?}", msg))),
        }
    }
}
