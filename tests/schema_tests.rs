#[cfg(test)]
pub mod schema {
    use std::fs;
    use std::path::PathBuf;

    fn test_db(db_file: &str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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
}
