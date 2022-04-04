#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct File {}

#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Name {}

#[cfg(test)]
pub mod schema {
    use hmdb::schema;

    schema! {
        AppSchemaV1 {
            accounts: <u64, String>,
            files: <crate::Name, crate::File>
        }
    }

    #[test]
    fn test() {
        let db = AppSchemaV1::init("data.db").unwrap();
        let value = db.transaction(|tx_db| {
            tx_db.accounts.get(&8).unwrap();

            75
        });
    }
}
