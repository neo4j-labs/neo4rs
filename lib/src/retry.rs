use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy)]
pub enum Retry<E> {
    Yes(E),
    No(E),
}

impl<E> Retry<E> {
    pub fn yes(e: E) -> Retry<E> {
        Retry::Yes(e)
    }

    pub fn no(e: E) -> Retry<E> {
        Retry::No(e)
    }

    pub fn inner(&self) -> &E {
        match self {
            Retry::Yes(e) => e,
            Retry::No(e) => e,
        }
    }

    pub fn into_inner(self) -> E {
        match self {
            Retry::Yes(e) => e,
            Retry::No(e) => e,
        }
    }
}

impl<E: Display> Display for Retry<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Retry::Yes(e) => e.fmt(f),
            Retry::No(e) => e.fmt(f),
        }
    }
}

impl<E: Error + 'static> Error for Retry<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner())
    }
}

impl<E> From<E> for Retry<E> {
    fn from(e: E) -> Self {
        Retry::Yes(e)
    }
}
