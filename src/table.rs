use crate::errors::Error;
use crate::log::{SchemaEvent, Writer};
use crate::transaction::TransactionTable;
use crate::{Key, Value};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct Table<K, V, Log>
where
    K: Key,
    V: Value,
    Log: SchemaEvent<K, V>,
{
    data: Arc<RwLock<HashMap<K, V>>>,
    writer: Writer,
    log: PhantomData<Log>,
}

impl<K, V, Log> Table<K, V, Log>
where
    K: Key,
    V: Value,
    Log: SchemaEvent<K, V>,
{
    pub fn init(data: HashMap<K, V>, writer: Writer) -> Self {
        let data = Arc::new(RwLock::new(data));
        let log = PhantomData {};
        Self { data, writer, log }
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, Error> {
        let val = self
            .data
            .read()
            .map_err(Error::lock_error)?
            .get(key)
            .cloned();
        Ok(val)
    }

    pub fn exists(&self, key: &K) -> Result<bool, Error> {
        let val = self
            .data
            .read()
            .map_err(Error::lock_error)?
            .contains_key(key);
        Ok(val)
    }

    pub fn insert(&self, key: K, val: V) -> Result<Option<V>, Error> {
        let prior = self
            .data
            .write()
            .map_err(Error::lock_error)?
            .insert(key.clone(), val.clone());

        let s = Log::insert(key, val);
        self.writer.append(&s)?;

        Ok(prior)
    }

    pub fn delete(&self, key: K) -> Result<Option<V>, Error> {
        let prior = self.data.write().map_err(Error::lock_error)?.remove(&key);

        let s = Log::delete(key);
        self.writer.append(&s)?;

        Ok(prior)
    }

    #[doc(hidden)]
    pub fn begin_transaction(&self) -> Result<(TransactionTable<K, V, Log>, Writer), Error> {
        let data = self.data.write().map_err(Error::lock_error)?;

        Ok((TransactionTable::init(data), self.writer.clone()))
    }
}
