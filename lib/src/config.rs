pub use crate::errors::*;

const DEFAULT_FETCH_SIZE: i64 = 200;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) uri: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) db: String,
    pub(crate) fetch_size: i64,
}

pub struct ConfigBuilder {
    pub(crate) uri: Option<String>,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) db: Option<String>,
    pub(crate) fetch_size: Option<i64>,
}

impl ConfigBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_owned());
        self
    }

    pub fn user(mut self, user: &str) -> Self {
        self.user = Some(user.to_owned());
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    pub fn db(mut self, db: &str) -> Self {
        self.db = Some(db.to_owned());
        self
    }

    pub fn fetch_size(mut self, fetch_size: u32) -> Self {
        self.fetch_size = Some(fetch_size as i64);
        self
    }

    pub fn build(self) -> Result<Config> {
        if self.uri.is_none()
            || self.user.is_none()
            || self.password.is_none()
            || self.fetch_size.is_none()
        {
            Err(Error::InvalidConfig)
        } else {
            Ok(Config {
                uri: self.uri.unwrap(),
                user: self.user.unwrap(),
                password: self.password.unwrap(),
                fetch_size: self.fetch_size.unwrap(),
                db: self.db.unwrap(),
            })
        }
    }
}

pub fn config() -> ConfigBuilder {
    ConfigBuilder {
        uri: None,
        user: None,
        password: None,
        db: Some("".to_owned()),
        fetch_size: Some(DEFAULT_FETCH_SIZE),
    }
}
