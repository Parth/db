use std::fmt::Display;
use std::io;

#[derive(Debug)]
pub enum Error {
    OsError(String, io::Error),
    LogParseError(String, bincode::Error),
    LockError(String),
    SerializeError(String, bincode::Error),
}

impl Error {
    pub(crate) fn lock_error<E: Display>(e: E) -> Self {
        Self::LockError(format!(
            "RwLock Poisoned, this indicates that one of your transactions panicked! Error: {}",
            e
        ))
    }

    pub(crate) fn serialize(type_name: &str, e: bincode::Error) -> Self {
        Self::SerializeError(
            format!(
                "Unexpected Error while trying to serialize type {}. Bincode error: {}",
                type_name, e
            ),
            e,
        )
    }
}
