// tests/knowledge_axis_test.rs
// Integration test for Phase 4: Knowledge Axis support

mod common;

use iqrah_core::{KnowledgeAxis, KnowledgeNode, Node, NodeType};

#[tokio::test]
async fn test_knowledge_node_parsing() {
    // Test parsing knowledge node IDs with different axes

    // Test memorization axis
    let node_id = "WORD_INSTANCE:1:1:1:memorization";
    let kn = KnowledgeNode::parse(node_id);
    assert!(kn.is_some(), "Should parse memorization node");
    let kn = kn.unwrap();
    assert_eq!(kn.base_node_id, "WORD_INSTANCE:1:1:1");
    assert_eq!(kn.axis, KnowledgeAxis::Memorization);
    assert_eq!(kn.full_id, node_id);

    // Test translation axis
    let node_id = "VERSE:1:1:translation";
    let kn = KnowledgeNode::parse(node_id);
    assert!(kn.is_some(), "Should parse translation node");
    let kn = kn.unwrap();
    assert_eq!(kn.base_node_id, "VERSE:1:1");
    assert_eq!(kn.axis, KnowledgeAxis::Translation);
    assert_eq!(kn.full_id, node_id);

    // Test tajweed axis
    let node_id = "WORD_INSTANCE:2:5:3:tajweed";
    let kn = KnowledgeNode::parse(node_id);
    assert!(kn.is_some(), "Should parse tajweed node");
    let kn = kn.unwrap();
    assert_eq!(kn.base_node_id, "WORD_INSTANCE:2:5:3");
    assert_eq!(kn.axis, KnowledgeAxis::Tajweed);

    // Test invalid axis (should return None)
    let node_id = "WORD_INSTANCE:1:1:1:invalid_axis";
    let kn = KnowledgeNode::parse(node_id);
    assert!(kn.is_none(), "Should not parse invalid axis");

    // Test non-knowledge node (no axis suffix)
    let node_id = "WORD_INSTANCE:1:1:1";
    let kn = KnowledgeNode::parse(node_id);
    assert!(kn.is_none(), "Should not parse non-knowledge node");
}

#[tokio::test]
async fn test_knowledge_node_construction() {
    // Test constructing knowledge nodes
    let kn = KnowledgeNode::new(
        "WORD_INSTANCE:1:1:1".to_string(),
        KnowledgeAxis::Memorization,
    );
    assert_eq!(kn.base_node_id, "WORD_INSTANCE:1:1:1");
    assert_eq!(kn.axis, KnowledgeAxis::Memorization);
    assert_eq!(kn.full_id, "WORD_INSTANCE:1:1:1:memorization");

    let kn = KnowledgeNode::new(
        "VERSE:2:255".to_string(),
        KnowledgeAxis::Translation,
    );
    assert_eq!(kn.base_node_id, "VERSE:2:255");
    assert_eq!(kn.axis, KnowledgeAxis::Translation);
    assert_eq!(kn.full_id, "VERSE:2:255:translation");
}

#[tokio::test]
async fn test_node_with_knowledge_axis() {
    // Test that Node struct correctly stores knowledge_node info
    let node = Node {
        id: "WORD_INSTANCE:1:1:1:memorization".to_string(),
        node_type: NodeType::Knowledge,
        knowledge_node: Some(KnowledgeNode::new(
            "WORD_INSTANCE:1:1:1".to_string(),
            KnowledgeAxis::Memorization,
        )),
    };

    assert_eq!(node.node_type, NodeType::Knowledge);
    assert!(node.knowledge_node.is_some());
    let kn = node.knowledge_node.unwrap();
    assert_eq!(kn.axis, KnowledgeAxis::Memorization);
    assert_eq!(kn.base_node_id, "WORD_INSTANCE:1:1:1");
}

#[tokio::test]
async fn test_knowledge_axis_to_string() {
    assert_eq!(KnowledgeAxis::Memorization.as_str(), "memorization");
    assert_eq!(KnowledgeAxis::Translation.as_str(), "translation");
    assert_eq!(KnowledgeAxis::Tafsir.as_str(), "tafsir");
    assert_eq!(KnowledgeAxis::Tajweed.as_str(), "tajweed");
    assert_eq!(KnowledgeAxis::ContextualMemorization.as_str(), "contextual_memorization");
    assert_eq!(KnowledgeAxis::Meaning.as_str(), "meaning");
}

#[tokio::test]
async fn test_knowledge_axis_from_string() {
    assert_eq!(
        KnowledgeAxis::from_str("memorization"),
        Some(KnowledgeAxis::Memorization)
    );
    assert_eq!(
        KnowledgeAxis::from_str("translation"),
        Some(KnowledgeAxis::Translation)
    );
    assert_eq!(
        KnowledgeAxis::from_str("tafsir"),
        Some(KnowledgeAxis::Tafsir)
    );
    assert_eq!(
        KnowledgeAxis::from_str("tajweed"),
        Some(KnowledgeAxis::Tajweed)
    );
    assert_eq!(
        KnowledgeAxis::from_str("contextual_memorization"),
        Some(KnowledgeAxis::ContextualMemorization)
    );
    assert_eq!(
        KnowledgeAxis::from_str("meaning"),
        Some(KnowledgeAxis::Meaning)
    );
    assert_eq!(KnowledgeAxis::from_str("invalid"), None);
}
