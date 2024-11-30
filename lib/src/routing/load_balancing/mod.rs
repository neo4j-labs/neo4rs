pub(crate) mod round_robin_strategy;

use crate::routing::Server;

pub trait LoadBalancingStrategy: Sync + Send {
    fn select_reader(&self, servers: &[Server]) -> Option<Server>;
    fn select_writer(&self, servers: &[Server]) -> Option<Server>;
    fn select_router(&self, servers: &[Server]) -> Option<Server>;
}
