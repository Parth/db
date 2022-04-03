#[cfg(test)]
pub mod schema {
    use std::fs;
    use std::path::PathBuf;

    fn test_db(db_file: &str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push(db_file);
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn start_empty() {
        hmdb::schema! {
            Schema {
                word_counts: <String, u64>,
                word_counts2: <String, u64>
            }
        }

        fs::remove_file(test_db("empty.db"))
            .unwrap_or_else(|_| println!("starting log did not exist"));

        let db = Schema::init(&test_db("empty.db")).unwrap();

        assert!(!db.incomplete_write);
    }

    hmdb::schema! {
        Schema {
            word_counts: <String, u64>,
            word_counts2: <String, u64>
        }
    }

    #[test]
    fn write_test() {
        fs::remove_file(test_db("write1.db"))
            .unwrap_or_else(|_| println!("starting log did not exist"));

        let db1 = Schema::init(&test_db("write1.db")).unwrap();
        db1.word_counts.insert("test".to_string(), 5).unwrap();

        let db2 = Schema::init(&test_db("write1.db")).unwrap();
        assert_eq!(
            db2.word_counts.get(&"test".to_string()).unwrap().unwrap(),
            5
        );
        db2.word_counts.insert("test2".to_string(), 3).unwrap();
        assert_eq!(
            db2.word_counts.get(&"test2".to_string()).unwrap().unwrap(),
            3
        );

        let db3 = Schema::init(&test_db("write1.db")).unwrap();
        assert_eq!(
            db3.word_counts.get(&"test2".to_string()).unwrap().unwrap(),
            3
        );
    }
}
