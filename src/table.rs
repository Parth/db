use crate::errors::TableError;
use crate::log::{LogFormat, Writer};
use crate::{Key, Value};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

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

    pub fn delete(&self, _key: K) {
        todo!()
    }
}
