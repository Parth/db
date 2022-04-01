#[cfg(test)]
pub mod schema {
    hmdb::schema! {
        Schema {
            word_counts: <String, u64>
        }
    }

    #[test]
    fn test() {
        // Schema::init("").word_counts.
    }
}
