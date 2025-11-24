use iqrah_storage::content::init_content_db;
use iqrah_storage::user::init_user_db;
use iqrah_storage::version::get_schema_version;
use tempfile::TempDir;

#[tokio::test]
async fn test_content_db_has_version() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("content.db");

    let pool = init_content_db(db_path.to_str().unwrap()).await.unwrap();
    let version = get_schema_version(&pool).await.unwrap();

    assert!(
        version.starts_with("2."),
        "Content DB should be version 2.x.x, found {}",
        version
    );
}

#[tokio::test]
async fn test_user_db_has_version() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("user.db");

    let pool = init_user_db(db_path.to_str().unwrap()).await.unwrap();
    let version = get_schema_version(&pool).await.unwrap();

    assert!(
        version.starts_with("1."),
        "User DB should be version 1.x.x, found {}",
        version
    );
}
