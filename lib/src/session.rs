use crate::config::ImpersonateUser;
use crate::{Database, DetachedRowStream, Error, Graph, Query, RunResult};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct SessionConfig {
    db: Option<Database>,
    imp_user: Option<ImpersonateUser>,
    fetch_size: Option<usize>,
    bookmarks: Vec<String>,
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
                query.into(),
            )
            .await
        {
            Ok(result) => {
                if let Some(bookmark) = result.bookmark.as_ref() {
                    self.bookmarks.push(bookmark.clone());
                }
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn execute(&mut self, query: impl Into<Query>) -> crate::Result<DetachedRowStream> {
        self.update_db_name().await?;
        self.driver
            .impl_execute_on(
                self.db.clone(),
                self.imp_user.clone(),
                &self.bookmarks,
                query.into(),
            )
            .await
    }

    async fn update_db_name(&mut self) -> Result<(), Error> {
        if self.db.is_none()
            && self
            .should_fetch_default_db
            .fetch_or(false, Ordering::Relaxed)
        {
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
