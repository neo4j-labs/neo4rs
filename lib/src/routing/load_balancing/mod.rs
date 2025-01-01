pub(crate) mod round_robin_strategy;

use crate::routing::connection_registry::BoltServer;

pub trait LoadBalancingStrategy: Sync + Send {
    fn select_reader(&self) -> Option<BoltServer>;
    fn select_writer(&self) -> Option<BoltServer>;
    fn select_router(&self) -> Option<BoltServer>;
}
