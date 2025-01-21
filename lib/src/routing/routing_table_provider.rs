use crate::connection::{Connection, ConnectionInfo};
use crate::routing::{RouteBuilder, RoutingTable};
use crate::{Config, Error};
use std::future::Future;
use std::pin::Pin;

pub(crate) trait RoutingTableProvider: Send + Sync {
    fn fetch_routing_table(
        &self,
        config: &Config,
    ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>>;
}

pub struct ClusterRoutingTableProvider;

impl RoutingTableProvider for ClusterRoutingTableProvider {
    fn fetch_routing_table(
        &self,
        config: &Config,
    ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>> {
        let config = config.clone();
        Box::pin(async move {
            let info = ConnectionInfo::new(
                &config.uri,
                &config.user,
                &config.password,
                &config.tls_config,
            )?;
            let mut connection = Connection::new(&info).await?;
            let mut builder = RouteBuilder::new(info.routing, vec![]);
            if let Some(db) = config.db.clone() {
                builder = builder.with_db(db);
            }
            connection.route(builder.build(connection.version())).await
        })
    }
}