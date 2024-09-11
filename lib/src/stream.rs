#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
use crate::messages::{BoltRequest, BoltResponse};
#[cfg(feature = "unstable-result-summary")]
use crate::summary::{ResultSummary, Streaming};
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{
    bolt::{Bolt, Discard, Pull, Response, Summary, WrapExtra as _},
    BoltType,
};
use crate::{
    errors::{Error, Result},
    pool::ManagedConnection,
    row::Row,
    txn::TransactionHandle,
    types::BoltList,
    DeError,
};

use futures::{stream::try_unfold, TryStream};
use serde::de::DeserializeOwned;

use std::{collections::VecDeque, sync::Arc, sync::OnceLock};

#[cfg(feature = "unstable-result-summary")]
type BoxedSummary = Box<ResultSummary>;
#[cfg(not(feature = "unstable-result-summary"))]
type BoxedSummary = ();

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
type FinishResult = Option<ResultSummary>;
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
type FinishResult = ();

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
    buffer: VecDeque<BoltList>,
}

impl RowStream {
    pub(crate) fn new(
        qid: i64,
        #[cfg(feature = "unstable-bolt-protocol-impl-v2")] available_after: i64,
        fields: BoltList,
        fetch_size: usize,
    ) -> Self {
        RowStream {
            qid,
            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            available_after,
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
    #[cfg(feature = "unstable-result-summary")]
    Summary(Box<ResultSummary>),
}

impl<T> RowItem<T> {
    pub fn row(&self) -> Option<&T> {
        match self {
            RowItem::Row(row) => Some(row),
            #[cfg(feature = "unstable-result-summary")]
            _ => None,
        }
    }

    #[cfg(feature = "unstable-result-summary")]
    pub fn summary(&self) -> Option<&ResultSummary> {
        match self {
            RowItem::Summary(summary) => Some(summary),
            _ => None,
        }
    }

    pub fn into_row(self) -> Option<T> {
        match self {
            RowItem::Row(row) => Some(row),
            #[cfg(feature = "unstable-result-summary")]
            _ => None,
        }
    }

    #[cfg(feature = "unstable-result-summary")]
    pub fn into_summary(self) -> Option<Box<ResultSummary>> {
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
            .map(|item| item.and_then(RowItem::into_row))
    }

    /// A call to next_or_summary() will return a row from an internal buffer if the buffer has any entries,
    /// if the buffer is empty and the server has more rows left to consume, then a new batch of rows
    /// are fetched from the server (using the fetch_size value configured see [`crate::ConfigBuilder::fetch_size`])
    pub async fn next_or_summary(
        &mut self,
        mut handle: impl TransactionHandle,
    ) -> Result<Option<RowItem>> {
        loop {
            if let Some(record) = self.buffer.pop_front() {
                let row = Row::new(self.fields.clone(), record);
                return Ok(Some(RowItem::Row(row)));
            }

            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            {
                if self.state == State::Ready {
                    let pull = Pull::some(self.fetch_size as i64).for_query(self.qid);
                    let connection = handle.connection();
                    connection.send_as(pull).await?;
                    self.state = loop {
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
                                self.buffer.push_back(record);
                            }
                            Response::Success(Streaming::HasMore) => break State::Ready,
                            Response::Success(Streaming::Done(mut s)) => {
                                s.set_t_first(self.available_after);
                                break State::Complete(Some(s));
                            }
                            otherwise => return Err(otherwise.into_error("PULL")),
                        }
                    };
                } else if let State::Complete(ref mut summary) = self.state {
                    break match summary.take() {
                        Some(summary) => Ok(Some(RowItem::Summary(summary))),
                        None => Ok(None),
                    };
                }
            }

            #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
            {
                if self.state == State::Ready {
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
                } else if let State::Complete(_) = self.state {
                    break Ok(None);
                };
            }
        }
    }

