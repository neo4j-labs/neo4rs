#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
use crate::messages::{BoltRequest, BoltResponse};
#[cfg(feature = "unstable-result-summary")]
use crate::summary::{ResultSummary, Streaming};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{
    bolt::{Bolt, Discard, Pull, Response, WrapExtra as _},
    BoltType,
};
use crate::{
    errors::{Error, Result},
    pool::ManagedConnection,
    row::Row,
    txn::TransactionHandle,
    types::BoltList,
    DeError, RunResult,
};

use futures::{stream::try_unfold, TryStream};
use serde::de::DeserializeOwned;

use std::collections::VecDeque;

#[cfg(feature = "unstable-result-summary")]
type BoxedSummary = Box<ResultSummary>;
#[cfg(not(feature = "unstable-result-summary"))]
type BoxedSummary = ();

/// An abstraction over a stream of rows, this is returned as a result of [`crate::Txn::execute`].
///
/// A stream needs a running transaction to be consumed.
#[must_use = "Results must be streamed through with `next` in order to execute the query"]
pub struct RowStream {
    qid: i64,
    fields: BoltList,
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    available_after: i64,
    state: State,
    fetch_size: usize,
    buffer: VecDeque<Row>,
}

impl RowStream {
    pub(crate) fn new(
        qid: i64,
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")] available_after: i64,
        fields: BoltList,
        fetch_size: usize,
        state: State,
    ) -> Self {
        RowStream {
            qid,
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            available_after,
            fields,
            fetch_size,
            state,
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

impl RowStream {
    /// A call to next() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next(&mut self, mut handle: impl TransactionHandle) -> Result<Option<Row>> {
        loop {
            if let Some(row) = self.buffer.pop_front() {
                return Ok(Some(row));
            }

            match self.state {
                State::Ready => self.pull(handle.connection()).await?,
                State::Pulling | State::Discarding => self.receive(handle.connection()).await?,
                State::Complete(_) => return Ok(None),
            }
        }
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    async fn pull(&mut self, connection: &mut ManagedConnection) -> Result<()> {
        let pull = Pull::some(self.fetch_size as i64).for_query(self.qid);
        connection.send_as(pull).await?;
        self.state = State::Pulling;
        Ok(())
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    async fn pull(&mut self, connection: &mut ManagedConnection) -> Result<()> {
        let pull = BoltRequest::pull(self.fetch_size, self.qid);
        connection.send(pull).await?;
        self.state = State::Pulling;
        Ok(())
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    async fn discard(&mut self, connection: &mut ManagedConnection) -> Result<()> {
        let discard = Discard::all().for_query(self.qid);
        connection.send_as(discard).await?;
        self.state = State::Discarding;
        Ok(())
    }

    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    async fn discard(&mut self, connection: &mut ManagedConnection) -> Result<()> {
        let discard = BoltRequest::discard_all_for(self.qid);
        connection.send(discard).await?;
        self.state = State::Discarding;
        Ok(())
    }

    /// Receives the responses to the request in flight until its summary arrives.
    ///
    /// An error while decoding a single record leaves the state untouched, so the
    /// next call continues to receive the same batch and the connection stays in sync.
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    async fn receive(&mut self, connection: &mut ManagedConnection) -> Result<()> {
        loop {
            let response = connection
                .recv_as::<Response<Vec<Bolt>, Streaming>>()
                .await?;
            match response {
                Response::Detail(record) => {
                    let record = BoltList::from(
                        record
                            .into_iter()
                            .map(BoltType::from)
                            .collect::<Vec<BoltType>>(),
                    );
                    let row = Row::new(self.fields.clone(), record);
                    self.buffer.push_back(row);
                }
                Response::Success(Streaming::HasMore) => {
                    let discarding = self.state == State::Discarding;
                    self.state = State::Ready;
                    if discarding {
                        return Err(Error::UnexpectedMessage(
                            "Query returned has_more after a discard_all".into(),
                        ));
                    }
                    return Ok(());
                }
                Response::Success(Streaming::Done(mut s)) => {
                    s.set_t_first(self.available_after);
                    self.state = State::Complete(s);
                    return Ok(());
                }
                otherwise => {
                    let request = self.request_in_flight();
                    self.state = State::Ready;
                    return Err(otherwise.into_error(request));
                }
            }
        }
    }

    /// Receives the responses to the request in flight until its summary arrives.
    ///
    /// An error while decoding a single record leaves the state untouched, so the
    /// next call continues to receive the same batch and the connection stays in sync.
    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    async fn receive(&mut self, connection: &mut ManagedConnection) -> Result<()> {
        loop {
            match connection.recv().await? {
                BoltResponse::Success(s) => {
                    let has_more =
                        self.state == State::Pulling && s.get("has_more").unwrap_or(false);
                    self.state = if has_more {
                        State::Ready
                    } else {
                        State::Complete(())
                    };
                    return Ok(());
                }
                BoltResponse::Record(record) => {
                    let row = Row::new(self.fields.clone(), record.data);
                    self.buffer.push_back(row);
                }
                otherwise => {
                    let request = self.request_in_flight();
                    self.state = State::Ready;
                    return Err(otherwise.into_error(request));
                }
            }
        }
    }

    fn request_in_flight(&self) -> &'static str {
        match self.state {
            State::Discarding => "DISCARD",
            _ => "PULL",
        }
    }

    /// Return the [`RowStream::next`] item,
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// Unlike `next`, this method returns a missing items as an error ([`Error::NoMoreRows`]).
    pub async fn next_as<'this, 'db: 'this, T: DeserializeOwned + 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
    ) -> Result<T> {
        self.next(handle)
            .await
            .and_then(|row| row.ok_or(Error::NoMoreRows))
            .and_then(|row| row.to::<T>().map_err(Error::DeserializationError))
    }

    /// Return the first [`crate::Row`] in the result.
    ///
    /// If there are 0 results, [`Error::NoMoreRows`] is returned.
    /// If there are 2 or more results, [`Error::NotSingleResult`] is returned.
    pub async fn single(&mut self, mut handle: impl TransactionHandle) -> Result<Row> {
        let row = self
            .next(&mut handle)
            .await
            .and_then(|row| row.ok_or(Error::NoMoreRows))?;
        let None = self.next(handle).await? else {
            return Err(Error::NotSingleResult);
        };
        Ok(row)
    }

    /// Return the first [`crate::Row`] in the result.
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// If there are 0 results, [`Error::NoMoreRows`] is returned.
    /// If there are 2 or more results, [`Error::NotSingleResult`] is returned.
    pub async fn single_as<'this, 'db: 'this, T: DeserializeOwned + 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
    ) -> Result<T> {
        self.single(handle)
            .await
            .and_then(|row| row.to::<T>().map_err(Error::DeserializationError))
    }

    /// Return the first [`crate::Row`] buffered result without consuming it.
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning `None` does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn peek(&self) -> Option<&Row> {
        self.buffer.front()
    }

    /// Return the first [`crate::Row`] buffered result without consuming it,
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning [`Error::NoMoreRows`] does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn peek_as<'this, T: DeserializeOwned + 'this>(&'this self) -> Result<T> {
        self.peek()
            .ok_or(Error::NoMoreRows)
            .and_then(|row| row.to::<T>().map_err(Error::DeserializationError))
    }

    /// Return the first [`crate::Row`] buffered result and consume it.
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning `None` does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn pop(&mut self) -> Option<Row> {
        self.buffer.pop_front()
    }

    /// Return the first [`crate::Row`] buffered result and consume it,
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning [`Error::NoMoreRows`] does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn pop_as<'this, T: DeserializeOwned + 'this>(&'this mut self) -> Result<T> {
        self.pop()
            .ok_or(Error::NoMoreRows)
            .and_then(|row| row.to::<T>().map_err(Error::DeserializationError))
    }

    /// Stop consuming the stream and return a summary, if available.
    /// Stopping the stream will also discard any messages on the server side.
    pub async fn finish(mut self, mut handle: impl TransactionHandle) -> Result<RunResult> {
        loop {
            self.buffer.clear();
            match self.state {
                State::Ready => self.discard(handle.connection()).await?,
                State::Pulling | State::Discarding => self.receive(handle.connection()).await?,
                State::Complete(_) => break,
            }
        }
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        match self.state {
            State::Complete(summary) => Ok(*summary),
            _ => unreachable!("the loop only ends once the stream is complete"),
        }

        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        Ok(())
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every element is a [`crate::row::Row`].
    ///
    /// The stream can only be converted once.
    /// After the returned stream is consumed, this stream can be [`Self::finish`]ed to get the summary.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_stream<'this, 'db: 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
    ) -> impl TryStream<Ok = Row, Error = Error> + 'this {
        self.convert_rows(handle, Ok)
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every row is converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// The stream can only be converted once.
    /// After the returned stream is consumed, this stream can be [`Self::finish`]ed to get the summary.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_stream_as<'this, 'db: 'this, T: DeserializeOwned + 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
    ) -> impl TryStream<Ok = T, Error = Error> + 'this {
        self.convert_rows(handle, |row| row.to::<T>())
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a `T`
    /// by calling [`crate::row::Row::get`].
    ///
    /// The stream can only be converted once.
    /// After the returned stream is consumed, this stream can be [`Self::finish`]ed to get the summary.
    pub fn column_into_stream<'this, 'db: 'this, T: DeserializeOwned + 'db>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
        column: &'db str,
    ) -> impl TryStream<Ok = T, Error = Error> + 'this {
        self.convert_rows(handle, move |row| row.get::<T>(column))
    }

    fn convert_rows<'this, 'db: 'this, T: 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
        convert: impl Fn(Row) -> Result<T, DeError> + 'this,
    ) -> impl TryStream<Ok = T, Error = Error> + 'this {
        try_unfold((self, handle, convert), |(stream, mut hd, de)| async move {
            match stream.next(&mut hd).await? {
                Some(row) => match de(row) {
                    Ok(res) => Ok(Some((res, (stream, hd, de)))),
                    Err(e) => Err(Error::DeserializationError(e)),
                },
                None => Ok(None),
            }
        })
    }
}

