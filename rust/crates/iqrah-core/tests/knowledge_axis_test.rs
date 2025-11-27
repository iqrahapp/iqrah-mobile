// tests/knowledge_axis_test.rs
// Integration test for Phase 4: Knowledge Axis support

use iqrah_core::domain::node_id;
use iqrah_core::{KnowledgeAxis, KnowledgeNode};

#[tokio::test]
async fn test_knowledge_node_stability() {
    // 1. Define inputs
    let base_ukey = "VERSE:2:255";
    let axis = KnowledgeAxis::Translation;
    let expected_ukey = "VERSE:2:255:translation";

    // 2. Parse UKEY -> i64 ID
    let id = node_id::from_ukey(expected_ukey).expect("Should parse from ukey");

    // 3. Decode i64 ID -> components
    let (decoded_base_id, decoded_axis) =
        node_id::decode_knowledge_id(id).expect("Should decode id");
    let decoded_base_ukey = node_id::to_ukey(decoded_base_id).expect("Should decode base id");

    // 4. Verify components match original inputs
    assert_eq!(decoded_base_ukey, base_ukey);
    assert_eq!(decoded_axis, axis);

    // 5. Decode i64 ID -> UKEY
    let roundtrip_ukey = node_id::to_ukey(id).expect("Should convert to ukey");

    // 6. Verify full roundtrip
    assert_eq!(roundtrip_ukey, expected_ukey);
}

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

    let kn = KnowledgeNode::new("VERSE:2:255".to_string(), KnowledgeAxis::Translation);
    assert_eq!(kn.base_node_id, "VERSE:2:255");
    assert_eq!(kn.axis, KnowledgeAxis::Translation);
    assert_eq!(kn.full_id, "VERSE:2:255:translation");
}

#[tokio::test]
async fn test_knowledge_axis_to_string() {
    assert_eq!(KnowledgeAxis::Memorization.as_ref(), "memorization");
    assert_eq!(KnowledgeAxis::Translation.as_ref(), "translation");
    assert_eq!(KnowledgeAxis::Tafsir.as_ref(), "tafsir");
    assert_eq!(KnowledgeAxis::Tajweed.as_ref(), "tajweed");
    assert_eq!(
        KnowledgeAxis::ContextualMemorization.as_ref(),
        "contextual_memorization"
    );
    assert_eq!(KnowledgeAxis::Meaning.as_ref(), "meaning");
}

#[tokio::test]
async fn test_knowledge_axis_from_string() {
    assert_eq!(
        KnowledgeAxis::parse("memorization"),
        Ok(KnowledgeAxis::Memorization)
    );
    assert_eq!(
        KnowledgeAxis::parse("translation"),
        Ok(KnowledgeAxis::Translation)
    );
    assert_eq!(KnowledgeAxis::parse("tafsir"), Ok(KnowledgeAxis::Tafsir));
    assert_eq!(KnowledgeAxis::parse("tajweed"), Ok(KnowledgeAxis::Tajweed));
    assert_eq!(
        KnowledgeAxis::parse("contextual_memorization"),
        Ok(KnowledgeAxis::ContextualMemorization)
    );
    assert_eq!(KnowledgeAxis::parse("meaning"), Ok(KnowledgeAxis::Meaning));
    assert!(KnowledgeAxis::parse("invalid").is_err());
}
