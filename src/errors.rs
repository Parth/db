use std::fmt::Display;

pub enum TableError {
    LockError(String),
}

impl TableError {
    pub(crate) fn lock_error<E: Display>(e: E) -> TableError {
        TableError::LockError(format!(
            "RwLock Poisoned, this indicates that one of your transactions panicked! Error: {}",
            e
        ))
    }
}
