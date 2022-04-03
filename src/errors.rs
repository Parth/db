use std::fmt::Display;
use std::io;

#[derive(Debug)]
pub enum TableError {
    LockError(String),
}

impl TableError {
    pub(crate) fn lock_error<E: Display>(e: E) -> Self {
        Self::LockError(format!(
            "RwLock Poisoned, this indicates that one of your transactions panicked! Error: {}",
            e
        ))
    }
}

#[derive(Debug)]
pub enum ReadError {
    OsError(String, io::Error),
    LogParseError(String, bincode::Error),
}
