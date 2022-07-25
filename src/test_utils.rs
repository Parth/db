use std::path::PathBuf;
use uuid::Uuid;

pub fn test_dbs_folder() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("benches");
    path
}

pub fn test_db() -> PathBuf {
    let mut path = test_dbs_folder().clone();
    path.push(Uuid::new_v4().to_string());
    path
}
