use iqrah_core::domain::node_id as nid;
use iqrah_core::{ContentRepository, NodeType};
use iqrah_storage::{init_content_db, content::node_registry::NodeRegistry, SqliteContentRepository};
use std::sync::Arc;

async fn setup() -> (SqliteContentRepository, Arc<NodeRegistry>) {
    let pool = init_content_db(":memory:").await.unwrap();
    let registry = Arc::new(NodeRegistry::new(pool.clone()));
    registry.load_all().await.unwrap();
    let repo = SqliteContentRepository::new(pool.clone(), registry.clone());
    (repo, registry)
}

#[tokio::test]
async fn test_get_node_refactor() {
    let (repo, _) = setup().await;

    // Test Verse Node
    let verse_id = nid::verse(1, 1); // "VERSE:1:1"
    let node = repo.get_node(&verse_id).await.unwrap();
    assert!(node.is_some(), "Verse 1:1 should exist");
    let node = node.unwrap();
    assert_eq!(node.id, verse_id);
    assert!(matches!(node.node_type, NodeType::Verse));

    // Test Chapter Node
    let chapter_id = nid::chapter(1); // "CHAPTER:1"
    let node = repo.get_node(&chapter_id).await.unwrap();
    assert!(node.is_some(), "Chapter 1 should exist");
    let node = node.unwrap();
    assert_eq!(node.id, chapter_id);
    assert!(matches!(node.node_type, NodeType::Chapter));

    // Test Word Node
    let words = repo
        .get_words_in_ayahs(std::slice::from_ref(&verse_id))
        .await
        .unwrap();
    assert!(!words.is_empty());
    let word_node = &words[0];
    let word_id = &word_node.id;

    let node = repo.get_node(word_id).await.unwrap();
    assert!(node.is_some());
    let node = node.unwrap();
    assert_eq!(node.id, *word_id);
    assert!(matches!(node.node_type, NodeType::Word));
}

#[tokio::test]
async fn test_node_exists_refactor() {
    let (repo, _) = setup().await;

    assert!(repo.node_exists(&nid::verse(1, 1)).await.unwrap());
    assert!(repo.node_exists(&nid::chapter(1)).await.unwrap());
    assert!(!repo.node_exists("VERSE:2:1").await.unwrap());
}

#[tokio::test]
async fn test_get_quran_text_refactor() {
    let (repo, _) = setup().await;

    let text = repo.get_quran_text(&nid::verse(1, 1)).await.unwrap();
    assert!(text.is_some());
    assert!(text.unwrap().contains("بِسْمِ"));
}

#[tokio::test]
async fn test_get_translation_refactor() {
    let (repo, _) = setup().await;

    let text = repo.get_translation(&nid::verse(1, 1), "en").await.unwrap();
    assert!(text.is_some());
}
