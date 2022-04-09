#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use hmdb::schema;
    use hmdb::transaction::Transaction;

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
            table4: <u8, Value>
        }
    }

    fn test_db() -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push(Uuid::new_v4().to_string());
        path.to_str().unwrap().into()
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
}
