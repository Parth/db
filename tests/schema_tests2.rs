#[cfg(test)]
pub mod tests {
    use hmdb::schema;
    use hmdb::transaction::Transaction;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Test;

    schema! {
        Db {
            table1: <Test, String>,
            table2: <Test, u128>
        }
    }

    fn test_db(db_file: &str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push(db_file);
        path.to_str().unwrap().into()
    }

    #[test]
    fn test_non_keys() {
        let db_path = &test_db("write1_db");

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
}
