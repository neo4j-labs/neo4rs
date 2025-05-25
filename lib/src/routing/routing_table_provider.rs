use crate::connection::{Connection, ConnectionInfo};
use crate::routing::{RouteBuilder, RoutingTable};
use crate::{Config, Database, Error};
use std::future::Future;
use std::pin::Pin;

pub(crate) trait RoutingTableProvider: Send + Sync {
    fn fetch_routing_table(
        &self,
        config: &Config,
        bookmarks: &[String],
        db: Option<Database>,
    ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>>;
}

pub struct ClusterRoutingTableProvider;

impl RoutingTableProvider for ClusterRoutingTableProvider {
    fn fetch_routing_table(
        &self,
        config: &Config,
        bookmarks: &[String],
        db: Option<Database>,
    ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>> {
        let config = config.clone();
        let bookmarks = bookmarks.to_vec();
        Box::pin(async move {
            let info = ConnectionInfo::new(
                &config.uri,
                &config.user,
                &config.password,
                &config.tls_config,
            )?;
            let mut connection = Connection::new(&info).await?;
            let mut builder = RouteBuilder::new(info.init.routing, bookmarks);
            if let Some(db) = db.clone() {
                builder = builder.with_db(db);
            }
            connection.route(builder.build(connection.version())).await
        })
    }
}
