use crate::{
    errors::{unexpected, Error, Result},
    messages::{BoltRequest, BoltResponse},
    pool::ManagedConnection,
    row::Row,
    types::BoltList,
    DeError,
};
use futures::{stream::try_unfold, TryStream};
use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// An abstraction over a stream of rows, this is returned as a result of [`crate::Graph::execute`] or
/// [`crate::Txn::execute`] operations
///
/// A stream will contain a connection from the connection pool which will be released to the pool
/// when the stream is dropped.
#[must_use = "Results must be streamed through with `next` in order to execute the query"]
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
    /// fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
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
                    Ok(BoltResponse::Success(s)) => {
                        if s.get("has_more").unwrap_or(false) {
                            self.state = State::Buffered;
                        } else {
                            self.state = State::Complete;
                        }
                    }
                    Ok(BoltResponse::Record(record)) => {
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

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every element is a [`crate::row::Row`].
    pub fn into_stream(self) -> impl TryStream<Ok = Row, Error = Error> {
        try_unfold(self, |mut stream| async move {
            match stream.next().await {
                Ok(Some(row)) => Ok(Some((row, stream))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every row is converted into a `T` by calling [`crate::row::Row::to`].
    pub fn into_stream_as<T: DeserializeOwned>(self) -> impl TryStream<Ok = T, Error = Error> {
        self.into_stream_de(|row| row.to::<T>())
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a `T`
    /// by calling [`crate::row::Row::get`].
    pub fn column_into_stream<'db, T: DeserializeOwned + 'db>(
        self,
        column: &'db str,
    ) -> impl TryStream<Ok = T, Error = Error> + 'db {
        self.into_stream_de(move |row| row.get::<T>(column))
    }

    fn into_stream_de<T: DeserializeOwned>(
        self,
        deser: impl Fn(Row) -> Result<T, DeError>,
    ) -> impl TryStream<Ok = T, Error = Error> {
        try_unfold((self, deser), |(mut stream, de)| async move {
            match stream.next().await {
                Ok(Some(row)) => match de(row) {
                    Ok(res) => Ok(Some((res, (stream, de)))),
                    Err(e) => Err(Error::DeserializationError(e)),
                },
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }
}
