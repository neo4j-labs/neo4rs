use std::cell::{Cell, RefCell};

use crate::config::ImpersonateUser;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{bolt::Summary, summary::ResultSummary};
use crate::{
    errors::Result,
    graph::ConnectionPoolManager,
    messages::{BoltRequest, BoltResponse},
    pool::ManagedConnection,
    retry::Retry,
    stream::{DetachedRowStream, RowStream},
    types::{BoltList, BoltMap, BoltString, BoltType},
    Database, Error, Operation, Success,
};

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
pub type RunResult = ResultSummary;
#[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
pub type RunResult = ();

/// Abstracts a cypher query that is sent to neo4j server.
#[derive(Clone)]
pub struct Query {
    query: String,
    params: BoltMap,
    extra: BoltMap,
}

impl Query {
    pub fn new(query: String) -> Self {
        Query {
            query,
            params: BoltMap::default(),
            extra: BoltMap::default(),
        }
    }

    pub fn with_params(mut self, params: BoltMap) -> Self {
        self.params = params;
        self
    }

    pub fn param<T: Into<BoltType>>(mut self, key: &str, value: T) -> Self {
        self.params.put(key.into(), value.into());
        self
    }

    pub fn extra<T: Into<BoltType>>(mut self, key: &str, value: T) -> Self {
        self.extra.put(key.into(), value.into());
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

    pub fn extras<K, V>(mut self, input_params: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<BoltString>,
        V: Into<BoltType>,
    {
        for (key, value) in input_params {
            self.extra.put(key.into(), value.into());
        }

        self
    }

    pub fn imp_user(self, imp_user: Option<ImpersonateUser>) -> Self {
        if let Some(imp_user) = imp_user {
            self.extra("imp_user", imp_user.as_ref().to_string())
        } else {
            self
        }
    }

    pub fn has_param_key(&self, key: &str) -> bool {
        self.params.value.contains_key(key)
    }

    pub fn has_extra_key(&self, key: &str) -> bool {
        self.extra.value.contains_key(key)
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn get_params(&self) -> &BoltMap {
        &self.params
    }

    pub(crate) async fn run(self, connection: &mut ManagedConnection) -> Result<RunResult> {
        let request = BoltRequest::run(&self.query, self.params, self.extra);
        Self::try_run(request, connection)
            .await
            .map_err(Retry::into_inner)
    }

    pub(crate) fn into_retryable<'a>(
        self,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        operation: Operation,
        pool: &'a ConnectionPoolManager,
        fetch_size: Option<usize>,
        bookmarks: &'a [String],
    ) -> RetryableQuery<'a> {
        let query = match db.as_deref() {
            Some(db) => self.extra("db", db),
            None => self,
        };

        let query = if let Some(imp_user) = imp_user.as_deref() {
            query.extra("imp_user", imp_user)
        } else {
            query
        };

        let is_read = operation.is_read();
        let query = query.extra("mode", if is_read { "r" } else { "w" });

        RetryableQuery {
            pool,
            query,
            operation,
            fetch_size,
            db,
            imp_user,
            bookmarks: bookmarks.to_vec(),
        }
    }

    pub(crate) async fn run_retryable(
        &self,
        connection: &mut ManagedConnection,
    ) -> QueryResult<RunResult> {
        let request = BoltRequest::run(&self.query, self.params.clone(), self.extra.clone());
        Self::try_run(request, connection).await
    }

    pub(crate) async fn execute_retryable(
        &self,
        fetch_size: usize,
        mut connection: ManagedConnection,
    ) -> QueryResult<DetachedRowStream> {
        let request = BoltRequest::run(&self.query, self.params.clone(), self.extra.clone());
        Self::try_execute(request, fetch_size, &mut connection)
            .await
            .map(|stream| DetachedRowStream::new(stream, connection))
    }

    pub(crate) async fn execute_mut(
        self,
        fetch_size: usize,
        connection: &mut ManagedConnection,
    ) -> Result<RowStream> {
        let run = BoltRequest::run(&self.query, self.params, self.extra);
        Self::try_execute(run, fetch_size, connection)
            .await
            .map_err(Retry::into_inner)
    }

    async fn try_run(
        request: BoltRequest,
        connection: &mut ManagedConnection,
    ) -> QueryResult<RunResult> {
        let result = Self::try_execute(request, 4096, connection).await?;
        Ok(result.finish(connection).await?)
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

impl std::fmt::Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query")
            .field("query", &self.query)
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

pub(crate) type QueryResult<T> = Result<T, Retry<Error>>;

fn wrap_error<T>(resp: impl IntoError, req: &'static str) -> QueryResult<T> {
    let error = resp.into_error(req);
    let can_retry = match &error {
        Error::Neo4j(e) => e.can_retry(),
        _ => false,
    };

    if can_retry {
        Err(Retry::yes(error))
    } else {
        Err(Retry::no(error))
    }
}

pub(crate) struct RetryableQuery<'a> {
    pool: &'a ConnectionPoolManager,
    query: Query,
    operation: Operation,
    fetch_size: Option<usize>,
    db: Option<Database>,
    imp_user: Option<ImpersonateUser>,
    bookmarks: Vec<String>,
}

impl<'a> RetryableQuery<'a> {
    pub(crate) async fn retry_run(self) -> (Self, QueryResult<RunResult>) {
        let result = self.run().await;
        (self, result)
    }

    async fn run(&self) -> QueryResult<RunResult> {
        let mut connection = self.connect().await?;
        self.query.run_retryable(&mut connection).await
    }

    pub(crate) async fn retry_execute(self) -> (Self, QueryResult<DetachedRowStream>) {
        let result = self.execute().await;
        (self, result)
    }

    async fn execute(&self) -> QueryResult<DetachedRowStream> {
        debug_assert!(
            self.fetch_size.is_some(),
            "Calling execute requires a fetch_size"
        );

        let connection = self.connect().await?;
        self.query
            .execute_retryable(self.fetch_size.expect("fetch_size must be set"), connection)
            .await
    }

    async fn connect(&self) -> QueryResult<ManagedConnection> {
        // an error when retrieving a connection is considered permanent
        self.pool
            .get(
                Some(self.operation),
                self.db.clone(),
                self.imp_user.clone(),
                &self.bookmarks,
            )
            .await
            .map_err(Retry::No)
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

#[doc(hidden)]
pub struct QueryParameter<'x, T> {
    value: Cell<Option<T>>,
    name: &'static str,
    params: &'x RefCell<BoltMap>,
}

impl<'x, T: Into<BoltType>> QueryParameter<'x, T> {
    #[allow(dead_code)]
    pub fn new(value: T, name: &'static str, params: &'x RefCell<BoltMap>) -> Self {
        Self {
            value: Cell::new(Some(value)),
            name,
            params,
        }
    }
}

impl<T: Into<BoltType>> std::fmt::Display for QueryParameter<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(v) = self.value.replace(None) else {
            return Err(std::fmt::Error);
        };
        self.params.borrow_mut().put(self.name.into(), v.into());
        write!(f, "${}", self.name)
    }
}

impl<T: Into<BoltType>> std::fmt::Debug for QueryParameter<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Create a query with a format! like syntax
///
/// `query!` works similar to `format!`:
///   - The first argument is the query string with `{<name>}` placeholders
///   - Following that is a list of `name = value` parameters arguments
///   - All placeholders in the query strings are replaced with query parameters
///
/// The macro is a compiler-supported alternative to using the `params` method on `Query`.
///
/// ## Differences from `format!` and limitations
///
/// - Implicit `{name}` bindings without adding a `name = <value>` argument does not
///   actually create a new parameter; It does default string interpolation instead.
/// - Formatting parameters are largely ignored and have no effect on the query string.
/// - Argument values need to implement `Into<BoltType>` instead of `Display`
///   (and don't need to implement the latter)
/// - Only named placeholders syntax is supported (`{<name>}` instead of `{}`)
///     - This is because query parameters are always named
///     - By extension, adding an unnamed argument (e.g. `<value>` instead of `name = <value>`) is also not supported
///
/// # Examples
///
/// ```
/// use neo4rs::{query, Query};
///
/// // This creates an unparametrized query.
/// let q: Query = query!("MATCH (n) RETURN n");
/// assert_eq!(q.query(), "MATCH (n) RETURN n");
/// assert!(q.get_params().is_empty());
///
/// // This creates a parametrized query.
/// let q: Query = query!("MATCH (n) WHERE n.value = {answer} RETURN n", answer = 42);
/// assert_eq!(q.query(), "MATCH (n) WHERE n.value = $answer RETURN n");
/// assert_eq!(q.get_params().get::<i64>("answer").unwrap(), 42);
///
/// // by contrast, using the implicit string interpolation syntax does not
/// // create a parameter, effectively being the same as `format!`.
/// let answer = 42;
/// let q: Query = query!("MATCH (n) WHERE n.value = {answer} RETURN n");
/// assert_eq!(q.query(), "MATCH (n) WHERE n.value = 42 RETURN n");
/// assert!(q.has_param_key("answer") == false);
///
/// // The value can be any type that implements Into<BoltType>, it does not
/// // need to implement Display or Debug.
/// use neo4rs::{BoltInteger, BoltType};
///
/// struct Answer;
/// impl Into<BoltType> for Answer {
///     fn into(self) -> BoltType {
///         BoltType::Integer(BoltInteger::new(42))
///     }
/// }
///
/// let q: Query = query!("MATCH (n) WHERE n.value = {answer} RETURN n", answer = Answer);
/// assert_eq!(q.query(), "MATCH (n) WHERE n.value = $answer RETURN n");
/// assert_eq!(q.get_params().get::<i64>("answer").unwrap(), 42);
/// ```
#[macro_export]
macro_rules! query {
    // Create a unparametrized query
    ($query:expr) => {
        $crate::Query::new(format!($query))
    };

    // Create a parametrized query with a format! like syntax
    ($query:expr $(, $($input:tt)*)?) => {
        $crate::query!(@internal $query, [] $(; $($input)*)?)
    };

    (@internal $query:expr, [$($acc:tt)*]; $name:ident = $value:expr $(, $($rest:tt)*)?) => {
        $crate::query!(@internal $query, [$($acc)* ($name = $value)] $(; $($rest)*)?)
    };

    (@internal $query:expr, [$($acc:tt)*]; $value:expr $(, $($rest:tt)*)?) => {
        compile_error!("Only named parameter syntax (`name = value`) is supported");
    };

    (@internal $query:expr, [$($acc:tt)*];) => {
        $crate::query!(@final $query; $($acc)*)
    };

    (@internal $query:expr, [$($acc:tt)*]) => {
        $crate::query!(@final $query; $($acc)*)
    };

    (@final $query:expr; $(($name:ident = $value:expr))*) => {{
        let params = $crate::BoltMap::default();
        let params = ::std::cell::RefCell::new(params);

        let query = format!($query, $(
            $name = $crate::QueryParameter::new(
                $value,
                stringify!($name),
                &params,
            ),
        )*);
        let params = params.into_inner();

        $crate::Query::new(query).with_params(params)
    }};
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

    #[test]
    fn query_macro() {
        let q = query!(
            "MATCH (n) WHERE n.name = {name} AND n.age > {age} RETURN n",
            age = 42,
            name = "Frobniscante",
        );

        assert_eq!(
            q.query.as_str(),
            "MATCH (n) WHERE n.name = $name AND n.age > $age RETURN n"
        );

        assert_eq!(
            q.params.get::<String>("name").unwrap(),
            String::from("Frobniscante")
        );
        assert_eq!(q.params.get::<i64>("age").unwrap(), 42);

        assert!(q.has_param_key("name"));
        assert!(!q.has_param_key("country"));
    }
}
