use crate::config::ImpersonateUser;
use crate::connection::{Connection, ConnectionInfo};
use crate::routing::{RouteBuilder, RoutingTable};
use crate::{Config, Database, Error};
use std::future::Future;
use std::pin::Pin;

pub(crate) trait RoutingTableProvider: Send + Sync {
    fn fetch_routing_table(
        &self,
        bookmarks: &[String],
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
    ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>>;
}

pub struct ClusterRoutingTableProvider {
    config: Config,
}

impl ClusterRoutingTableProvider {
    pub fn new(config: Config) -> Self {
        ClusterRoutingTableProvider { config }
    }
}

impl RoutingTableProvider for ClusterRoutingTableProvider {
    fn fetch_routing_table(
        &self,
        bookmarks: &[String],
        db: Option<Database>,
        imp_user: Option<ImpersonateUser>,
    ) -> Pin<Box<dyn Future<Output = Result<RoutingTable, Error>> + Send>> {
        let config = self.config.clone();
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
                if !db.is_empty() {
                    builder = builder.with_db(db);
                }
            }
            if let Some(imp_user) = imp_user.clone() {
                if !imp_user.is_empty() {
                    builder = builder.with_imp_user(imp_user);
                }
            }
            connection.route(builder.build(connection.version())).await
        })
    }
}