impl DetachedRowStream {
    /// A call to next() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next(&mut self) -> Result<Option<Row>> {
        self.stream.next(&mut self.connection).await
    }

    /// Return the [`RowStream::next`] item,
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// Unlike `next`, this method returns a missing items as an error ([`Error::NoMoreRows`]).
    pub async fn next_as<'this, T: DeserializeOwned + 'this>(&'this mut self) -> Result<T> {
        self.stream.next_as(&mut self.connection).await
    }

    /// Return the first [`crate::Row`] in the result.
    ///
    /// If there are 0 results, [`Error::NoMoreRows`] is returned.
    /// If there are 2 or more results, [`Error::NotSingleResult`] is returned.
    pub async fn single(&mut self) -> Result<Row> {
        self.stream.single(&mut self.connection).await
    }

    /// Return the first [`crate::Row`] in the result.
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// If there are 0 results, [`Error::NoMoreRows`] is returned.
    /// If there are 2 or more results, [`Error::NotSingleResult`] is returned.
    pub async fn single_as<'this, T: DeserializeOwned + 'this>(&'this mut self) -> Result<T> {
        self.stream.single_as(&mut self.connection).await
    }

    /// Return the first [`crate::Row`] buffered result without consuming it.
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning `None` does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn peek(&self) -> Option<&Row> {
        self.stream.peek()
    }

    /// Return the first [`crate::Row`] buffered result without consuming it,
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning [`Error::NoMoreRows`] does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn peek_as<'this, T: DeserializeOwned + 'this>(&'this self) -> Result<T> {
        self.stream.peek_as()
    }

    /// Return the first [`crate::Row`] buffered result and consume it.
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning `None` does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn pop(&mut self) -> Option<Row> {
        self.stream.pop()
    }

    /// Return the first [`crate::Row`] buffered result and consume it,
    /// converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// Note that this method is not `async` and does not require a `TransactionHandle` because
    /// only buffered results are inspected and no actual communication is done.
    ///
    /// As such, returning [`Error::NoMoreRows`] does not mean that there are no more results,
    /// it just means that the buffer is empty.
    pub fn pop_as<'this, T: DeserializeOwned + 'this>(&'this mut self) -> Result<T> {
        self.stream.pop_as()
    }

    /// Stop consuming the stream and return a summary, if available.
    /// Stopping the stream will also discard any messages on the server side.
    pub async fn finish(mut self) -> Result<RunResult> {
        self.stream.finish(&mut self.connection).await
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every element is a [`crate::row::Row`].
    ///
    /// The stream can only be converted once.
    /// After the returned stream is consumed, this stream can be [`Self::finish`]ed to get the summary.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_stream(&mut self) -> impl TryStream<Ok = Row, Error = Error> + '_ {
        self.stream.into_stream(&mut self.connection)
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// every row is converted into a `T` by calling [`crate::row::Row::to`].
    ///
    /// The stream can only be converted once.
    /// After the returned stream is consumed, this stream can be [`Self::finish`]ed to get the summary.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_stream_as<'this, T: DeserializeOwned + 'this>(
        &'this mut self,
    ) -> impl TryStream<Ok = T, Error = Error> + 'this {
        self.stream.into_stream_as(&mut self.connection)
    }

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a `T`
    /// by calling [`crate::row::Row::get`].
    ///
    /// The stream can only be converted once.
    /// After the returned stream is consumed, this stream can be [`Self::finish`]ed to get the summary.
    pub fn column_into_stream<'this, 'db: 'this, T: DeserializeOwned + 'db>(
        &'this mut self,
        column: &'db str,
    ) -> impl TryStream<Ok = T, Error = Error> + 'this {
        self.stream.column_into_stream(&mut self.connection, column)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum State {
    /// No request is in flight, the next batch of records must be requested with PULL.
    Ready,
    /// A PULL is in flight: its records and then its summary are still to be received.
    Pulling,
    /// A DISCARD is in flight: only its summary is still to be received.
    Discarding,
    Complete(BoxedSummary),
}
