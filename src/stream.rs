use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::row::*;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RowStream {
    qid: i64,
    fields: BoltList,
    state: State,
    connection: Arc<Mutex<ManagedConnection>>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Ready,
    Pulling,
    Complete,
}

impl RowStream {
    pub fn new(qid: i64, fields: BoltList, connection: Arc<Mutex<ManagedConnection>>) -> RowStream {
        RowStream {
            qid,
            fields,
            connection,
            state: State::Ready,
        }
    }

    pub async fn next(&mut self) -> Result<Option<Row>> {
        let mut connection = self.connection.lock().await;
        while self.state == State::Ready || self.state == State::Pulling {
            match self.state {
                State::Ready => {
                    connection.send(BoltRequest::pull(self.qid)).await?;
                    self.state = State::Pulling;
                }
                State::Pulling => match connection.recv().await {
                    Ok(BoltResponse::SuccessMessage(s)) => {
                        if s.get("has_more").unwrap_or(false) {
                            self.state = State::Ready;
                        } else {
                            self.state = State::Complete;
                            return Ok(None);
                        }
                    }
                    Ok(BoltResponse::RecordMessage(record)) => {
                        let row = Row::new(self.fields.clone(), record.data);
                        return Ok(Some(row));
                    }
                    msg => {
                        eprintln!("Got unexpected message: {:?}", msg);
                        return Err(Error::QueryError);
                    }
                },
                state => panic!("invalid state {:?}", state),
            }
        }
        Ok(None)
    }
}
