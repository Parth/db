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
        use hmdb::log::{TableEvent, Reader, LogFormat, Writer};
        use hmdb::table::Table;

        struct $schema_name {
            incomplete_write: bool,
            $(pub $table_name: Table<$table_key, $table_value, log::$table_name>),*
        }

        mod disk {
            use hmdb::log::TableEvent;

            #[allow(non_camel_case_types)]
            #[derive(serde::Serialize, serde::Deserialize)]
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

        $(impl LogFormat<$table_key, $table_value> for log::$table_name {
            type LogEntry = disk::$schema_name;

            fn insert(k: $table_key, v: $table_value) -> Self::LogEntry {
                disk::$schema_name::$table_name(TableEvent::Insert(k, v))
            }
            fn delete(k: $table_key) -> Self::LogEntry {
                disk::$schema_name::$table_name(TableEvent::Delete(k))
            }
        })*

        impl Reader<disk::$schema_name, $schema_name> for $schema_name {
            fn init(path: &str) -> Result<Self, hmdb::errors::ReadError> {
                let mut file = Self::open_file(path)?;
                let (log, incomplete_write) = Self::parse_log(&mut file).unwrap();
                let writer = Writer::init(file);
                $(let mut $table_name: HashMap<$table_key, $table_value> = HashMap::new();)*
                for entry in log {
                    match entry {
                        $(
                            disk::$schema_name::$table_name(TableEvent::Insert(k, v)) => $table_name.insert(k, v),
                            disk::$schema_name::$table_name(TableEvent::Delete(k)) => $table_name.remove(&k)
                        ),*
                    };
                }

                Ok(
                    Self {
                        incomplete_write,
                        $($table_name: Table::init($table_name, writer.clone())),*
                    }
                )
            }

            fn incomplete_write(&self) -> bool {
                self.incomplete_write
            }
        }
    }
}

pub mod errors;
pub mod log;
pub mod table;

// TODO: Remove all unwraps

// TODO: Document all the traits

// TODO: Tests that write to the db, open that db check contents, do another write, open another db
//       check contents. Make sure writer isn't starting from the top when it starts writing (after reading)

// TODO: Log compaction

// TODO: Consider taking a `Path` instead of an str
