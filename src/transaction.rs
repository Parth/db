use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::RwLockWriteGuard;

use crate::errors::Error;
use crate::log::SchemaEvent;
use crate::{Key, Value};

pub trait Transaction<'b, In> {
    fn transaction<F, Out>(&'b self, tx: F) -> Result<Out, Error>
    where
        F: for<'a> FnOnce(&'a mut In) -> Out;
}

pub struct TransactionTable<'a, K, V, Log>
where
    K: Key,
    V: Value,
    Log: SchemaEvent<K, V>,
{
    data: RwLockWriteGuard<'a, HashMap<K, V>>,
    pub pending: Vec<Log::LogEntry>,
    log: PhantomData<Log>,
}

impl<'a, K, V, Log> TransactionTable<'a, K, V, Log>
where
    K: Key,
    V: Value,
    Log: SchemaEvent<K, V>,
{
    pub fn init(data: RwLockWriteGuard<'a, HashMap<K, V>>) -> Self {
        let pending = vec![];
        let log = PhantomData {};
        Self { data, pending, log }
    }

    pub fn keys(&self) -> HashSet<&K> {
        self.data.keys().collect()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn get_all(&self) -> &HashMap<K, V> {
        self.data.deref()
    }

    pub fn exists(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        let prior = self.data.insert(key.clone(), val.clone());

        let s = Log::insert(key, val);
        self.pending.push(s);

        prior
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        let prior = self.data.remove(&key);

        let s = Log::delete(key);
        self.pending.push(s);

        prior
    }

    pub fn clear(&mut self) {
        self.data.clear();

        let s = Log::clear();
        self.pending.push(s);
    }
}
