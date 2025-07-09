use crate::routing::connection_registry::{BoltServer, ConnectionRegistry};
use crate::routing::load_balancing::LoadBalancingStrategy;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct RoundRobinStrategy {
    connection_registry: Arc<ConnectionRegistry>,
    reader_index: AtomicUsize,
    writer_index: AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new(connection_registry: Arc<ConnectionRegistry>) -> Self {
        RoundRobinStrategy {
            connection_registry,
            reader_index: AtomicUsize::new(0),
            writer_index: AtomicUsize::new(0),
        }
    }

    fn select(
        all_servers: &[BoltServer],
        servers: &[BoltServer],
        index: &AtomicUsize,
    ) -> Option<BoltServer> {
        if servers.is_empty() {
            return None;
        }

        let mut used = vec![];
        loop {
            if used.len() >= all_servers.len() {
                return None; // All servers have been used
            }
            let _ =
                index.compare_exchange(0, all_servers.len(), Ordering::Relaxed, Ordering::Relaxed);
            let i = index.fetch_sub(1, Ordering::Relaxed);
            if let Some(server) = all_servers.get(i - 1) {
                if servers.contains(server) {
                    return Some(server.clone());
                }
                used.push(server.clone());
            }
        }
    }
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    fn select_reader(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        let readers = servers
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect::<Vec<BoltServer>>();
        let mut all_readers = self
            .connection_registry
            .all_servers()
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect::<Vec<BoltServer>>();

        // Sort all writers by address to ensure consistent ordering
        all_readers.sort_by(|a, b| a.address.cmp(&b.address));
        Self::select(&all_readers, &readers, &self.reader_index)
    }

    fn select_writer(&self, servers: &[BoltServer]) -> Option<BoltServer> {
        let writers = servers
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect::<Vec<BoltServer>>();
        let mut all_writers = self
            .connection_registry
            .all_servers()
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect::<Vec<BoltServer>>();

        // Sort all writers by address to ensure consistent ordering
        all_writers.sort_by(|a, b| a.address.cmp(&b.address));
        Self::select(&all_writers, &writers, &self.writer_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::{RoutingTable, Server};

    #[test]
    fn should_get_next_server() {
        let routers = vec![Server {
            addresses: vec!["server1:7687".to_string()],
            role: "ROUTE".to_string(),
        }];
        let readers1 = vec![
            Server {
                addresses: vec!["server1:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["server2:7687".to_string()],
                role: "READ".to_string(),
            },
        ];
        let writers1 = vec![Server {
            addresses: vec!["server4:7687".to_string()],
            role: "WRITE".to_string(),
        }];
        let readers2 = vec![
            Server {
                addresses: vec!["server1:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["server3:7687".to_string()],
                role: "READ".to_string(),
            },
        ];

        let writers2 = vec![Server {
            addresses: vec!["server4:7687".to_string()],
            role: "WRITE".to_string(),
        }];

        let routing_table_1 = RoutingTable {
            ttl: 300,
            db: Some("db-1".into()),
            servers: routers
                .clone()
                .into_iter()
                .chain(readers1.clone())
                .chain(writers1.clone())
                .collect(),
        };
        let routing_table_2 = RoutingTable {
            ttl: 300,
            db: Some("db-2".into()),
            servers: routers
                .clone()
                .into_iter()
                .chain(readers2.clone())
                .chain(writers2.clone())
                .collect(),
        };

        let registry = Arc::new(ConnectionRegistry::default());

        let mut servers1 = routing_table_1.resolve();
        servers1.retain(|s| s.role == "READ");
        let mut servers2 = routing_table_2.resolve();
        servers2.retain(|s| s.role == "READ");

        let mut all_readers: Vec<BoltServer> = Vec::new();
        for s in servers1.iter() {
            if !all_readers.iter().any(|x| x == s) {
                all_readers.push(s.clone());
            }
        }
        for s in servers2.iter() {
            if !all_readers.iter().any(|x| x == s) {
                all_readers.push(s.clone());
            }
        }
        all_readers.retain(|s| s.role == "READ");

        assert_eq!(all_readers.len(), 3);
        let strategy = RoundRobinStrategy::new(registry.clone());

        // select a reader for db-1
        let reader =
            RoundRobinStrategy::select(&all_readers, &servers1, &strategy.reader_index).unwrap();
        assert_eq!(reader.address, "server2");
        // select a reader for db-2
        let reader =
            RoundRobinStrategy::select(&all_readers, &servers2, &strategy.reader_index).unwrap();
        assert_eq!(reader.address, "server1");
        // select another reader for db-1
        let reader =
            RoundRobinStrategy::select(&all_readers, &servers1, &strategy.reader_index).unwrap();
        assert_eq!(reader.address, "server2");
    }
}
