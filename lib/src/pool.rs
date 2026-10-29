use std::time::Duration;

use crate::auth::ConnectionTLSConfig;
use crate::{
    config::Config,
    connection::{Connection, ConnectionInfo},
    errors::{Error, Result},
};
use backon::ExponentialBuilder;
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use log::{info, trace};
use std::ops::{Deref, DerefMut};

pub type ConnectionPool = Pool<ConnectionManager>;
pub type ManagedConnection = Object<ConnectionManager>;

pub(crate) struct DiscardOnAbandon<M: Manager> {
    repr: Repr<M>,
}

enum Repr<M: Manager> {
    Open(Object<M>),
    Closed(Object<M>),
    Dropped,
}

impl<M: Manager> DiscardOnAbandon<M> {
    pub(crate) fn new(connection: Object<M>) -> Self {
        Self {
            repr: Repr::Open(connection),
        }
    }

    pub(crate) fn close_transaction(&mut self) {
        let repr = std::mem::replace(&mut self.repr, Repr::Dropped);
        self.repr = match repr {
            Repr::Open(connection) => Repr::Closed(connection),
            repr => repr,
        };
    }
}

impl<M: Manager> Deref for DiscardOnAbandon<M> {
    type Target = Object<M>;

    fn deref(&self) -> &Self::Target {
        match &self.repr {
            Repr::Open(connection) | Repr::Closed(connection) => connection,
            Repr::Dropped => panic!("connection is taken while dropping"),
        }
    }
}

impl<M: Manager> DerefMut for DiscardOnAbandon<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.repr {
            Repr::Open(connection) | Repr::Closed(connection) => connection,
            Repr::Dropped => panic!("connection is taken while dropping"),
        }
    }
}

impl<M: Manager> Drop for DiscardOnAbandon<M> {
    fn drop(&mut self) {
        let repr = std::mem::replace(&mut self.repr, Repr::Dropped);
        if let Repr::Open(connection) = repr {
            drop(Object::take(connection));
        }
    }
}

pub struct ConnectionManager {
    info: ConnectionInfo,
    backoff: ExponentialBuilder,
}

impl ConnectionManager {
    pub fn new(
        uri: &str,
        user: &str,
        password: &str,
        tls_config: &ConnectionTLSConfig,
        connection_timeout: Duration,
        tcp_keepalive: Option<Duration>,
    ) -> Result<Self> {
        let info = ConnectionInfo::new(
            uri,
            user,
            password,
            tls_config,
            connection_timeout,
            tcp_keepalive,
        )?;
        let backoff = backoff();
        Ok(ConnectionManager { info, backoff })
    }

    pub fn backoff(&self) -> ExponentialBuilder {
        self.backoff
    }
}

pub(crate) fn backoff() -> ExponentialBuilder {
    ExponentialBuilder::new()
        .with_jitter()
        .with_factor(2.0)
        .without_max_times()
        .with_min_delay(Duration::from_millis(1))
        .with_max_delay(Duration::from_secs(10))
        .with_total_delay(Some(Duration::from_secs(60)))
}

impl Manager for ConnectionManager {
    type Type = Connection;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        trace!("creating new connection");
        Connection::new(&self.info).await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        trace!("recycling connection");
        // A stream dropped before it was consumed leaves its pipelined PULL in flight.
        // Its responses are drained here, so the connection can be reset and reused.
        // A connection with an interrupted read fails the drain and is discarded.
        let health_check = async {
            obj.drain_pending_responses().await?;
            obj.reset().await
        };
        match tokio::time::timeout(Duration::from_secs(5), health_check).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(deadpool::managed::RecycleError::Backend(e)),
            Err(_) => Err(deadpool::managed::RecycleError::message(
                "Connection health check timed out",
            )),
        }
    }
}

pub fn create_pool(config: &Config) -> Result<ConnectionPool> {
    let mgr = ConnectionManager::new(
        &config.uri,
        &config.user,
        &config.password,
        &config.tls_config,
        config.connection_timeout,
        config.tcp_keepalive,
    )?;
    // Log the parsed host and port, not the URI: a URI can carry credentials.
    info!(
        "creating connection pool for node {}:{} with max size {}",
        mgr.info.prepare.host, mgr.info.prepare.port, config.max_connections
    );
    let mut builder = ConnectionPool::builder(mgr).max_size(config.max_connections);

    // Wire idle_timeout as the recycle timeout — connections idle longer than this
    // will fail the recycle check, causing deadpool to discard and recreate them.
    if let Some(idle_timeout) = config.idle_timeout {
        builder = builder
            .recycle_timeout(Some(idle_timeout))
            .runtime(deadpool::Runtime::Tokio1);
    }

    // Wire max_lifetime as the wait timeout — puts an upper bound on how long
    // a caller will wait for a connection from the pool.
    if let Some(max_lifetime) = config.max_lifetime {
        builder = builder
            .wait_timeout(Some(max_lifetime))
            .runtime(deadpool::Runtime::Tokio1);
    }

    Ok(builder.build().expect("Pool build failed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;
    use std::{
        convert::Infallible,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    struct TestManager {
        created: Arc<AtomicUsize>,
        dropped: Arc<AtomicUsize>,
    }

    struct TestConnection(Arc<AtomicUsize>);

    impl Drop for TestConnection {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl Manager for TestManager {
        type Type = TestConnection;
        type Error = Infallible;

        async fn create(&self) -> Result<Self::Type, Self::Error> {
            self.created.fetch_add(1, Ordering::SeqCst);
            Ok(TestConnection(self.dropped.clone()))
        }

        async fn recycle(&self, _: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
            Ok(())
        }
    }

    fn test_manager() -> (TestManager, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let created = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicUsize::new(0));
        (
            TestManager {
                created: created.clone(),
                dropped: dropped.clone(),
            },
            created,
            dropped,
        )
    }

    #[test]
    fn should_create_pool_with_default_config() {
        let config = ConfigBuilder::default()
            .uri("bolt://127.0.0.1:7687")
            .user("neo4j")
            .password("test")
            .build()
            .unwrap();
        let pool = create_pool(&config);
        assert!(pool.is_ok());
    }

    #[test]
    fn should_create_pool_with_idle_and_max_lifetime() {
        let config = ConfigBuilder::default()
            .uri("bolt://127.0.0.1:7687")
            .user("neo4j")
            .password("test")
            .idle_timeout(Some(Duration::from_secs(300)))
            .max_lifetime(Some(Duration::from_secs(3600)))
            .build()
            .unwrap();
        let pool = create_pool(&config);
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn abandoned_transaction_discards_connection_immediately() {
        let (manager, created, dropped) = test_manager();
        let pool = Pool::builder(manager).max_size(1).build().unwrap();
        let connection = pool.get().await.unwrap();
        let guarded = DiscardOnAbandon::new(connection);

        drop(guarded);

        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        assert_eq!(pool.status().size, 0);

        let _connection = pool.get().await.unwrap();
        assert_eq!(created.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn completed_transaction_returns_connection_to_pool() {
        let (manager, created, dropped) = test_manager();
        let pool = Pool::builder(manager).max_size(1).build().unwrap();
        let connection = pool.get().await.unwrap();
        let mut guarded = DiscardOnAbandon::new(connection);
        guarded.close_transaction();

        drop(guarded);

        assert_eq!(dropped.load(Ordering::SeqCst), 0);
        let _connection = pool.get().await.unwrap();
        assert_eq!(created.load(Ordering::SeqCst), 1);
    }
}
