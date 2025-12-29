/// Creates a test content.db file with sample data for integration testing
use iqrah_storage::init_test_content_db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "content.db".to_string());

    println!("Creating test database at: {}", db_path);

    // Create the database with sample data
    let _pool = init_test_content_db(&db_path).await?;

    println!("✓ Successfully created test database with sample data");
    Ok(())
}
