pub(crate) mod round_robin_strategy;

use crate::routing::connection_registry::BoltServer;

pub trait LoadBalancingStrategy: Sync + Send {
    fn select_reader(&self, servers: &[BoltServer]) -> Option<BoltServer>;
    fn select_writer(&self, servers: &[BoltServer]) -> Option<BoltServer>;
}
