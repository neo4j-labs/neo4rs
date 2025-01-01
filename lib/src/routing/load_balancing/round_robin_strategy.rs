use crate::routing::load_balancing::LoadBalancingStrategy;
use std::sync::atomic::AtomicUsize;
use crate::routing::connection_registry::BoltServer;

pub struct RoundRobinStrategy {
    readers: Vec<BoltServer>,
    writers: Vec<BoltServer>,
    routers: Vec<BoltServer>,
    reader_index: AtomicUsize,
    writer_index: AtomicUsize,
    router_index: AtomicUsize,
}

impl RoundRobinStrategy {
    pub(crate) fn new(servers: &[BoltServer]) -> Self {
        let readers: Vec<BoltServer> = servers
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect();
        let writers: Vec<BoltServer> = servers
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect();
        let routers: Vec<BoltServer> = servers
            .iter()
            .filter(|s| s.role == "ROUTE")
            .cloned()
            .collect();
        let reader_index = AtomicUsize::new(readers.len());
        let writer_index = AtomicUsize::new(writers.len());
        let router_index = AtomicUsize::new(routers.len());
        RoundRobinStrategy {
            readers,
            writers,
            routers,
            reader_index,
            writer_index,
            router_index,
        }
    }

    fn select(servers: &[BoltServer], index: &AtomicUsize) -> Option<BoltServer> {
        if servers.is_empty() {
            return None;
        }

        index
            .compare_exchange(
                0,
                servers.len(),
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .ok();
        let i = index.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if let Some(server) = servers.get(i - 1) {
            Some(server.clone())
        } else {
            //reset index
            index.store(servers.len(), std::sync::atomic::Ordering::Relaxed);
            servers.last().cloned()
        }
    }
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    fn select_reader(&self) -> Option<BoltServer> {
        Self::select(&self.readers, &self.reader_index)
    }

    fn select_writer(&self) -> Option<BoltServer> {
        Self::select(&self.writers, &self.writer_index)
    }

    fn select_router(&self) -> Option<BoltServer> {
        Self::select(&self.routers, &self.router_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::routing::{RoutingTable, Server};
    use super::*;

    #[test]
    fn should_get_next_server() {
        let readers = vec![
            Server {
                addresses: vec!["localhost:7687".to_string()],
                role: "READ".to_string(),
            },
            Server {
                addresses: vec!["localhost:7688".to_string()],
                role: "READ".to_string(),
            },
        ];
        let writers = vec![];
        let cluster_routing_table = RoutingTable {
            ttl: 0,
            db: None,
            servers: readers.clone().into_iter().chain(writers.clone()).collect(),
        };
        let strategy = RoundRobinStrategy::new(&cluster_routing_table.resolve());

        let reader = strategy
            .select_reader()
            .unwrap();
        assert_eq!(reader.address, readers[1].addresses[0]);
        let reader = strategy
            .select_reader()
            .unwrap();
        assert_eq!(reader.address, readers[0].addresses[0]);
        let reader = strategy
            .select_reader()
            .unwrap();
        assert_eq!(reader.address, readers[1].addresses[0]);
        let writer = strategy.select_writer();
        assert_eq!(writer, None);
    }
}
