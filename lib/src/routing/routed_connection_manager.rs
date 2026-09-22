use crate::config::ImpersonateUser;
use crate::pool::ManagedConnection;
use crate::routing::connection_registry::ConnectionRegistry;
use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::routing_table_provider::RoutingTableProvider;
use crate::routing::types::BoltServer;
use crate::routing::RoundRobinStrategy;
use crate::Database;
#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::{Config, Error, Operation};
use backon::ExponentialBuilder;
use log::{debug, error};
use std::sync::Arc;

#[derive(Clone)]
pub struct RoutedConnectionManager {
    load_balancing_strategy: Arc<dyn LoadBalancingStrategy>,
    connection_registry: Arc<ConnectionRegistry>,
    backoff: ExponentialBuilder,
}

impl RoutedConnectionManager {
    pub fn new(config: &Config, provider: Arc<dyn RoutingTableProvider>) -> Result<Self, Error> {
        let backoff = crate::pool::backoff();
        let connection_registry = Arc::new(ConnectionRegistry::new(config, provider));
        Ok(RoutedConnectionManager {
            load_balancing_strategy: Arc::new(RoundRobinStrategy::new(connection_registry.clone())),
            connection_registry,
            backoff,
        })
    }

    pub(crate) async fn get(
        &self,
        operation: Option<Operation>,
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<ManagedConnection, Error> {
        let op = operation.unwrap_or(Operation::Write);

        // We need to ensure that the router is selected before attempting to get a server pool.
        let router = self.router_pool();

        // We request the list of servers for the selected db.
        // If db is None, we will fetch the default database from the router.
        // If something goes wrong, we will return an empty list of servers: this will cause
        // the connection manager to fail.
        let mut servers = self
            .connection_registry
            .servers(db.clone(), imp_user.clone(), bookmarks, router)
            .await;

        let mut refresh_attempted = false;

        loop {
            let role = match op {
                Operation::Read => "readers",
                Operation::Write => "writers",
            };
            let selected_server = match op {
                Operation::Read => self.select_reader(&servers),
                Operation::Write => self.select_writer(&servers),
            };

            let Some(selected_server) = selected_server else {
                if refresh_attempted {
                    error!("No available {role} in the routing table");
                    return Err(Error::ServerUnavailableError(format!(
                        "No available {role} in the routing table for operation {op}"
                    )));
                }
                refresh_attempted = true;
                debug!("No available {role} in the routing table, refreshing");
                servers = self
                    .connection_registry
                    .refresh_servers(db.clone(), imp_user.clone(), bookmarks, self.router_pool())
                    .await;
                continue;
            };

            if let Some(pool) = self.connection_registry.get_server_pool(&selected_server) {
                match pool.get().await {
                    Ok(conn) => return Ok(conn),
                    Err(e) => {
                        error!("Failed to get connection from pool for server {selected_server:?}: {e}");
                        self.connection_registry.mark_unavailable(&selected_server);
                        servers.retain(|s| !s.has_same_address(&selected_server));
                        continue; // Try selecting another server
                    }
                }
            } else {
                error!("No connection pool found for server: {selected_server:?}");
                return Err(Error::ServerUnavailableError(format!(
                    "No connection pool found for server: {selected_server:?}",
                )));
            }
        }
    }

    pub(crate) async fn get_default_db(
        &self,
        imp_user: Option<ImpersonateUser>,
        bookmarks: &[String],
    ) -> Result<Option<Database>, Error> {
        self.connection_registry
            .get_default_db(imp_user, bookmarks, self.router_pool())
            .await
    }

    pub(crate) fn backoff(&self) -> ExponentialBuilder {
        self.backoff
    }

    fn router_pool(&self) -> Option<crate::pool::ConnectionPool> {
        self.select_router(&self.connection_registry.all_servers())
            .and_then(|router| {
                debug!("Selected router: {router:?}");
                self.connection_registry.get_server_pool(&router)
            })
    }

    fn select_reader(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        self.load_balancing_strategy.select_reader(servers)
    }

    fn select_writer(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        self.load_balancing_strategy.select_writer(servers)
    }

    fn select_router(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        self.load_balancing_strategy.select_router(servers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;
    use crate::packstream::bolt;
    use crate::pool::ConnectionPool;
    use crate::routing::{RoutingTable, Server};
    use crate::Version;
    use std::collections::VecDeque;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::task::JoinHandle;

    const BOLT_V4_4: [u8; 4] = [0, 0, 4, 4];
    const HANDSHAKE_LEN: usize = 20;

    fn make_config() -> Config {
        ConfigBuilder::default()
            .uri("neo4j://127.0.0.1:7687")
            .user("user")
            .password("password")
            .connection_timeout(Duration::from_secs(5))
            .build()
            .unwrap()
    }

    fn localhost(port: u16) -> String {
        format!("127.0.0.1:{port}")
    }

    fn routing_table(
        ttl: u64,
        readers: &[String],
        writers: &[String],
        routers: &[String],
    ) -> RoutingTable {
        let servers = |addresses: &[String], role: &str| Server {
            addresses: addresses.to_vec(),
            role: role.to_string(),
        };
        RoutingTable {
            ttl,
            db: Some("neo4j".into()),
            servers: [
                servers(readers, "READ"),
                servers(writers, "WRITE"),
                servers(routers, "ROUTE"),
            ]
            .into_iter()
            .filter(|s| !s.addresses.is_empty())
            .collect(),
        }
    }

    fn spawn_bolt_hello_server(listener: TcpListener) -> JoinHandle<()> {
        tokio::spawn(async move {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            serve_hello(&mut stream).await;
        })
    }

    async fn serve_hello(stream: &mut TcpStream) {
        let mut handshake = [0; HANDSHAKE_LEN];
        if stream.read_exact(&mut handshake).await.is_err() {
            return;
        }
        if stream.write_all(&BOLT_V4_4).await.is_err() {
            return;
        }
        if read_chunked_message(stream).await.is_err() {
            return;
        }
        let hello_success = bolt()
            .structure(1, 0x70)
            .tiny_map(2)
            .tiny_string("server")
            .tiny_string("Neo4j/4.4.0")
            .tiny_string("connection_id")
            .tiny_string("bolt-42")
            .build();
        if write_chunked_message(stream, &hello_success).await.is_err() {
            return;
        }
        let mut incoming = [0; 1024];
        while let Ok(received) = stream.read(&mut incoming).await {
            if received == 0 {
                break;
            }
        }
    }

    async fn read_chunked_message(stream: &mut TcpStream) -> std::io::Result<()> {
        loop {
            let chunk_size = stream.read_u16().await?;
            if chunk_size == 0 {
                return Ok(());
            }
            let mut chunk = vec![0; chunk_size as usize];
            stream.read_exact(&mut chunk).await?;
        }
    }

    async fn write_chunked_message(stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
        stream.write_u16(payload.len() as u16).await?;
        stream.write_all(payload).await?;
        stream.write_u16(0).await?;
        stream.flush().await
    }

    struct SequentialRoutingTableProvider {
        tables: Mutex<VecDeque<RoutingTable>>,
        fetch_count: AtomicUsize,
        router_used: Mutex<Vec<bool>>,
    }

    impl SequentialRoutingTableProvider {
        fn new(tables: Vec<RoutingTable>) -> Self {
            Self {
                tables: Mutex::new(tables.into()),
                fetch_count: AtomicUsize::new(0),
                router_used: Mutex::new(Vec::new()),
            }
        }

        fn fetch_count(&self) -> usize {
            self.fetch_count.load(Ordering::SeqCst)
        }

        fn router_used(&self) -> Vec<bool> {
            self.router_used.lock().unwrap().clone()
        }
    }

    impl RoutingTableProvider for SequentialRoutingTableProvider {
        fn fetch_routing_table(
            &self,
            _bookmarks: &[String],
            _db: Option<Database>,
            _imp_user: Option<ImpersonateUser>,
            router: Option<ConnectionPool>,
        ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            self.router_used.lock().unwrap().push(router.is_some());
            let mut tables = self.tables.lock().unwrap();
            let table = if tables.len() > 1 {
                tables.pop_front()
            } else {
                tables.front().cloned()
            };
            Box::pin(async move {
                table.ok_or_else(|| Error::RoutingTableRefreshFailed("no routing table".into()))
            })
        }
    }

    #[tokio::test]
    async fn recovers_new_writer_from_router_before_ttl_expiry() {
        let dead_writer = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let new_writer = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let reader = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let router = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let dead_writer_port = dead_writer.local_addr().unwrap().port();
        let new_writer_port = new_writer.local_addr().unwrap().port();
        let reader_port = reader.local_addr().unwrap().port();
        let router_port = router.local_addr().unwrap().port();
        let new_writer_task = spawn_bolt_hello_server(new_writer);
        let dead_writer_task = tokio::spawn(async move {
            while let Ok((stream, _)) = dead_writer.accept().await {
                drop(stream);
            }
        });

        let stale_table = routing_table(
            300,
            &[localhost(reader_port)],
            &[localhost(dead_writer_port)],
            &[localhost(router_port)],
        );
        let refreshed_table = routing_table(
            300,
            &[localhost(reader_port)],
            &[localhost(new_writer_port)],
            &[localhost(router_port)],
        );
        let provider = Arc::new(SequentialRoutingTableProvider::new(vec![
            stale_table,
            refreshed_table,
        ]));
        let manager = RoutedConnectionManager::new(&make_config(), provider.clone()).unwrap();

        let db = Some(Database::from("neo4j"));
        let connection = manager
            .get(Some(Operation::Write), db.clone(), None, &[])
            .await
            .expect("the refreshed table advertises a reachable writer");
        assert_eq!(connection.version(), Version::V4_4);
        assert_eq!(provider.fetch_count(), 2);
        assert_eq!(provider.router_used(), vec![false, true]);

        let cached = manager
            .connection_registry
            .servers(db, None, &[], None)
            .await;
        assert_eq!(provider.fetch_count(), 2);
        assert!(cached
            .iter()
            .any(|s| s.role == "WRITE" && s.port == new_writer_port));
        assert!(!cached.iter().any(|s| s.port == dead_writer_port));

        drop(connection);
        new_writer_task.abort();
        dead_writer_task.abort();
    }

    #[tokio::test]
    async fn falls_back_to_seed_when_no_router_remains() {
        let dead_writer = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let new_writer = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let dead_writer_port = dead_writer.local_addr().unwrap().port();
        let new_writer_port = new_writer.local_addr().unwrap().port();
        let new_writer_task = spawn_bolt_hello_server(new_writer);
        let dead_writer_task = tokio::spawn(async move {
            while let Ok((stream, _)) = dead_writer.accept().await {
                drop(stream);
            }
        });

        let stale_table = routing_table(300, &[], &[localhost(dead_writer_port)], &[]);
        let refreshed_table = routing_table(300, &[], &[localhost(new_writer_port)], &[]);
        let provider = Arc::new(SequentialRoutingTableProvider::new(vec![
            stale_table,
            refreshed_table,
        ]));
        let manager = RoutedConnectionManager::new(&make_config(), provider.clone()).unwrap();

        let db = Some(Database::from("neo4j"));
        let connection = manager
            .get(Some(Operation::Write), db, None, &[])
            .await
            .expect("the refreshed table advertises a reachable writer");
        assert_eq!(connection.version(), Version::V4_4);
        assert_eq!(provider.fetch_count(), 2);
        assert_eq!(provider.router_used(), vec![false, false]);

        drop(connection);
        new_writer_task.abort();
        dead_writer_task.abort();
    }

    #[tokio::test]
    async fn writerless_table_does_not_cause_infinite_refresh() {
        let reader = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let router = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let reader_port = reader.local_addr().unwrap().port();
        let router_port = router.local_addr().unwrap().port();
        let writerless = routing_table(
            300,
            &[localhost(reader_port)],
            &[],
            &[localhost(router_port)],
        );
        let provider = Arc::new(SequentialRoutingTableProvider::new(vec![writerless]));
        let manager = RoutedConnectionManager::new(&make_config(), provider.clone()).unwrap();

        let db = Some(Database::from("neo4j"));
        let result = manager
            .get(Some(Operation::Write), db.clone(), None, &[])
            .await;

        let Err(err) = result else {
            panic!("a writerless table must fail a WRITE operation");
        };
        match err {
            Error::ServerUnavailableError(msg) => {
                assert!(msg.contains("writers"), "unexpected message: {msg}");
            }
            e => panic!("expected ServerUnavailableError, got {e:?}"),
        }
        assert_eq!(provider.fetch_count(), 2);
        assert_eq!(provider.router_used(), vec![false, true]);

        let result = manager.get(Some(Operation::Write), db, None, &[]).await;
        assert!(matches!(result, Err(Error::ServerUnavailableError(_))));
        assert_eq!(provider.fetch_count(), 3);
    }
}
