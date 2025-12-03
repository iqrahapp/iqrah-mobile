use anyhow::Result;
use iqrah_core::{import_cbor_graph_from_bytes, ContentRepository};
use iqrah_storage::{create_content_repository, init_content_db};
use std::fs::File;
use std::sync::Arc;

/// Import CBOR graph data directly into the content database
pub async fn import_cbor(cbor_file: &str) -> Result<()> {
    println!("📥 Importing CBOR graph from: {}", cbor_file);
    println!();

    // Get database path from environment or use default
    let content_db_path =
        std::env::var("CONTENT_DB_PATH").unwrap_or_else(|_| "data/content.db".to_string());

    println!("   Using content database: {}", content_db_path);
    println!();

    // Initialize database
    let content_pool = init_content_db(&content_db_path).await?;
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(create_content_repository(content_pool));

    // Open and decompress the CBOR file
    let file = File::open(cbor_file)?;
    let decoder = zstd::Decoder::new(file)?;

    // Import the graph
    println!("   Importing graph data...");
    let stats = import_cbor_graph_from_bytes(content_repo, decoder).await?;

    // Print results
    println!();
    println!("✅ Import Complete!");
    println!();
    println!("   Nodes imported: {}", stats.nodes_imported);
    println!("   Edges imported: {}", stats.edges_imported);

    Ok(())
}
