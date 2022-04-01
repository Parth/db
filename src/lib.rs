use std::hash::Hash;

pub trait Key: Clone + Eq + Hash {}
pub trait Value: Clone {}
impl<K> Key for K where K: Clone + Eq + Hash {}
impl<V> Value for V where V: Clone {}

#[macro_export]
macro_rules! schema {
    ($schema_name:ident {
        $($table_name: ident: <$table_key: ty, $table_value: ty>),*
    }) => {

        use std::collections::HashMap;
        use hmdb::log::{TableEvent, Init, Logger, Writer};
        use hmdb::table::Table;

        struct $schema_name {
            $(pub $table_name: Table<$table_key, $table_value, log::$table_name>),*
        }

        mod disk {
            use hmdb::log::TableEvent;

            #[allow(non_camel_case_types)]
            #[derive(serde::Serialize)]
            pub enum $schema_name {
                $($table_name(TableEvent<$table_key, $table_value>)),*
            }
        }

        mod log {
            $(
                #[allow(non_camel_case_types)]
                pub struct $table_name {}
            )*
        }

        $(impl Logger<$table_key, $table_value> for log::$table_name {
            type Output = disk::$schema_name;

            fn insert(k: $table_key, v: $table_value) -> Self::Output {
                disk::$schema_name::$table_name(TableEvent::Insert(k, v))
            }
            fn delete(k: $table_key) -> Self::Output {
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
                    $($table_name: Table::init($table_name, Writer::init(path))),*
                }
            }
        }
    }
}

pub mod errors;
pub mod log;
pub mod table;
