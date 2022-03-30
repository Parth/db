use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

trait DiskFormat {
    type DiskRepr;

    fn to_disk(&self) -> Self::DiskRepr;
    fn from_disk(data: Self::DiskRepr, writer: Writer) -> Self;
}

trait Initialize<Schema> {
    fn init(path: &str) -> Schema;
}

pub trait Key: Clone {}
pub trait Value: Clone {}
impl<K> Key for K where K: Clone {}
impl<V> Value for V where V: Clone {}

pub struct Table<K: Key, V: Value> {
    data: Arc<RwLock<HashMap<K, V>>>,
    writer: Writer,
}

impl<K: Key, V: Value> Table<K, V> {
    fn snapshot(&self) -> HashMap<K, V> {
        self.data.read().unwrap().clone()
    }
}

pub struct Writer {}

macro_rules! schema {
    ($schema_name:ident {
        $($table_name: ident: <$table_key: ty, $table_value: ty>),*
    }) => {
        struct $schema_name {
            pub $($table_name: Table<$table_key, $table_value>),*
        }

        mod disk {
            pub struct $schema_name {
                $(pub $table_name: std::collections::HashMap<$table_key, $table_value>),*
            }
        }

        impl From<$schema_name> for disk::$schema_name {
            fn from(schema: $schema_name) -> Self {
                $(let $table_name = schema.$table_name.snapshot();)*
                Self {
                    $($table_name,)*
                }
            }
        }

        impl DiskFormat for $schema_name {
            type DiskRepr = disk::$schema_name;

            fn to_disk(&self) -> Self::DiskRepr {
                todo!()
            }

            fn from_disk(data: Self::DiskRepr, writer: Writer) -> Self {
                todo!()
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

fn test() {}
