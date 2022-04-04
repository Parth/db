use crate::errors::TableError;
use crate::log::{LogFormat, Writer};
use crate::{Key, Value};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

#[derive(Clone)]
pub struct Table<K, V, Log>
where
    K: Key,
    V: Value,
    Log: LogFormat<K, V>,
{
    data: Arc<RwLock<HashMap<K, V>>>,
    writer: Writer,
    log: PhantomData<Log>,
}

impl<K, V, Log> Table<K, V, Log>
where
    K: Key,
    V: Value,
    Log: LogFormat<K, V>,
{
    pub fn init(data: HashMap<K, V>, writer: Writer) -> Self {
        let data = Arc::new(RwLock::new(data));
        let log = PhantomData {};
        Self { data, writer, log }
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, TableError> {
        let val = self
            .data
            .read()
            .map_err(TableError::lock_error)?
            .get(key)
            .cloned();
        Ok(val)
    }

    pub fn exists(&self, key: &K) -> Result<bool, TableError> {
        let val = self
            .data
            .read()
            .map_err(TableError::lock_error)?
            .contains_key(key);
        Ok(val)
    }

    pub fn insert(&self, key: K, val: V) -> Result<Option<V>, TableError> {
        let prior = self
            .data
            .write()
            .map_err(TableError::lock_error)?
            .insert(key.clone(), val.clone());

        let s = Log::insert(key, val);
        self.writer.append(&s);

        Ok(prior)
    }

    pub fn delete(&self, key: K) -> Result<Option<V>, TableError> {
        let prior = self
            .data
            .write()
            .map_err(TableError::lock_error)?
            .remove(&key);

        let s = Log::delete(key);
        self.writer.append(&s);

        Ok(prior)
    }

    #[doc(hidden)]
    pub fn begin_transaction(&self) -> (TransactionTable<K, V, Log>, Writer) {
        let data = self.data.write().unwrap();
        let pending = vec![];
        let log = PhantomData {};

        (TransactionTable { data, pending, log }, self.writer.clone())
    }
}

pub struct TransactionTable<'a, K, V, Log>
where
    K: Key,
    V: Value,
    Log: LogFormat<K, V>,
{
    data: RwLockWriteGuard<'a, HashMap<K, V>>,
    pub pending: Vec<Log::LogEntry>,
    log: PhantomData<Log>,
}

impl<'a, K, V, Log> TransactionTable<'a, K, V, Log>
where
    K: Key,
    V: Value,
    Log: LogFormat<K, V>,
{
    pub fn get(&self, key: &K) -> Result<Option<V>, TableError> {
        let val = self.data.get(key).cloned();
        Ok(val)
    }

    pub fn exists(&self, key: &K) -> Result<bool, TableError> {
        let val = self.data.contains_key(key);
        Ok(val)
    }

    pub fn insert(&mut self, key: K, val: V) -> Result<Option<V>, TableError> {
        let prior = self.data.insert(key.clone(), val.clone());

        let s = Log::insert(key, val);
        self.pending.push(s);

        Ok(prior)
    }

    pub fn delete(&mut self, key: K) -> Result<Option<V>, TableError> {
        let prior = self.data.remove(&key);

        let s = Log::delete(key);
        self.pending.push(s);

        Ok(prior)
    }
}
