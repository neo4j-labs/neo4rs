use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::row::*;
use crate::types::*;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

const FETCH_SIZE: i64 = 200;

pub struct RowStream {
    qid: i64,
    fields: BoltList,
    state: State,
    buffer: VecDeque<Row>,
    connection: Arc<Mutex<ManagedConnection>>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Ready,
    Streaming,
    Buffered,
    Complete,
}

impl RowStream {
    pub fn new(qid: i64, fields: BoltList, connection: Arc<Mutex<ManagedConnection>>) -> RowStream {
        RowStream {
            qid,
            fields,
            connection,
            buffer: VecDeque::with_capacity(FETCH_SIZE as usize),
            state: State::Ready,
        }
    }

    pub async fn next(&mut self) -> Result<Option<Row>> {
        let mut connection = self.connection.lock().await;
        loop {
            match self.state {
                State::Ready => {
                    let pull = BoltRequest::pull(FETCH_SIZE, self.qid);
                    connection.send(pull).await?;
                    self.state = State::Streaming;
                }
                State::Streaming => match connection.recv().await {
                    Ok(BoltResponse::SuccessMessage(s)) => {
                        if s.get("has_more").unwrap_or(false) {
                            self.state = State::Buffered;
                        } else {
                            self.state = State::Complete;
                        }
                    }
                    Ok(BoltResponse::RecordMessage(record)) => {
                        let row = Row::new(self.fields.clone(), record.data);
                        self.buffer.push_back(row);
                    }
                    msg => {
                        return Err(Error::UnexpectedMessage(format!(
                            "unexpected response for PULL: {:?}",
                            msg
                        )))
                    }
                },
                State::Buffered => {
                    if !self.buffer.is_empty() {
                        return Ok(self.buffer.pop_front());
                    }
                    self.state = State::Ready;
                }
                State::Complete => {
                    return Ok(self.buffer.pop_front());
                }
            }
        }
    }
}
