mod common;
use anyhow::Result;
use rust_lib_iqrah::{
    cbor_import::{ImportedEdge, ImportedNode},
    propagation::{DistributionParams, EdgeType},
    repository::{KnowledgeGraphRepository, LearningService, ReviewGrade},
};

#[tokio::test]
async fn test_review_and_propagation() -> Result<()> {
    // =================================================================
    // ARRANGE: Set up a knowledge flow from a root to two words.
    // =================================================================
    let repo = common::setup_test_repo().await;
    let service = LearningService::new(repo.clone());
    let user_id = "test_user";

    repo.insert_nodes_batch(&[
        ImportedNode {
            id: "root:rhm".to_string(),
            attributes: Default::default(),
        },
        ImportedNode {
            id: "word:rahman".to_string(),
            attributes: Default::default(),
        },
        ImportedNode {
            id: "word:rahim".to_string(),
            attributes: Default::default(),
        },
        ImportedNode {
            id: "verse:1:1".to_string(),
            attributes: Default::default(),
        },
    ])
    .await?;

    repo.insert_edges_batch(&[
        // KNOWLEDGE edge: Learning the root helps with "rahman".
        ImportedEdge {
            source_id: "root:rhm".to_string(),
            target_id: "word:rahman".to_string(),
            edge_type: EdgeType::Knowledge,
            distribution: DistributionParams::Beta {
                alpha: 4.0,
                beta: 2.0,
            },
        },
        // KNOWLEDGE edge: Learning the root also helps with "rahim".
        ImportedEdge {
            source_id: "root:rhm".to_string(),
            target_id: "word:rahim".to_string(),
            edge_type: EdgeType::Knowledge,
            distribution: DistributionParams::Normal {
                mean: 0.5,
                std_dev: 0.1,
            },
        },
        // DEPENDENCY edge: This should be IGNORED by the propagation engine.
        ImportedEdge {
            source_id: "word:rahman".to_string(),
            target_id: "verse:1:1".to_string(),
            edge_type: EdgeType::Dependency,
            distribution: DistributionParams::Constant { weight: 1.0 },
        },
    ])
    .await?;

    repo.sync_user_nodes(user_id).await?;

    // =================================================================
    // ACT: Review the foundational concept, "root:rhm".
    // =================================================================
    let _ = service
        .process_review(user_id, "root:rhm", ReviewGrade::Good)
        .await?;

    // =================================================================
    // ASSERT: Energy propagated ONLY along KNOWLEDGE edges.
    // =================================================================
    let energy_root = repo.get_node_energy(user_id, "root:rhm").await?.unwrap();
    let energy_rahman = repo.get_node_energy(user_id, "word:rahman").await?.unwrap();
    let energy_rahim = repo.get_node_energy(user_id, "word:rahim").await?.unwrap();
    let energy_verse = repo.get_node_energy(user_id, "verse:1:1").await?.unwrap();

    assert!(
        energy_root > 0.0,
        "The reviewed node's energy should increase"
    );
    assert!(
        energy_rahman > 0.0,
        "Energy should propagate to 'rahman' via Knowledge edge"
    );
    assert!(
        energy_rahim > 0.0,
        "Energy should propagate to 'rahim' via Knowledge edge"
    );
    assert_eq!(
        energy_verse, 0.0,
        "Energy should NOT propagate along Dependency edge"
    );

    Ok(())
}
