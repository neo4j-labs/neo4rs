#[cfg(feature = "unstable-streaming-summary")]
use crate::summary::StreamingSummary;

use crate::{
    errors::{Error, Result},
    messages::{BoltRequest, BoltResponse},
    pool::ManagedConnection,
    row::Row,
    txn::TransactionHandle,
    types::BoltList,
    DeError,
};
use futures::{
    stream::{try_unfold, TryStreamExt as _},
    TryStream,
};
use serde::de::DeserializeOwned;
use std::collections::VecDeque;

/// An abstraction over a stream of rows, this is returned as a result of [`crate::Txn::execute`].
///
/// A stream needs a running transaction to be consumed.
#[must_use = "Results must be streamed through with `next` in order to execute the query"]
pub struct RowStream {
    qid: i64,
    fields: BoltList,
    state: State,
    fetch_size: usize,
    buffer: VecDeque<Row>,
}

impl RowStream {
    pub(crate) fn new(qid: i64, fields: BoltList, fetch_size: usize) -> Self {
        RowStream {
            qid,
            fields,
            fetch_size,
            state: State::Ready,
            buffer: VecDeque::with_capacity(fetch_size),
        }
    }
}

/// An abstraction over a stream of rows, this is returned as a result of [`crate::Graph::execute`].
///
/// A stream will contain a connection from the connection pool which will be released to the pool
/// when the stream is dropped.
#[must_use = "Results must be streamed through with `next` in order to execute the query"]
pub struct DetachedRowStream {
    stream: RowStream,
    connection: ManagedConnection,
}

impl DetachedRowStream {
    pub(crate) fn new(stream: RowStream, connection: ManagedConnection) -> Self {
        DetachedRowStream { stream, connection }
    }
}

#[derive(Clone, Debug)]
pub enum RowItem<T = Row> {
    Row(T),
    #[cfg(feature = "unstable-streaming-summary")]
    Summary(Box<StreamingSummary>),
    Done,
}

impl<T> RowItem<T> {
    pub fn row(&self) -> Option<&T> {
        match self {
            RowItem::Row(row) => Some(row),
            _ => None,
        }
    }

    #[cfg(feature = "unstable-streaming-summary")]
    pub fn summary(&self) -> Option<&StreamingSummary> {
        match self {
            RowItem::Summary(summary) => Some(summary),
            _ => None,
        }
    }

    pub fn into_row(self) -> Option<T> {
        match self {
            RowItem::Row(row) => Some(row),
            _ => None,
        }
    }

    #[cfg(feature = "unstable-streaming-summary")]
    pub fn into_summary(self) -> Option<Box<StreamingSummary>> {
        match self {
            RowItem::Summary(summary) => Some(summary),
            _ => None,
        }
    }
}

