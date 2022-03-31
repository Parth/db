use serde::ser::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::process::Output;
use std::sync::{Arc, PoisonError, RwLock};

type Namespace = String;

pub trait Key: Clone + Eq + Hash {}

pub trait Value: Clone {}

impl<K> Key for K where K: Clone + Eq + Hash {}

impl<V> Value for V where V: Clone {}

pub enum TableError {
    LockError(String),
}

impl TableError {
    fn lock_error<E: Display>(e: E) -> TableError {
        TableError::LockError(format!(
            "RwLock Poisoned, this indicates that one of your transactions panicked! Error: {}",
            e
        ))
    }
}

pub struct Table<K: Key, V: Value> {
    data: Arc<RwLock<HashMap<K, V>>>,
    writer: Writer,
}

impl<K: Key, V: Value> Table<K, V> {
    fn init(data: HashMap<K, V>, writer: Writer) -> Self {
        let data = Arc::new(RwLock::new(data));
        Self { data, writer }
    }

    fn get(&self, key: &K) -> Result<Option<V>, TableError> {
        let val = self
            .data
            .read()
            .map_err(TableError::lock_error)?
            .get(key)
            .cloned();
        Ok(val)
    }

    fn insert(&self, key: K, val: V) -> Result<Option<V>, TableError> {
        let val = self
            .data
            .write()
            .map_err(TableError::lock_error)?
            .insert(key, val);

        Ok(val)
    }
}

#[derive(serde::Serialize)]
pub enum TableEvent<K: Key, V: Value> {
    Insert(K, V),
    Delete(K),
}

pub trait Logger<K: Key, V: Value> {
    type Output: Serialize;

    fn insert<S: Serialize>(k: K, v: V) -> Self::Output;
    fn delete<S: Serialize>(k: K) -> Self::Output;
}

pub struct Writer {}

pub trait Init<OnDisk, InMemory> {
    fn read_from_disk(path: &str) -> Vec<OnDisk> {
        todo!()
    }

    fn init(path: &str) -> InMemory;
}

macro_rules! schema {
    ($schema_name:ident {
        $($table_name: ident: <$table_key: ty, $table_value: ty>),*
    }) => {
        struct $schema_name {
            $(pub $table_name: Table<$table_key, $table_value>),*
        }

        mod disk {
            #[allow(non_camel_case_types)]
            #[derive(serde::Serialize)]
            pub enum $schema_name {
                $($table_name(crate::TableEvent<$table_key, $table_value>)),*
            }
        }

        mod log {
            $(pub struct $table_name {})*
        }

        $(impl Logger<$table_key, $table_value> for log::$table_name {
            type Output = disk::$schema_name;

            fn insert<S: Serialize>(k: $table_key, v: $table_value) -> Self::Output {
                disk::$schema_name::$table_name(TableEvent::Insert(k, v))
            }
            fn delete<S: Serialize>(k: $table_key) -> Self::Output {
                disk::$schema_name::$table_name(TableEvent::Delete(k))
            }
        })*

        impl Init<disk::$schema_name, $schema_name> for $schema_name {
            fn init(path: &str) -> Self {
                let log = Self::read_from_disk(path);
                $(let mut $table_name: HashMap<$table_key, $table_value> = HashMap::new();)*
                for entry in log {
                    match entry {
                        $(
                            disk::$schema_name::$table_name(TableEvent::Insert(k, v)) => $table_name.insert(k, v),
                            disk::$schema_name::$table_name(TableEvent::Delete(k)) => todo!()
                        ),*
                    };
                }

                Self {
                    $($table_name: Table::init($table_name, Writer{})),*
                }
            }
        }
    }
}

schema! {
    SchemaV1 {
        accounts: <u8, String>,
        keys: <String, String>
    }
}

#[test]
pub fn test() {
    let db = SchemaV1::init("");
}
