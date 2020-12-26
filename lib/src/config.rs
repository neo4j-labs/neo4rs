pub use crate::errors::*;

const DEFAULT_FETCH_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) uri: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) db: String,
    pub(crate) fetch_size: usize,
}

pub struct ConfigBuilder {
    uri: Option<String>,
    user: Option<String>,
    password: Option<String>,
    db: Option<String>,
    fetch_size: Option<usize>,
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

    pub fn fetch_size(mut self, fetch_size: usize) -> Self {
        self.fetch_size = Some(fetch_size);
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
