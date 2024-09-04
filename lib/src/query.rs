#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::bolt::{Discard, Summary, WrapExtra as _};
use crate::{
    errors::Result,
    messages::{BoltRequest, BoltResponse},
    pool::ManagedConnection,
    stream::{DetachedRowStream, RowStream},
    types::{BoltList, BoltMap, BoltString, BoltType},
    Error, Success,
};

/// Abstracts a cypher query that is sent to neo4j server.
#[derive(Clone)]
pub struct Query {
    query: String,
    params: BoltMap,
}

impl Query {
    pub fn new(query: String) -> Self {
        Query {
            query,
            params: BoltMap::default(),
        }
    }

    pub fn param<T: Into<BoltType>>(mut self, key: &str, value: T) -> Self {
        self.params.put(key.into(), value.into());
        self
    }

    pub fn params<K, V>(mut self, input_params: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<BoltString>,
        V: Into<BoltType>,
    {
        for (key, value) in input_params {
            self.params.put(key.into(), value.into());
        }

        self
    }

    pub fn has_param_key(&self, key: &str) -> bool {
        self.params.value.contains_key(key)
    }

    pub(crate) async fn run(
        self,
        db: Option<&str>,
        connection: &mut ManagedConnection,
    ) -> Result<()> {
        let request = BoltRequest::run(db, &self.query, self.params);
        Self::try_run(request, connection)
            .await
            .map_err(unwrap_backoff)
    }

    pub(crate) async fn run_retryable(
        &self,
        db: Option<&str>,
        connection: &mut ManagedConnection,
    ) -> QueryResult<()> {
        let request = BoltRequest::run(db, &self.query, self.params.clone());
        Self::try_run(request, connection).await
    }

    pub(crate) async fn execute_retryable(
        &self,
        db: Option<&str>,
        fetch_size: usize,
        mut connection: ManagedConnection,
    ) -> QueryResult<DetachedRowStream> {
        let request = BoltRequest::run(db, &self.query, self.params.clone());
        Self::try_execute(request, fetch_size, &mut connection)
            .await
            .map(|stream| DetachedRowStream::new(stream, connection))
    }

    pub(crate) async fn execute_mut<'conn>(
        self,
        db: Option<&str>,
        fetch_size: usize,
        connection: &'conn mut ManagedConnection,
    ) -> Result<RowStream> {
        let run = BoltRequest::run(db, &self.query, self.params);
        Self::try_execute(run, fetch_size, connection)
            .await
            .map_err(unwrap_backoff)
    }

    async fn try_run(request: BoltRequest, connection: &mut ManagedConnection) -> QueryResult<()> {
        let _ = Self::try_request(request, connection).await?;

        #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
        {
            match connection.send_recv(BoltRequest::discard_all()).await {
                Ok(BoltResponse::Success(_)) => Ok(()),
                otherwise => wrap_error(otherwise, "DISCARD"),
            }
        }

        #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
        {
            match connection.send_recv_as(Discard::all()).await {
                Ok(Summary::Success(_discard_success)) => Ok(()),
                otherwise => wrap_error(otherwise, "DISCARD"),
            }
        }
    }

    async fn try_execute(
        request: BoltRequest,
        fetch_size: usize,
        connection: &mut ManagedConnection,
    ) -> QueryResult<RowStream> {
        Self::try_request(request, connection).await.map(|success| {
            let fields: BoltList = success.get("fields").unwrap_or_default();
            let qid: i64 = success.get("qid").unwrap_or(-1);

            #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
            {
                let available: i64 = success.get("t_first").unwrap_or(-1);
                RowStream::new(qid, available, fields, fetch_size)
            }

            #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
            {
                RowStream::new(qid, fields, fetch_size)
            }
        })
    }

    async fn try_request(
        request: BoltRequest,
        connection: &mut ManagedConnection,
    ) -> QueryResult<Success> {
        match connection.send_recv(request).await {
            Ok(BoltResponse::Success(success)) => Ok(success),
            otherwise => wrap_error(otherwise, "RUN"),
        }
    }
}

impl From<String> for Query {
    fn from(query: String) -> Self {
        Query::new(query)
    }
}

impl From<&str> for Query {
    fn from(query: &str) -> Self {
        Query::new(query.to_owned())
    }
}

type QueryResult<T> = Result<T, backoff::Error<Error>>;

fn wrap_error<T>(resp: impl IntoError, req: &'static str) -> QueryResult<T> {
    let error = resp.into_error(req);
    let can_retry = match &error {
        Error::Neo4j(e) => e.can_retry(),
        _ => false,
    };

    if can_retry {
        Err(backoff::Error::transient(error))
    } else {
        Err(backoff::Error::permanent(error))
    }
}

trait IntoError {
    fn into_error(self, msg: &'static str) -> Error;
}

impl IntoError for Result<BoltResponse> {
    fn into_error(self, msg: &'static str) -> Error {
        match self {
            Ok(BoltResponse::Failure(failure)) => Error::Neo4j(failure.into_error()),
            Ok(resp) => resp.into_error(msg),
            Err(e) => e,
        }
    }
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
impl<R: std::fmt::Debug> IntoError for Result<Summary<R>> {
    fn into_error(self, msg: &'static str) -> Error {
        match self {
            Ok(resp) => resp.into_error(msg),
            Err(e) => e,
        }
    }
}

fn unwrap_backoff(err: backoff::Error<Error>) -> Error {
    match err {
        backoff::Error::Permanent(e) => e,
        backoff::Error::Transient { err, .. } => err,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_params() {
        let q = Query::new("MATCH (n) WHERE n.name = $name AND n.age > $age RETURN n".to_owned());
        let q = q.params([
            ("name", BoltType::from("Frobniscante")),
            ("age", BoltType::from(42)),
        ]);

        assert_eq!(
            q.params.get::<String>("name").unwrap(),
            String::from("Frobniscante")
        );
        assert_eq!(q.params.get::<i64>("age").unwrap(), 42);

        assert!(q.has_param_key("name"));
        assert!(!q.has_param_key("country"));
    }
}
