use std::path::PathBuf;
use uuid::Uuid;

pub fn test_dbs_folder() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/tmp")
}

pub fn test_db() -> PathBuf {
    let mut path = test_dbs_folder();
    path.push(Uuid::new_v4().to_string());
    path
}
