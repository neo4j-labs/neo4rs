use crate::config::ImpersonateUser;
use crate::summary::Counters;
use crate::{Database, DetachedRowStream, Error, Graph, Operation, Query, RowStream, RunResult};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Default)]
pub struct SessionConfig {
    db: Option<Database>,
    imp_user: Option<ImpersonateUser>,
    fetch_size: Option<usize>,
    bookmarks: Vec<String>,
}

impl SessionConfig {
    pub fn builder() -> SessionConfigBuilder {
        SessionConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct SessionConfigBuilder {
    db: Option<Database>,
    imp_user: Option<ImpersonateUser>,
    fetch_size: Option<usize>,
    bookmarks: Vec<String>,
}

impl SessionConfigBuilder {
    pub fn with_db(mut self, db: Database) -> Self {
        self.db = Some(db);
        self
    }

    pub fn with_imp_user(mut self, imp_user: ImpersonateUser) -> Self {
        self.imp_user = Some(imp_user);
        self
    }

    pub fn with_fetch_size(mut self, fetch_size: usize) -> Self {
        self.fetch_size = Some(fetch_size);
        self
    }

    pub fn with_bookmarks(mut self, bookmarks: Vec<String>) -> Self {
        self.bookmarks = bookmarks;
        self
    }

    pub fn build(self) -> SessionConfig {
        SessionConfig {
            db: self.db,
            imp_user: self.imp_user,
            fetch_size: self.fetch_size,
            bookmarks: self.bookmarks,
        }
    }
}

pub struct Session<'a> {
    db: Option<Database>,
    imp_user: Option<ImpersonateUser>,
    fetch_size: Option<usize>,
    bookmarks: Vec<String>,
    should_fetch_default_db: AtomicBool,
    driver: &'a Graph,
}

impl<'a> Session<'a> {
    pub(crate) fn new(config: SessionConfig, graph: &'a Graph) -> Session<'a> {
        Self {
            db: config.db,
            imp_user: config.imp_user,
            fetch_size: config.fetch_size,
            bookmarks: config.bookmarks,
            should_fetch_default_db: AtomicBool::new(true),
            driver: graph,
        }
    }

    pub async fn run(&mut self, query: impl Into<Query>) -> crate::Result<RunResult> {
        self.update_db_name().await?;
        match self
            .driver
            .impl_run_on(
                self.db.clone(),
                self.imp_user.clone(),
                &self.bookmarks,
                self.fetch_size,
                query.into(),
            )
            .await
        {
            Ok(result) => {
                if let Some(bookmark) = result.bookmark.as_ref() {
                    self.bookmarks = vec![bookmark.clone()];
                }
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn execute_read(
        &mut self,
        query: impl Into<Query>,
    ) -> crate::Result<DetachedRowStream> {
        self.update_db_name().await?;
        self.driver
            .impl_execute_on(
                Operation::Read,
                self.db.clone(),
                self.imp_user.clone(),
                &self.bookmarks,
                self.fetch_size,
                query.into(),
            )
            .await
    }

    pub async fn execute_write(
        &mut self,
        query: impl Into<Query>,
    ) -> crate::Result<DetachedRowStream> {
        self.update_db_name().await?;
        self.driver
            .impl_execute_on(
                Operation::Write,
                self.db.clone(),
                self.imp_user.clone(),
                &self.bookmarks,
                self.fetch_size,
                query.into(),
            )
            .await
    }

    pub async fn write_transaction(
        &mut self,
        queries: Vec<impl Into<Query>>,
    ) -> crate::Result<Counters> {
        self.update_db_name().await?;
        let mut txn = self
            .driver
            .impl_start_txn_on(
                self.db.clone(),
                Operation::Write,
                self.imp_user.clone(),
                &self.bookmarks,
                self.fetch_size,
            )
            .await?;
        match txn.run_queries(queries).await {
            Ok(counters) => match txn.commit().await {
                Ok(Some(bookmark)) => {
                    self.bookmarks = vec![bookmark.clone()];
                    Ok(counters)
                }
                Ok(None) => Ok(counters),
                Err(e) => Err(e),
            },
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    pub async fn read_transaction(&mut self, query: impl Into<Query>) -> crate::Result<RowStream> {
        self.update_db_name().await?;
        let mut txn = self
            .driver
            .impl_start_txn_on(
                self.db.clone(),
                Operation::Read,
                self.imp_user.clone(),
                &self.bookmarks,
                self.fetch_size,
            )
            .await?;
        txn.execute(query).await
    }

    pub fn last_bookmark(&self) -> Option<String> {
        self.bookmarks.last().cloned()
    }

    async fn update_db_name(&mut self) -> Result<(), Error> {
        if self.db.is_none() && self.should_fetch_default_db.fetch_or(false, Relaxed) {
            let db = self
                .driver
                .get_default_db(self.imp_user.clone(), &self.bookmarks)
                .await?;
            self.db = db;
            self.should_fetch_default_db
                .compare_exchange(true, false, Relaxed, Relaxed)
                .unwrap();
        }
        Ok(())
    }
}
