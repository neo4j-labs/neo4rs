use crate::routing::load_balancing::LoadBalancingStrategy;
use crate::routing::{RoutingTable, Server};
use std::sync::atomic::AtomicUsize;

pub struct RoundRobinStrategy {
    reader_index: AtomicUsize,
    writer_index: AtomicUsize,
    router_index: AtomicUsize,
}

impl RoundRobinStrategy {
    pub(crate) fn new(cluster_routing_table: RoutingTable) -> Self {
        let readers: Vec<Server> = cluster_routing_table
            .servers
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect();
        let writers: Vec<Server> = cluster_routing_table
            .servers
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect();
        let routers: Vec<Server> = cluster_routing_table
            .servers
            .iter()
            .filter(|s| s.role == "ROUTE")
            .cloned()
            .collect();
        let reader_index = AtomicUsize::new(readers.len());
        let writer_index = AtomicUsize::new(writers.len());
        let router_index = AtomicUsize::new(routers.len());
        RoundRobinStrategy {
            reader_index,
            writer_index,
            router_index,
        }
    }

    fn select(servers: &[Server], index: &AtomicUsize) -> Option<Server> {
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
    fn select_reader(&self, servers: &[Server]) -> Option<Server> {
        let readers = servers
            .iter()
            .filter(|s| s.role == "READ")
            .cloned()
            .collect::<Vec<Server>>();

        Self::select(readers.as_slice(), &self.reader_index)
    }

    fn select_writer(&self, servers: &[Server]) -> Option<Server> {
        let writers = servers
            .iter()
            .filter(|s| s.role == "WRITE")
            .cloned()
            .collect::<Vec<Server>>();

        Self::select(writers.as_slice(), &self.writer_index)
    }

    fn select_router(&self, servers: &[Server]) -> Option<Server> {
        let routers = servers
            .iter()
            .filter(|s| s.role == "ROUTE")
            .cloned()
            .collect::<Vec<Server>>();

        Self::select(routers.as_slice(), &self.router_index)
    }
}

#[cfg(test)]
mod tests {
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
        let strategy = RoundRobinStrategy::new(cluster_routing_table.clone());
        let reader = strategy
            .select_reader(cluster_routing_table.servers.as_slice())
            .unwrap();
        assert_eq!(reader, readers[1]);
        let reader = strategy
            .select_reader(cluster_routing_table.servers.as_slice())
            .unwrap();
        assert_eq!(reader, readers[0]);
        let reader = strategy
            .select_reader(cluster_routing_table.servers.as_slice())
            .unwrap();
        assert_eq!(reader, readers[1]);
        let writer = strategy.select_writer(cluster_routing_table.servers.as_slice());
        assert_eq!(writer, None);
    }
}
