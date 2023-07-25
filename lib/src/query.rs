use crate::config::Config;
use crate::errors::*;
use crate::messages::*;
use crate::pool::*;
use crate::stream::*;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::Mutex;

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
        config: &Config,
        connection: Arc<Mutex<ManagedConnection>>,
    ) -> Result<()> {
        let run = BoltRequest::run(&config.db, &self.query, self.params.clone());
        let mut connection = connection.lock().await;
        match connection.send_recv(run).await? {
            BoltResponse::Success(_) => match connection.send_recv(BoltRequest::discard()).await? {
                BoltResponse::Success(_) => Ok(()),
                msg => Err(unexpected(msg, "DISCARD")),
            },
            msg => Err(unexpected(msg, "RUN")),
        }
    }

    pub(crate) async fn execute(
        self,
        config: &Config,
        connection: Arc<Mutex<ManagedConnection>>,
    ) -> Result<RowStream> {
        let run = BoltRequest::run(&config.db, &self.query, self.params);
        match connection.lock().await.send_recv(run).await {
            Ok(BoltResponse::Success(success)) => {
                let fields: BoltList = success.get("fields").unwrap_or_else(BoltList::new);
                let qid: i64 = success.get("qid").unwrap_or(-1);
                Ok(RowStream::new(
                    qid,
                    fields,
                    config.fetch_size,
                    connection.clone(),
                ))
            }
            msg => Err(unexpected(msg, "RUN")),
        }
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
        assert!(q.has_param_key("country") == false);
    }
}
