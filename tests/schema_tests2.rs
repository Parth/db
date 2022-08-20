#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use std::time::Duration;

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use hmdb::schema;
    use hmdb::transaction::Transaction;

    const SCHEMA_NAME: &str = "schema_tests2__tests__Db";

    #[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Test;

    #[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Value {
        pub field: Vec<u8>,
        pub field2: Vec<u8>,
    }
    schema! {
        Db {
            table1: <Test, String>,
            table2: <Test, u128>,
            table3: <String, Vec<u8>>,
            table4: <u8, Value>,
            table5: <String, u8>
        }
    }

    fn test_db() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(Uuid::new_v4().to_string())
    }

    #[test]
    fn test_non_keys() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
        let db = Db::init(db_path).unwrap();

        assert!(!db.table1.exists(&Test {}).unwrap());
        assert!(!db.table2.exists(&Test {}).unwrap());

        db.table1.insert(Test {}, "test".to_string()).unwrap();

        assert!(db.table1.exists(&Test {}).unwrap());
        assert!(!db.table2.exists(&Test {}).unwrap());

        let db = Db::init(db_path).unwrap();
        assert!(db.table1.exists(&Test {}).unwrap());
        assert!(!db.table2.exists(&Test {}).unwrap());

        db.transaction(|tx| {
            tx.table1.delete(Test {});
            tx.table2.insert(Test {}, u128::MAX);
        })
        .unwrap();

        assert!(!db.table1.exists(&Test {}).unwrap());
        assert_eq!(db.table2.get(&Test {}).unwrap().unwrap(), u128::MAX);

        let db = Db::init(db_path).unwrap();
        assert!(!db.table1.exists(&Test {}).unwrap());
        assert_eq!(db.table2.get(&Test {}).unwrap().unwrap(), u128::MAX);

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
    }

    #[test]
    fn test_binary_data() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
        let db = Db::init(db_path).unwrap();
        db.table3.insert("Test".to_string(), vec![1, 2, 3]).unwrap();
        let db = Db::init(db_path).unwrap();
        assert_eq!(
            db.table3.get(&"Test".to_string()).unwrap().unwrap(),
            vec![1, 2, 3]
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
    }

    #[test]
    fn test_more_binary_data() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
        let db = Db::init(db_path).unwrap();
        db.table4
            .insert(
                1,
                Value {
                    field: vec![1, 2, 3],
                    field2: vec![4, 5, 6],
                },
            )
            .unwrap();

        let db = Db::init(db_path).unwrap();
        assert_eq!(
            db.table4.get(&1).unwrap().unwrap(),
            Value {
                field: vec![1, 2, 3],
                field2: vec![4, 5, 6],
            }
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
    }

    #[test]
    fn test_more_binary_data_2() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
        let db = Db::init(db_path).unwrap();
        db.transaction(|tx| {
            tx.table4.insert(
                1,
                Value {
                    field: vec![1, 2, 3],
                    field2: vec![4, 5, 6],
                },
            );
            tx.table3.insert("test".to_string(), vec![1, 2, 3]);
        })
        .unwrap();

        let db = Db::init(db_path).unwrap();
        assert_eq!(
            db.table4.get(&1).unwrap().unwrap(),
            Value {
                field: vec![1, 2, 3],
                field2: vec![4, 5, 6],
            }
        );
        assert_eq!(
            db.table3.get(&"test".to_string()).unwrap().unwrap(),
            vec![1, 2, 3]
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| println!("starting log did not exist"));
    }

    #[test]
    fn test_clear_no_tx_1() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();
        db.table1.insert(Test {}, "test".to_string()).unwrap();
        db.table2.insert(Test {}, 123).unwrap();
        db.table3.insert("a".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("c".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("d".to_string(), vec![1, 2, 3]).unwrap();

        db.transaction(|tx| {
            tx.table3.clear();
        })
        .unwrap();

        assert!(db.table3.get_all().unwrap().is_empty());
        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_clear_no_tx_2() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();
        db.table1.insert(Test {}, "test".to_string()).unwrap();
        db.table2.insert(Test {}, 123).unwrap();
        db.table3.insert("a".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("c".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("d".to_string(), vec![1, 2, 3]).unwrap();

        let db = Db::init(db_path).unwrap();
        db.transaction(|tx| {
            tx.table3.clear();
        })
        .unwrap();

        assert!(db.table3.get_all().unwrap().is_empty());
        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_clear_no_tx_3() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();
        db.table1.insert(Test {}, "test".to_string()).unwrap();
        db.table2.insert(Test {}, 123).unwrap();
        db.table3.insert("a".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("c".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("d".to_string(), vec![1, 2, 3]).unwrap();

        let db = Db::init(db_path).unwrap();
        db.transaction(|tx| {
            tx.table3.clear();
        })
        .unwrap();

        let db = Db::init(db_path).unwrap();
        assert!(db.table3.get_all().unwrap().is_empty());
        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_background_compacter_1() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table1.insert(Test {}, "apple".to_string()).unwrap();
        db.table1.insert(Test {}, "sauce".to_string()).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();
        db.start_background_compacter(Duration::from_secs(3))
            .unwrap();
        thread::sleep(Duration::from_secs(10));
        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert!(size_before > size_after);

        assert_eq!(
            db.table1.get(&Test {}).unwrap().unwrap(),
            "sauce".to_string()
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_background_compacter_2() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table2.insert(Test {}, 126).unwrap();

        db.table3.insert("c".to_string(), vec![1]).unwrap();
        db.table3.insert("c".to_string(), vec![1, 3]).unwrap();
        db.table3.insert("d".to_string(), vec![3, 2, 1]).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();
        db.start_background_compacter(Duration::from_secs(3))
            .unwrap();
        thread::sleep(Duration::from_secs(10));
        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert!(size_before > size_after);

        assert_eq!(
            db.table3.get(&"c".to_string()).unwrap().unwrap(),
            vec![1, 3]
        );
        assert_eq!(
            db.table3.get(&"d".to_string()).unwrap().unwrap(),
            vec![3, 2, 1]
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_background_compacter_compaction_3() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table4
            .insert(
                1,
                Value {
                    field: vec![1, 2, 3],
                    field2: vec![4, 5, 6],
                },
            )
            .unwrap();
        db.table4
            .insert(
                1,
                Value {
                    field: vec![6, 5, 4],
                    field2: vec![3, 2, 1],
                },
            )
            .unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();
        db.start_background_compacter(Duration::from_secs(3))
            .unwrap();
        thread::sleep(Duration::from_secs(5));

        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert!(size_before > size_after);

        assert_eq!(
            db.table4.get(&1).unwrap().unwrap(),
            Value {
                field: vec![6, 5, 4],
                field2: vec![3, 2, 1],
            }
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_background_compacter_on_init() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        db.start_background_compacter(Duration::from_secs(3))
            .unwrap();
        thread::sleep(Duration::from_secs(5));

        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert_eq!(size_before, 0);
        assert_eq!(size_after, 16);

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_background_compacter_on_clear() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table3.insert("c".to_string(), vec![3, 2, 1]).unwrap();
        db.table3.insert("c".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("d".to_string(), vec![1]).unwrap();
        db.table3.insert("d".to_string(), vec![1, 3]).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        db.transaction(|tx| tx.table3.clear()).unwrap();
        db.start_background_compacter(Duration::from_secs(3))
            .unwrap();
        thread::sleep(Duration::from_secs(5));
        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert!(size_before > size_after);

        assert_eq!(db.table3.get(&"c".to_string()).unwrap(), None);
        assert_eq!(db.table3.get(&"d".to_string()).unwrap(), None);

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_background_compacter_multiple_edits() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table3.insert("a".to_string(), vec![3, 2, 1]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 3]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 2]).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        db.start_background_compacter(Duration::from_secs(3))
            .unwrap();
        thread::sleep(Duration::from_secs(5));

        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert!(size_before > size_after);

        assert_eq!(
            db.table3.get(&"a".to_string()).unwrap().unwrap(),
            vec![3, 2, 1]
        );
        assert_eq!(
            db.table3.get(&"b".to_string()).unwrap().unwrap(),
            vec![1, 2]
        );

        db.table2.insert(Test {}, 15).unwrap();
        db.table2.insert(Test {}, 20).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        thread::sleep(Duration::from_secs(4));

        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert!(size_before > size_after);

        assert_eq!(db.table2.get(&Test {}).unwrap().unwrap(), 20);

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_log_compaction_1() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table1.insert(Test {}, "test".to_string()).unwrap();
        db.table1.insert(Test {}, "test".to_string()).unwrap();
        db.table1.insert(Test {}, "test".to_string()).unwrap();
        db.table1.insert(Test {}, "test".to_string()).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();
        db.compact_log().unwrap();
        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert_eq!(size_before, 112);
        assert_eq!(size_after, 36);

        assert_eq!(
            db.table1.get(&Test {}).unwrap().unwrap(),
            "test".to_string()
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_log_compaction_2() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table2.insert(Test {}, 123).unwrap();
        db.table2.insert(Test {}, 124).unwrap();
        db.table2.insert(Test {}, 125).unwrap();
        db.table2.insert(Test {}, 126).unwrap();

        db.table3.insert("a".to_string(), vec![3, 2, 1]).unwrap();
        db.table3.insert("a".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("b".to_string(), vec![1]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 3]).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();
        db.compact_log().unwrap();
        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert_eq!(size_before, 269);
        assert_eq!(size_after, 95);

        assert_eq!(
            db.table3.get(&"a".to_string()).unwrap().unwrap(),
            vec![1, 2, 3]
        );
        assert_eq!(
            db.table3.get(&"b".to_string()).unwrap().unwrap(),
            vec![1, 3]
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_log_compaction_3() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table4
            .insert(
                1,
                Value {
                    field: vec![1, 2, 3],
                    field2: vec![4, 5, 6],
                },
            )
            .unwrap();
        db.table4
            .insert(
                1,
                Value {
                    field: vec![6, 5, 4],
                    field2: vec![3, 2, 1],
                },
            )
            .unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();
        db.compact_log().unwrap();
        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert_eq!(size_before, 78);
        assert_eq!(size_after, 47);

        assert_eq!(
            db.table4.get(&1).unwrap().unwrap(),
            Value {
                field: vec![6, 5, 4],
                field2: vec![3, 2, 1],
            }
        );

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_log_compaction_on_init() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        db.compact_log().unwrap();

        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert_eq!(size_before, 0);
        assert_eq!(size_after, 16);

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }

    #[test]
    fn test_log_compaction_on_clear() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
        let db = Db::init(db_path).unwrap();

        db.table3.insert("a".to_string(), vec![3, 2, 1]).unwrap();
        db.table3.insert("a".to_string(), vec![1, 2, 3]).unwrap();
        db.table3.insert("b".to_string(), vec![1]).unwrap();
        db.table3.insert("b".to_string(), vec![1, 3]).unwrap();

        let size_before = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        db.transaction(|tx| tx.table3.clear()).unwrap();
        db.compact_log().unwrap();

        let size_after = File::open(db_path.join(SCHEMA_NAME))
            .unwrap()
            .metadata()
            .unwrap()
            .len();

        assert_eq!(size_before, 141);
        assert_eq!(size_after, 16);

        assert_eq!(db.table3.get(&"a".to_string()).unwrap(), None);
        assert_eq!(db.table3.get(&"b".to_string()).unwrap(), None);

        fs::remove_dir_all(db_path).unwrap_or_else(|_| {});
    }
}
