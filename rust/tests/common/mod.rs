// tests/common/mod.rs
use rust_lib_iqrah::sqlite_repo::SqliteRepository;
use std::{path::PathBuf, sync::Arc};

// This helper gives us a fresh in-memory DB for each test.
pub async fn setup_test_repo() -> Arc<SqliteRepository> {
    // let repo = SqliteRepository::new(None).expect("Failed to create in-memory repo");
    // delete the existing repo
    let _ = std::fs::remove_file("/tmp/dbg.sqlite");

    let repo = SqliteRepository::new(Some(PathBuf::from("/tmp/dbg.sqlite")))
        .expect("Failed to create in-memory repo");
    // The schema is created on new(), so no need to call it again.
    Arc::new(repo)
}