impl RowStream {
    /// A call to next() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next(&mut self, handle: impl TransactionHandle) -> Result<Option<Row>> {
        self.next_or_summary(handle)
            .await
            .map(|item| item.into_row())
    }

    /// A call to next_or_summary() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next_or_summary(&mut self, mut handle: impl TransactionHandle) -> Result<RowItem> {
        loop {
            if let Some(row) = self.buffer.pop_front() {
                return Ok(RowItem::Row(row));
            }

            match self.state {
                State::Ready => {
                    let pull = BoltRequest::pull(self.fetch_size, self.qid);
                    let connection = handle.connection();
                    connection.send(pull).await?;

                    self.state = loop {
                        match connection.recv().await {
                            Ok(BoltResponse::Success(s)) => {
                                break if s.get("has_more").unwrap_or(false) {
                                    State::Ready
                                } else {
                                    State::Complete(None)
                                };
                            }
                            Ok(BoltResponse::Record(record)) => {
                                let row = Row::new(self.fields.clone(), record.data);
                                self.buffer.push_back(row);
                            }
                            Ok(msg) => return Err(msg.into_error("PULL")),
                            Err(e) => return Err(e),
                        }
                    };
                }
                State::Complete(ref mut _summary) => {
                    #[cfg(feature = "unstable-streaming-summary")]
                    return match _summary.take() {
                        Some(summary) => Ok(RowItem::Summary(summary)),
                        None => Ok(RowItem::Done),
                    };
                    #[cfg(not(feature = "unstable-streaming-summary"))]
                    return Ok(RowItem::Done);
                }
            }
        }
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every element is a [`crate::row::Row`].
    pub fn into_stream(
        self,
        handle: impl TransactionHandle,
    ) -> impl TryStream<Ok = Row, Error = Error> {
        self.into_stream_convert(handle, Ok)
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every row is converted into a `T` by calling [`crate::row::Row::to`].
    pub fn into_stream_as<T: DeserializeOwned>(
        self,
        handle: impl TransactionHandle,
    ) -> impl TryStream<Ok = T, Error = Error> {
        self.into_stream_convert(handle, |row| row.to::<T>())
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a `T`
    /// by calling [`crate::row::Row::get`].
    pub fn column_into_stream<'db, T: DeserializeOwned + 'db>(
        self,
        handle: impl TransactionHandle + 'db,
        column: &'db str,
    ) -> impl TryStream<Ok = T, Error = Error> + 'db {
        self.into_stream_convert(handle, move |row| row.get::<T>(column))
    }

    fn into_stream_convert<T>(
        self,
        handle: impl TransactionHandle,
        convert: impl Fn(Row) -> Result<T, DeError>,
    ) -> impl TryStream<Ok = T, Error = Error> {
        self.into_stream_convert_and_summary(handle, convert)
            .try_filter_map(|row| async { Ok(row.into_row()) })
    }

    fn into_stream_convert_and_summary<T>(
        self,
        handle: impl TransactionHandle,
        convert: impl Fn(Row) -> Result<T, DeError>,
    ) -> impl TryStream<Ok = RowItem<T>, Error = Error> {
        try_unfold(
            (self, handle, convert),
            |(mut stream, mut hd, de)| async move {
                match stream.next_or_summary(&mut hd).await {
                    Ok(RowItem::Row(row)) => match de(row) {
                        Ok(res) => Ok(Some((RowItem::Row(res), (stream, hd, de)))),
                        Err(e) => Err(Error::DeserializationError(e)),
                    },
                    #[cfg(feature = "unstable-streaming-summary")]
                    Ok(RowItem::Summary(summary)) => {
                        Ok(Some((RowItem::Summary(summary), (stream, hd, de))))
                    }
                    Ok(RowItem::Done) => Ok(None),
                    Err(e) => Err(e),
                }
            },
        )
    }
}

impl DetachedRowStream {
    /// A call to next() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next(&mut self) -> Result<Option<Row>> {
        self.stream.next(&mut self.connection).await
    }

    /// A call to next_or_summary() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next_or_summary(&mut self) -> Result<RowItem> {
        self.stream.next_or_summary(&mut self.connection).await
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every element is a [`crate::row::Row`].
    pub fn into_stream(self) -> impl TryStream<Ok = Row, Error = Error> {
        self.stream.into_stream(self.connection)
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every row is converted into a `T` by calling [`crate::row::Row::to`].
    pub fn into_stream_as<T: DeserializeOwned>(self) -> impl TryStream<Ok = T, Error = Error> {
        self.stream.into_stream_as(self.connection)
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a `T`
    /// by calling [`crate::row::Row::get`].
    pub fn column_into_stream<'db, T: DeserializeOwned + 'db>(
        self,
        column: &'db str,
    ) -> impl TryStream<Ok = T, Error = Error> + 'db {
        self.stream.column_into_stream(self.connection, column)
    }
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    Ready,
    #[cfg(feature = "unstable-streaming-summary")]
    Complete(Option<Box<StreamingSummary>>),
    #[cfg(not(feature = "unstable-streaming-summary"))]
    Complete(Option<()>),
}
