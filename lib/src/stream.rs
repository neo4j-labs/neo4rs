use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::row::*;
use crate::types::*;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// An abstraction over a stream of rows, this is returned as a result of [`Graph::execute`] or
/// [`Txn::execute`] operations
///
/// A stream will contain a connection from the connection pool which will be released to the pool
/// when the stream is dropped.
pub struct RowStream {
    qid: i64,
    fields: BoltList,
    state: State,
    fetch_size: usize,
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
    pub(crate) fn new(
        qid: i64,
        fields: BoltList,
        fetch_size: usize,
        connection: Arc<Mutex<ManagedConnection>>,
    ) -> RowStream {
        RowStream {
            qid,
            fields,
            connection,
            fetch_size,
            state: State::Ready,
            buffer: VecDeque::with_capacity(fetch_size),
        }
    }

    /// A call to next() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows are fetched from the server (using the
    /// fetch_size value configured see [`ConfigBuilder::fetch_size`])
    pub async fn next(&mut self) -> Result<Option<Row>> {
        let mut connection = self.connection.lock().await;
        loop {
            match self.state {
                State::Ready => {
                    let pull = BoltRequest::pull(self.fetch_size, self.qid);
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
                    msg => return Err(unexpected(msg, "PULL")),
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
