use crate::{
    errors::{unexpected, Result},
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

    pub(crate) async fn run(self, db: &str, connection: &mut ManagedConnection) -> Result<()> {
        let request = BoltRequest::run(db, &self.query, self.params);
        Self::try_run(request, connection)
            .await
            .map_err(unwrap_backoff)
    }

    pub(crate) async fn run_retryable(
        &self,
        db: &str,
        connection: &mut ManagedConnection,
    ) -> Result<(), backoff::Error<Error>> {
        let request = BoltRequest::run(db, &self.query, self.params.clone());
        Self::try_run(request, connection).await
    }

    pub(crate) async fn execute_retryable(
        &self,
        db: &str,
        fetch_size: usize,
        mut connection: ManagedConnection,
    ) -> Result<DetachedRowStream, backoff::Error<Error>> {
        let request = BoltRequest::run(db, &self.query, self.params.clone());
        Self::try_execute(request, fetch_size, &mut connection)
            .await
            .map(|stream| DetachedRowStream::new(stream, connection))
    }

    pub(crate) async fn execute_mut<'conn>(
        self,
        db: &str,
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
        match connection.send_recv(BoltRequest::discard()).await {
            Ok(BoltResponse::Success(_)) => Ok(()),
            otherwise => wrap_error(otherwise, "DISCARD"),
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
            RowStream::new(qid, fields, fetch_size)
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

fn wrap_error<T>(resp: Result<BoltResponse>, req: &str) -> QueryResult<T> {
    let error = match resp {
        Ok(BoltResponse::Failure(failure)) => Error::Neo4j(failure.into_error()),
        Ok(_) => unexpected(resp, req),
        Err(e) => e,
    };
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
