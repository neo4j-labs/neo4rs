mod begin;
mod bye;
mod commit;
mod discard;
mod failure;
mod hello;
mod pull;
mod record;
mod rollback;
mod run;
mod success;
use crate::errors::*;
use crate::types::*;
use begin::Begin;
use bye::Bye;
use bytes::*;
use commit::Commit;
use discard::Discard;
use failure::Failure;
use hello::Hello;
use pull::Pull;
use record::Record;
use rollback::Rollback;
use run::Run;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;
use success::Success;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BoltResponse {
    SuccessMessage(Success),
    FailureMessage(Failure),
    RecordMessage(Record),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BoltRequest {
    HelloMessage(Hello),
    RunMessage(Run),
    GoodByeMessage(Bye),
    PullMessage(Pull),
    DiscardMessage(Discard),
    BeginMessage(Begin),
    CommitMessage(Commit),
    RollbackMessage(Rollback),
}

impl BoltRequest {
    pub fn hello(agent: &str, principal: String, credentials: String) -> BoltRequest {
        let mut data = BoltMap::new();
        data.put("user_agent".into(), agent.into());
        data.put("scheme".into(), "basic".into());
        data.put("principal".into(), principal.into());
        data.put("credentials".into(), credentials.into());
        BoltRequest::HelloMessage(Hello::new(data))
    }

    pub fn run(query: &str, params: BoltMap) -> BoltRequest {
        BoltRequest::RunMessage(Run::new(query.into(), params))
    }

    pub fn pull() -> BoltRequest {
        BoltRequest::PullMessage(Pull::default())
    }

    pub fn discard() -> BoltRequest {
        BoltRequest::DiscardMessage(Discard::default())
    }

    pub fn begin() -> BoltRequest {
        BoltRequest::BeginMessage(Begin::new(BoltMap::new()))
    }

    pub fn commit() -> BoltRequest {
        BoltRequest::CommitMessage(Commit::new())
    }

    pub fn rollback() -> BoltRequest {
        BoltRequest::RollbackMessage(Rollback::new())
    }
}

impl TryInto<Bytes> for BoltRequest {
    type Error = Error;
    fn try_into(self) -> Result<Bytes> {
        let bytes: Bytes = match self {
            BoltRequest::HelloMessage(hello) => hello.try_into()?,
            BoltRequest::GoodByeMessage(bye) => bye.try_into()?,
            BoltRequest::RunMessage(run) => run.try_into()?,
            BoltRequest::PullMessage(pull) => pull.try_into()?,
            BoltRequest::DiscardMessage(discard) => discard.try_into()?,
            BoltRequest::BeginMessage(begin) => begin.try_into()?,
            BoltRequest::CommitMessage(commit) => commit.try_into()?,
            BoltRequest::RollbackMessage(rollback) => rollback.try_into()?,
        };
        Ok(bytes)
    }
}

impl TryFrom<Bytes> for BoltResponse {
    type Error = Error;

    fn try_from(response: Bytes) -> Result<BoltResponse> {
        match Rc::new(RefCell::new(response)) {
            input if Success::can_parse(input.clone()) => {
                Ok(BoltResponse::SuccessMessage(Success::try_from(input)?))
            }
            input if Failure::can_parse(input.clone()) => {
                Ok(BoltResponse::FailureMessage(Failure::try_from(input)?))
            }
            input if Record::can_parse(input.clone()) => {
                Ok(BoltResponse::RecordMessage(Record::try_from(input)?))
            }
            _ => Err(Error::UnknownMessage),
        }
    }
}
