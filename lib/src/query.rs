use crate::{
    errors::Result,
    messages::{BoltRequest, BoltResponse},
    pool::ManagedConnection,
    stream::{DetachedRowStream, RowStream},
    types::{BoltList, BoltMap, BoltString, BoltType},
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
        let run = BoltRequest::run(db, &self.query, self.params);
        match connection.send_recv(run).await? {
            BoltResponse::Success(_) => match connection.send_recv(BoltRequest::discard()).await? {
                BoltResponse::Success(_) => Ok(()),
                otherwise => Err(otherwise.into_error("DISCARD")),
            },
            msg => Err(msg.into_error("RUN")),
        }
    }

    pub(crate) async fn execute(
        self,
        db: &str,
        fetch_size: usize,
        mut connection: ManagedConnection,
    ) -> Result<DetachedRowStream> {
        let stream = self.execute_mut(db, fetch_size, &mut connection).await?;
        Ok(DetachedRowStream::new(stream, connection))
    }

    pub(crate) async fn execute_mut<'conn>(
        self,
        db: &str,
        fetch_size: usize,
        connection: &'conn mut ManagedConnection,
    ) -> Result<RowStream> {
        let run = BoltRequest::run(db, &self.query, self.params);
        match connection.send_recv(run).await {
            Ok(BoltResponse::Success(success)) => {
                let fields: BoltList = success.get("fields").unwrap_or_default();
                let qid: i64 = success.get("qid").unwrap_or(-1);
                Ok(RowStream::new(qid, fields, fetch_size))
            }
            Ok(msg) => Err(msg.into_error("RUN")),
            Err(e) => Err(e),
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
