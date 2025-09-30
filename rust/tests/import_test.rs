// tests/import_test.rs
mod common;
use rust_lib_iqrah::cbor_import::import_cbor_graph_from_file;
use rust_lib_iqrah::propagation::DistributionParams;
use rust_lib_iqrah::repository::KnowledgeGraphRepository;

#[tokio::test]
async fn test_full_cbor_import_and_verification() {
    // ARRANGE: Set up a fresh in-memory database
    let repo = common::setup_test_repo().await;

    // ACT: Run the import process on our test file
    let stats = import_cbor_graph_from_file(&*repo, "tests/data/test-graph.cbor.zst".to_string())
        .await
        .expect("CBOR import failed");

    // ASSERT: Check the high-level stats and the specific data in the DB
    assert_eq!(stats.nodes_imported, 3);
    assert_eq!(stats.edges_imported, 2);

    // Verify that the edges were inserted correctly
    let edges = repo
        .get_knowledge_edges("word:test1")
        .await
        .expect("Failed to get edges");

    assert_eq!(edges.len(), 2);

    // Find and verify the beta distribution edge
    let beta_edge = edges
        .iter()
        .find(|e| e.target_node_id == "root:tst")
        .expect("Beta edge not found");

    match beta_edge.distribution {
        DistributionParams::Beta { alpha, beta } => {
            assert_eq!(alpha, 4.0);
            assert_eq!(beta, 2.0);
        }
        _ => panic!("Expected Beta distribution"),
    }
}