    /// Stop consuming the stream and return a summary, if available.
    /// Stopping the stream will also discard any messages on the server side.
    pub async fn finish(mut self, mut handle: impl TransactionHandle) -> Result<FinishResult> {
        self.buffer.clear();

        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        match self.state {
            State::Ready => {
                let summary = {
                    let connected = handle.connection();
                    connected
                        .send_recv_as(Discard::all().for_query(self.qid))
                        .await
                }?;
                let summary = match summary {
                    Summary::Success(s) => match s.metadata {
                        Streaming::Done(summary) => Some(*summary),
                        Streaming::HasMore => {
                            // this should never happen
                            None
                        }
                    },
                    Summary::Ignored => None,
                    Summary::Failure(f) => {
                        self.state = State::Complete(None);
                        return Err(f.into_error());
                    }
                };
                self.state = State::Complete(None);
                Ok(summary)
            }
            State::Complete(summary) => Ok(summary.map(|o| *o)),
        }

        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        match self.state {
            State::Ready => {
                let summary = {
                    let connected = handle.connection();
                    connected
                        .send_recv(BoltRequest::discard_all_for(self.qid))
                        .await
                }?;
                let summary = match summary {
                    crate::messages::BoltResponse::Success(_) => Ok(()),
                    crate::messages::BoltResponse::Failure(f) => Err(Error::Neo4j(f.into_error())),
                    msg => Err(msg.into_error("DISCARD")),
                };
                self.state = State::Complete(None);
                summary
            }
            State::Complete(_) => Ok(()),
        }
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
    /// every element is a [`RowItem`].
    ///
    /// The stream can only be converted once.
    pub fn as_row_items<'this, 'db: 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
    ) -> impl TryStream<Ok = RowItem, Error = Error> + 'this {
        self.convert_with_summary(handle, Ok)
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
    /// every row is converted into a [`RowItem<T>`] by calling [`crate::row::Row::to`].
    ///
    /// The stream can only be converted once.
    pub fn as_items<'this, 'db: 'this, T: DeserializeOwned + 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
    ) -> impl TryStream<Ok = RowItem<T>, Error = Error> + 'this {
        self.convert_with_summary(handle, |row| row.to::<T>())
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

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a [`RowItem<T>`]
    /// by calling [`crate::row::Row::get`].
    ///
    /// The stream can only be converted once.
    pub fn column_to_items<'this, 'db: 'this, T: DeserializeOwned + 'db>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
        column: &'db str,
    ) -> impl TryStream<Ok = RowItem<T>, Error = Error> + 'this {
        self.convert_with_summary(handle, move |row| row.get::<T>(column))
    }

    fn convert_rows<'this, 'db: 'this, T: 'this>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
        convert: impl Fn(Row) -> Result<T, DeError> + 'this,
    ) -> impl TryStream<Ok = T, Error = Error> + 'this {
        try_unfold((self, handle, convert), |(stream, mut hd, de)| async move {
            match stream.next_or_summary(&mut hd).await {
                Ok(Some(RowItem::Row(row))) => match de(row) {
                    Ok(res) => Ok(Some((res, (stream, hd, de)))),
                    Err(e) => Err(Error::DeserializationError(e)),
                },
                #[cfg(feature = "unstable-result-summary")]
                Ok(Some(RowItem::Summary(summary))) => {
                    stream.state = State::Complete(Some(summary));
                    Ok(None)
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    fn convert_with_summary<'this, 'db: 'this, T>(
        &'this mut self,
        handle: impl TransactionHandle + 'db,
        convert: impl Fn(Row) -> Result<T, DeError> + 'this,
    ) -> impl TryStream<Ok = RowItem<T>, Error = Error> + 'this {
        try_unfold((self, handle, convert), |(stream, mut hd, de)| async move {
            match stream.next_or_summary(&mut hd).await {
                Ok(Some(RowItem::Row(row))) => match de(row) {
                    Ok(res) => Ok(Some((RowItem::Row(res), (stream, hd, de)))),
                    Err(e) => Err(Error::DeserializationError(e)),
                },
                #[cfg(feature = "unstable-result-summary")]
                Ok(Some(RowItem::Summary(summary))) => {
                    Ok(Some((RowItem::Summary(summary), (stream, hd, de))))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    #[cfg(all(feature = "polars_v0_43", not(feature = "unstable-result-summary")))]
    pub async fn into_dataframe(
        self,
        mut handle: impl TransactionHandle,
    ) -> Result<polars::frame::DataFrame> {
        self.into_df(handle).await
    }

    #[cfg(all(feature = "polars_v0_43", feature = "unstable-result-summary"))]
    pub async fn into_dataframe(
        self,
        handle: impl TransactionHandle,
    ) -> Result<(polars::frame::DataFrame, Option<ResultSummary>)> {
        let out_summary = Arc::new(OnceLock::new());
        let df = self.into_df(handle, out_summary.clone()).await?;
        let summary = Arc::into_inner(out_summary).and_then(|s| s.into_inner());
        Ok((df, summary))
    }

    #[cfg(feature = "polars_v0_43")]
    fn into_df(
        mut self,
        mut handle: impl TransactionHandle,
        #[cfg(feature = "unstable-result-summary")] out_summary: Arc<OnceLock<ResultSummary>>,
    ) -> impl std::future::Future<Output = Result<polars::frame::DataFrame, Error>> {
        let fields = self.fields.value.iter().filter_map(|x| match x {
            BoltType::String(s) => Some(s.value.as_str()),
            _ => None,
        });

        let mut buf = pl::DataBuf::new(fields);

        for row in self.buffer.drain(..) {
            buf.push(row.value);
        }

        async move {
            while self.state == State::Ready {
                #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
                {
                    let pull = Pull::some(self.fetch_size as i64).for_query(self.qid);
                    let connection = handle.connection();
                    connection.send_as(pull).await?;
                    self.state = loop {
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
                                buf.push(record.value);
                            }
                            Response::Success(Streaming::HasMore) => break State::Ready,
                            Response::Success(Streaming::Done(mut s)) => {
                                s.set_t_first(self.available_after);
                                break State::Complete(Some(s));
                            }
                            otherwise => return Err(otherwise.into_error("PULL")),
                        }
                    };
                    buf.flush()?;
                }

                #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
                {
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
                                buf.push(record.data);
                            }
                            Ok(msg) => return Err(msg.into_error("PULL")),
                            Err(e) => return Err(e),
                        }
                    };
                    buf.flush()?;
                }
            }

            #[cfg(feature = "unstable-result-summary")]
            if let State::Complete(ref mut summary) = self.state {
                if let Some(summary) = summary.take() {
                    out_summary.set(*summary).expect("only one summary");
                };
            }

            Ok(buf.into_df()?)
        }
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
    pub async fn next_or_summary(&mut self) -> Result<Option<RowItem>> {
        self.stream.next_or_summary(&mut self.connection).await
    }

    /// Stop consuming the stream and return a summary, if available.
    /// Stopping the stream will also discard any messages on the server side.
    pub async fn finish(mut self) -> Result<FinishResult> {
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
    /// every element is a [`RowItem`].
    ///
    /// The stream can only be converted once.
    pub fn as_row_items(&mut self) -> impl TryStream<Ok = RowItem, Error = Error> + '_ {
        self.stream.as_row_items(&mut self.connection)
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
    /// every row is converted into a [`RowItem<T>`] by calling [`crate::row::Row::to`].
    ///
    /// The stream can only be converted once.
    pub fn as_items<'this, T: DeserializeOwned + 'this>(
        &'this mut self,
    ) -> impl TryStream<Ok = RowItem<T>, Error = Error> + 'this {
        self.stream.as_items(&mut self.connection)
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

    /// Turns this RowStream into a [`futures::stream::TryStream`] where
    /// the value at the given column is converted into a [`RowItem<T>`]
    /// by calling [`crate::row::Row::get`].
    ///
    /// The stream can only be converted once.
    pub fn column_to_items<'this, 'db: 'this, T: DeserializeOwned + 'db>(
        &'this mut self,
        column: &'db str,
    ) -> impl TryStream<Ok = RowItem<T>, Error = Error> + 'this {
        self.stream.column_to_items(&mut self.connection, column)
    }

    #[cfg(all(feature = "polars_v0_43", not(feature = "unstable-result-summary")))]
    pub async fn into_dataframe(mut self) -> Result<polars::frame::DataFrame> {
        self.stream.into_dataframe(&mut self.connection).await
    }

    #[cfg(all(feature = "polars_v0_43", feature = "unstable-result-summary"))]
    pub async fn into_dataframe(
        mut self,
    ) -> Result<(polars::frame::DataFrame, Option<ResultSummary>)> {
        self.stream.into_dataframe(&mut self.connection).await
    }
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    Ready,
    Complete(Option<BoxedSummary>),
}

// mod pl {{{
#[cfg(feature = "polars_v0_43")]
mod pl {
    use polars::{
        error::PolarsError as Error,
        frame::DataFrame,
        prelude::{AnyValue, PlSmallStr},
        series::Series,
    };

    use crate::BoltType;

    #[derive(Debug, Clone)]
    pub(super) struct DataBuf {
        fields: Vec<PlSmallStr>,
        buffers: Vec<ColBuf>,
    }

    impl DataBuf {
        pub(super) fn new<S: Into<PlSmallStr>>(fields: impl IntoIterator<Item = S>) -> Self {
            let fields = fields.into_iter().map(Into::into).collect::<Vec<_>>();
            let buffers = vec![ColBuf::default(); fields.len()];
            Self { fields, buffers }
        }

        pub(super) fn push(&mut self, values: Vec<BoltType>) {
            assert_eq!(values.len(), self.fields.len());
            for (buf, value) in self.buffers.iter_mut().zip(values) {
                buf.push(value);
            }
        }

        pub(super) fn flush(&mut self) -> Result<(), Error> {
            for buf in &mut self.buffers {
                buf.flush()?;
            }
            Ok(())
        }

        pub(super) fn into_df(self) -> Result<DataFrame, polars::error::PolarsError> {
            let serieses = self
                .buffers
                .into_iter()
                .zip(self.fields.into_iter())
                .map(|(buf, field)| buf.into_series(field))
                .collect::<Result<Vec<_>, _>>()?;

            DataFrame::new(serieses)
        }
    }

    // TODO: use AnyValueBuffer
    #[derive(Debug, Default, Clone)]
    struct ColBuf {
        values: Vec<AnyValue<'static>>,
        series: Option<Series>,
    }

    impl ColBuf {
        fn push(&mut self, value: BoltType) {
            let value = match value {
                BoltType::String(v) => AnyValue::StringOwned(v.value.into()),
                BoltType::Boolean(v) => AnyValue::Boolean(v.value),
                BoltType::Map(_) => todo!(),
                BoltType::Null(_) => AnyValue::Null,
                BoltType::Integer(v) => AnyValue::Int64(v.value),
                BoltType::Float(v) => AnyValue::Float64(v.value),
                BoltType::List(_) => todo!(),
                BoltType::Node(_) => todo!(),
                BoltType::Relation(_) => todo!(),
                BoltType::UnboundedRelation(_) => todo!(),
                BoltType::Point2D(_) => todo!(),
                BoltType::Point3D(_) => todo!(),
                BoltType::Bytes(v) => AnyValue::BinaryOwned(v.value.into()),
                BoltType::Path(_) => todo!(),
                BoltType::Duration(_) => todo!(),
                BoltType::Date(_) => todo!(),
                BoltType::Time(_) => todo!(),
                BoltType::LocalTime(_) => todo!(),
                BoltType::DateTime(_) => todo!(),
                BoltType::LocalDateTime(_) => todo!(),
                BoltType::DateTimeZoneId(_) => todo!(),
            };
            self.values.push(value);
        }

        fn flush(&mut self) -> Result<(), Error> {
            let chunk = Series::from_any_values(PlSmallStr::EMPTY, &self.values, false)?;
            if let Some(series) = &mut self.series {
                series.append(&chunk)?;
            } else {
                self.series = Some(chunk);
            }

            Ok(())
        }

        fn into_series(mut self, name: PlSmallStr) -> Result<Series, Error> {
            self.flush()?;
            Ok(self.series.unwrap().with_name(name))
        }
    }
}
// }}}
