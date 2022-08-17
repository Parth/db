//! # hmdb
//!
//! An embedded database with the following properties:
//!
//! + Read Optimized
//! + Persistent
//! + Transactional
//! + In-Memory
//! + Key Value Store
//! + Schema defined and enforced in Rust
//!
//! ## Defining your schema
//!
//! ```ignore,rust
//!
//! hmdb::schema! {
//!     SchemaName {
//!         table1_name: <u8, String>,
//!         table2_name: <String, u64>
//!     }
//! }
//! ```
//!
//! ## Reading your db file
//!
//! ```ignore, rust
//! let db = SchemaName::init("db_dir").unwrap();
//! ```
//!
//! ## Using your tables
//!
//! ```ignore, rust
//! db.table1_name.insert(5, "test".to_string()).unwrap();
//! let val = db.table1_name.get(5).unwrap().unwrap();
//!
//! assert_eq(5, val);
//! ```
//!
//! ## Creating a transaction
//!
//! ```ignore, rust
//!  db.transaction(|tx| {
//!      let mut num = tx.table2_name.get(&"test".to_string()).unwrap();
//!      num += 1;
//!      tx.table2_name.insert("test".to_string(), num).unwrap();
//!  }).unwrap();
//! ```

#![forbid(unsafe_code)]

use std::hash::Hash;

#[macro_export]
macro_rules! head {
    ($head: ident, $($rest: ident),*) => {
        $head
    };
}

#[macro_export]
macro_rules! schema {
    ($schema_name:ident {
        $($table_name: ident: <$table_key: ty, $table_value: ty>),+
    }) => {

        use std::collections::HashMap;
        use $crate::log::{TableEvent, Reader, SchemaEvent, Writer, LogCompacter};
        use $crate::table::Table;
        use std::path::Path;
        use std::thread;

        #[derive(Clone, Debug)]
        pub struct $schema_name {
            incomplete_write: bool,
            $(pub $table_name: Table<$table_key, $table_value, helper_log::$table_name>),*
        }

        pub mod transaction {
            use super::*;
            use $crate::transaction::TransactionTable;

            pub struct $schema_name<'a> {
                $(pub $table_name: TransactionTable<'a, $table_key, $table_value, helper_log::$table_name>),*
            }
        }

        mod helper_disk {
            use super::*;
            use $crate::log::TableEvent;

            #[allow(non_camel_case_types)]
            #[derive(serde::Serialize, serde::Deserialize)]
            pub enum $schema_name {
                $($table_name(TableEvent<$table_key, $table_value>)),*
            }
        }

        pub mod helper_log {
            use super::*;
            $(
                #[derive(Clone, Debug)]
                #[allow(non_camel_case_types)]
                pub struct $table_name {}
            )*
        }

        $(impl SchemaEvent<$table_key, $table_value> for helper_log::$table_name {
            type LogEntry = helper_disk::$schema_name;

            fn insert(k: $table_key, v: $table_value) -> Self::LogEntry {
                helper_disk::$schema_name::$table_name(TableEvent::Insert(k, v))
            }
            fn delete(k: $table_key) -> Self::LogEntry {
                helper_disk::$schema_name::$table_name(TableEvent::Delete(k))
            }
            fn clear() -> Self::LogEntry {
                helper_disk::$schema_name::$table_name(TableEvent::Clear)
            }
        })*

        impl Reader<helper_disk::$schema_name, $schema_name> for $schema_name {
            fn init<P: AsRef<Path>>(path: P) -> Result<Self, $crate::errors::Error> {
                let (mut file, schema_path) = Self::open_log(&path)?;
                let (log, incomplete_write) = Self::parse_log(&mut file)?;
                let writer = Writer::init(file, schema_path);
                $(let mut $table_name: HashMap<$table_key, $table_value> = HashMap::new();)*
                for entry in log {
                    match entry {
                        $(
                            helper_disk::$schema_name::$table_name(TableEvent::Insert(k, v)) => { $table_name.insert(k, v); }
                            helper_disk::$schema_name::$table_name(TableEvent::Delete(k)) => { $table_name.remove(&k); }
                            helper_disk::$schema_name::$table_name(TableEvent::Clear) => { $table_name.clear(); }
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

        impl LogCompacter for $schema_name {
            fn compact_log(&self) -> Result<(), $crate::errors::Error> {
                $(let ($table_name, writer) = self.$table_name.begin_transaction()?;)*

                let mut data = vec![];
                $(
                    for (key, val) in $table_name.get_all() {
                        data.push(helper_log::$table_name::insert(key, val));
                    }
                )*

                writer.compact_log(data).unwrap();

                Ok(())
            }

            fn compact_log_async(&self, time_between_compacts: Duration) -> Result<(), $crate::errors::Error> {
                $(let $table_name = self.$table_name.clone();)*

                thread::spawn(move || {
                    loop {
                        $(let ($table_name, writer) = $table_name.begin_transaction().unwrap();)*

                        let mut data = vec![];
                        $(
                            for (key, val) in $table_name.get_all() {
                                data.push(helper_log::$table_name::insert(key, val));
                            }
                        )*

                        writer.compact_log(data).unwrap();
                        thread::sleep(time_between_compacts)
                    }
                });

                Ok(())
            }
        }

        impl<'b> $crate::transaction::Transaction<'b, transaction::$schema_name<'b>> for $schema_name {
             fn transaction<F, Out>(&'b self, tx: F) -> Result<Out, $crate::errors::Error>
             where
                F: for<'a> FnOnce(&'a mut transaction::$schema_name<'b>) -> Out,
             {
                $(let ($table_name, writer) = self.$table_name.begin_transaction()?;)*

                let mut db = transaction::$schema_name {
                    $($table_name: $table_name,)*
                };

                let ret = tx(&mut db);
                let mut result = vec![];
                $(result.extend(db.$table_name.pending);)*

                writer.append_all(result)?;
                Ok(ret)
            }
        }
    }
}

pub mod errors;
pub mod log;
pub mod table;
pub mod transaction;

pub trait Key: Clone + Eq + Hash {}
pub trait Value: Clone {}
impl<K> Key for K where K: Clone + Eq + Hash {}
impl<V> Value for V where V: Clone {}

// TODO: Document all the traits

// TODO: Consider taking a `Path` instead of an str

// TODO: The experience of using `String` as your key is somewhat bad, it complains you provided an
//       &str when it expected &String

// TODO: Have schema migration tests and examples

// TODO: More macro hygiene, see schema_tests::start_empty
