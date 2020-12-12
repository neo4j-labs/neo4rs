use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::row::*;
use crate::types::*;

#[derive(Clone)]
pub struct Query {
    query: String,
    params: BoltMap,
}

pub struct RowStream {
    qid: i64,
    fields: BoltList,
    state: State,
    connection: ManagedConnection,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Ready,
    Pulling,
    Complete,
}

impl RowStream {
    fn new(qid: i64, fields: BoltList, connection: ManagedConnection) -> RowStream {
        RowStream {
            qid,
            fields,
            connection,
            state: State::Ready,
        }
    }

    pub async fn next(&mut self) -> Result<Option<Row>> {
        while self.state == State::Ready || self.state == State::Pulling {
            match self.state {
                State::Ready => {
                    self.connection.send(BoltRequest::pull(self.qid)).await?;
                    self.state = State::Pulling;
                }
                State::Pulling => match self.connection.recv().await {
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

impl Query {
    pub fn new(query: String) -> Self {
        Query {
            query,
            params: BoltMap::new(),
        }
    }

    pub fn param<T: std::convert::Into<BoltType>>(mut self, key: &str, value: T) -> Self {
        self.params.put(key.into(), value.into());
        self
    }

    pub async fn run(self, connection: &mut ManagedConnection) -> Result<()> {
        let run = BoltRequest::run(&self.query, self.params.clone());
        match connection.send_recv(run).await? {
            BoltResponse::SuccessMessage(_) => {
                match connection.send_recv(BoltRequest::discard()).await? {
                    BoltResponse::SuccessMessage(_) => Ok(()),
                    _ => Err(Error::UnexpectedMessage),
                }
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }

    pub async fn execute(self, mut connection: ManagedConnection) -> Result<RowStream> {
        let run = BoltRequest::run(&self.query, self.params);
        match connection.send_recv(run).await {
            Ok(BoltResponse::SuccessMessage(success)) => {
                let fields: BoltList = success.get("fields").unwrap_or(BoltList::new());
                let qid: i64 = success.get("qid").unwrap_or(-1);
                Ok(RowStream::new(qid, fields, connection))
            }
            msg => {
                eprintln!("unexpected message received: {:?}", msg);
                Err(Error::QueryError)
            }
        }
    }
}
