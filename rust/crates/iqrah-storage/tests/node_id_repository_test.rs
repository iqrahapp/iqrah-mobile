use iqrah_core::domain::node_id as nid;
use iqrah_core::{ContentRepository, NodeType};
use iqrah_storage::{init_content_db, SqliteContentRepository};

#[tokio::test]
async fn test_get_node_refactor() {
    let pool = init_content_db(":memory:").await.unwrap();
    let repo = SqliteContentRepository::new(pool.clone());

    // Test Verse Node
    let verse_id = nid::encode_verse(1, 1); // "VERSE:1:1"
    let node = repo.get_node(verse_id).await.unwrap();
    assert!(node.is_some(), "Verse 1:1 should exist");
    let node = node.unwrap();
    assert_eq!(node.id, verse_id);
    assert!(matches!(node.node_type, NodeType::Verse));

    // Test Chapter Node
    let chapter_id = nid::encode_chapter(1); // "CHAPTER:1"
    let node = repo.get_node(chapter_id).await.unwrap();
    assert!(node.is_some(), "Chapter 1 should exist");
    let node = node.unwrap();
    assert_eq!(node.id, chapter_id);
    assert!(matches!(node.node_type, NodeType::Chapter));

    // Test Word Node
    // We need to find a valid word ID first
    let words = repo
        .get_words_in_ayahs(std::slice::from_ref(&verse_id))
        .await
        .unwrap();
    assert!(!words.is_empty());
    let word_node = &words[0];
    let word_id = &word_node.id; // Should be "WORD:..."

    let node = repo.get_node(*word_id).await.unwrap();
    assert!(node.is_some());
    let node = node.unwrap();
    assert_eq!(node.id, *word_id);
    assert!(matches!(node.node_type, NodeType::Word));

    // Test Word Instance Node
    // "WORD_INSTANCE:1:1:1"
    let instance_id = nid::encode_word_instance(1, 1, 1);
    let node = repo.get_node(instance_id).await.unwrap();
    assert!(node.is_some());
    let node = node.unwrap();
    assert_eq!(node.id, instance_id);
    assert!(matches!(node.node_type, NodeType::WordInstance));

    // Test Chapter Node - verify chapter 1 exists
    assert!(repo.node_exists(nid::encode_chapter(1)).await.unwrap());

    // Note: Knowledge nodes are not created in migrations, so we skip testing them
}

#[tokio::test]
async fn test_node_exists_refactor() {
    let pool = init_content_db(":memory:").await.unwrap();
    let repo = SqliteContentRepository::new(pool);

    assert!(repo.node_exists(nid::encode_verse(1, 1)).await.unwrap());
    assert!(repo.node_exists(nid::encode_chapter(1)).await.unwrap());
    assert!(repo.node_exists(nid::encode_verse(2, 1)).await.unwrap()); // Chapter 2 exists from migrations
    assert!(!repo.node_exists(nid::encode_verse(3, 250)).await.unwrap()); // Non-existent verse (chapter 3 max 200 verses)
}

#[tokio::test]
async fn test_get_quran_text_refactor() {
    let pool = init_content_db(":memory:").await.unwrap();
    let repo = SqliteContentRepository::new(pool);

    let text = repo.get_quran_text(nid::encode_verse(1, 1)).await.unwrap();
    assert!(text.is_some());
    assert!(text.unwrap().contains("بِسْمِ"));
}

#[tokio::test]
async fn test_get_translation_refactor() {
    let pool = init_content_db(":memory:").await.unwrap();
    let repo = SqliteContentRepository::new(pool);

    let text = repo
        .get_translation(nid::encode_verse(1, 1), "en")
        .await
        .unwrap();
    assert!(text.is_some());
}
