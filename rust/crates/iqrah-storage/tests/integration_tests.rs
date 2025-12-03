use chrono::Utc;
use iqrah_core::domain::node_id as nid;
use iqrah_core::{ContentRepository, MemoryState, UserRepository};
use iqrah_storage::{
    create_content_repository, init_test_content_db, init_user_db, SqliteUserRepository,
};
use sqlx::Row;

#[tokio::test]
async fn test_content_db_initialization() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Verify v2 schema was created - test with verse queries instead of nodes
    let verse = repo.get_verse("test").await.unwrap();
    assert!(verse.is_none(), "Should return None for non-existent verse");

    // Verify sample data exists
    let chapter = repo.get_chapter(1).await.unwrap();
    assert!(
        chapter.is_some(),
        "Al-Fatihah should exist from sample data"
    );
}

#[tokio::test]
async fn test_user_db_initialization_and_migrations() {
    let pool = init_user_db(":memory:").await.unwrap();

    // Check that migrations ran successfully
    let row = sqlx::query("SELECT version FROM schema_version")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(
        row.is_some(),
        "Migration v2 should have created schema_version table"
    );

    let version: String = row.unwrap().get("version");
    assert_eq!(
        version, "2.0.0",
        "Schema version should be 2.0.0 after migrations"
    );
}

#[tokio::test]
async fn test_content_repository_crud() {
    // Test v2 schema CRUD operations using sample data
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Test verse queries (v2 schema)
    let verse = repo.get_verse("1:1").await.unwrap();
    assert!(verse.is_some(), "Verse 1:1 should exist from sample data");
    let verse = verse.unwrap();
    assert_eq!(verse.key, "1:1");
    assert_eq!(verse.chapter_number, 1);
    assert_eq!(verse.verse_number, 1);

    // Test word queries (v2 schema)
    let words = repo.get_words_for_verse("1:1").await.unwrap();
    assert_eq!(words.len(), 4, "Bismillah has 4 words");
    assert_eq!(words[0].text_uthmani, "بِسْمِ");

    // Test non-existent verse
    let verse = repo.get_verse("nonexistent").await.unwrap();
    assert!(verse.is_none());
}

#[tokio::test]
async fn test_user_repository_memory_states() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Create a memory state
    let state = MemoryState {
        user_id: "user1".to_string(),
        node_id: 100, // node_id is i64
        stability: 1.5,
        difficulty: 5.0,
        energy: 0.7,
        last_reviewed: Utc::now(),
        due_at: Utc::now(),
        review_count: 3,
    };

    // Save it
    repo.save_memory_state(&state).await.unwrap();

    // Retrieve it
    let retrieved = repo.get_memory_state("user1", 100).await.unwrap();
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.user_id, "user1");
    assert_eq!(retrieved.node_id, 100);
    assert_eq!(retrieved.stability, 1.5);
    assert_eq!(retrieved.difficulty, 5.0);
    assert_eq!(retrieved.energy, 0.7);
    assert_eq!(retrieved.review_count, 3);

    // Update it
    let mut updated = state.clone();
    updated.energy = 0.9;
    updated.review_count = 4;

    repo.save_memory_state(&updated).await.unwrap();

    let retrieved = repo.get_memory_state("user1", 100).await.unwrap().unwrap();
    assert_eq!(retrieved.energy, 0.9);
    assert_eq!(retrieved.review_count, 4);
}

#[tokio::test]
async fn test_user_repository_get_due_states() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    let now = Utc::now();

    // Create overdue state
    let overdue = MemoryState {
        user_id: "user1".to_string(),
        node_id: 100,
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: now,
        due_at: now - chrono::Duration::hours(1), // Overdue
        review_count: 1,
    };

    // Create future state
    let future = MemoryState {
        user_id: "user1".to_string(),
        node_id: 101,
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: now,
        due_at: now + chrono::Duration::hours(1), // Not due yet
        review_count: 1,
    };

    repo.save_memory_state(&overdue).await.unwrap();
    repo.save_memory_state(&future).await.unwrap();

    // Get due states
    let due = repo.get_due_states("user1", now, 10).await.unwrap();

    assert_eq!(due.len(), 1, "Should only return overdue items");
    assert_eq!(due[0].node_id, 100);
}

#[tokio::test]
async fn test_user_repository_stats() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Set stat
    repo.set_stat("reviews_today", "42").await.unwrap();

    // Get stat
    let value = repo.get_stat("reviews_today").await.unwrap();
    assert_eq!(value, Some("42".to_string()));

    // Update stat
    repo.set_stat("reviews_today", "43").await.unwrap();
    let value = repo.get_stat("reviews_today").await.unwrap();
    assert_eq!(value, Some("43".to_string()));

    // Non-existent stat
    let value = repo.get_stat("nonexistent").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_user_repository_session_state() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    let nodes = vec![100, 101, 102];

    // Save session
    repo.save_session_state(&nodes).await.unwrap();

    // Retrieve session
    let retrieved = repo.get_session_state().await.unwrap();
    assert_eq!(retrieved.len(), 3);
    assert_eq!(retrieved[0], 100);
    assert_eq!(retrieved[1], 101);
    assert_eq!(retrieved[2], 102);

    // Clear session
    repo.clear_session_state().await.unwrap();
    let retrieved = repo.get_session_state().await.unwrap();
    assert_eq!(retrieved.len(), 0);
}

#[tokio::test]
async fn test_update_energy() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Create initial state
    let state = MemoryState {
        user_id: "user1".to_string(),
        node_id: 100,
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: Utc::now(),
        due_at: Utc::now(),
        review_count: 1,
    };

    repo.save_memory_state(&state).await.unwrap();

    // Update just the energy
    repo.update_energy("user1", 100, 0.8).await.unwrap();

    // Verify energy was updated
    let updated = repo.get_memory_state("user1", 100).await.unwrap().unwrap();
    assert_eq!(updated.energy, 0.8);
    assert_eq!(updated.stability, 1.0); // Other fields unchanged
}

#[tokio::test]
async fn test_two_database_integration() {
    // This test demonstrates the two-database architecture working together with v2 schema

    // Initialize both databases
    let content_pool = init_test_content_db(":memory:").await.unwrap();
    let user_pool = init_user_db(":memory:").await.unwrap();

    // Create repositories
    let content_repo = create_content_repository(content_pool);
    let user_repo = SqliteUserRepository::new(user_pool);

    // Verify content.db has v2 sample data (verse from Al-Fatihah)
    let verse = content_repo.get_verse("1:1").await.unwrap();
    assert!(verse.is_some(), "Sample verse 1:1 should exist");

    // Create user progress for that verse
    let verse_id = nid::encode_verse(1, 1);
    let state = MemoryState::new_for_node("user1".to_string(), verse_id);
    user_repo.save_memory_state(&state).await.unwrap();

    // Verify user.db has the state
    let user_state = user_repo.get_memory_state("user1", verse_id).await.unwrap();
    assert!(user_state.is_some());

    // Verify schema_version table exists (migration v2 proof)
    let pool = init_user_db(":memory:").await.unwrap();
    let row = sqlx::query("SELECT version FROM schema_version")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(row.is_some(), "Migration v2 should have run");
}

// ============================================================================
// V2 Schema Tests
// ============================================================================

#[tokio::test]
async fn test_v2_chapter_queries() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Test get_chapter with sample data (Al-Fatihah from migration)
    let chapter = repo.get_chapter(1).await.unwrap();
    assert!(
        chapter.is_some(),
        "Chapter 1 (Al-Fatihah) should exist from sample data"
    );

    let chapter = chapter.unwrap();
    assert_eq!(chapter.number, 1);
    assert_eq!(chapter.name_arabic, "الفاتحة");
    assert_eq!(chapter.name_transliteration, "Al-Fatihah");
    assert_eq!(chapter.name_translation, "The Opening");
    assert_eq!(chapter.verse_count, 7);
    assert_eq!(chapter.revelation_place, Some("makkah".to_string()));

    // Test get_chapters
    let chapters = repo.get_chapters().await.unwrap();
    assert_eq!(
        chapters.len(),
        3,
        "Should have 3 chapters from sample data (Al-Fatihah, Al-Baqarah, Al-Imran)"
    );
    assert_eq!(chapters[0].number, 1);

    // Test non-existent chapter
    let chapter = repo.get_chapter(999).await.unwrap();
    assert!(chapter.is_none(), "Non-existent chapter should return None");
}

#[tokio::test]
async fn test_v2_verse_queries() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Test get_verse with sample data
    let verse = repo.get_verse("1:1").await.unwrap();
    assert!(verse.is_some(), "Verse 1:1 should exist from sample data");

    let verse = verse.unwrap();
    assert_eq!(verse.key, "1:1");
    assert_eq!(verse.chapter_number, 1);
    assert_eq!(verse.verse_number, 1);
    assert_eq!(verse.text_uthmani, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
    assert_eq!(
        verse.text_simple,
        Some("بسم الله الرحمن الرحيم".to_string())
    );
    assert_eq!(verse.juz, 1);
    assert_eq!(verse.page, 1);

    // Test get_verses_for_chapter
    let verses = repo.get_verses_for_chapter(1).await.unwrap();
    assert_eq!(verses.len(), 7, "Al-Fatihah has 7 verses");

    // Verify verses are ordered
    for (i, verse) in verses.iter().enumerate() {
        assert_eq!(verse.verse_number, (i + 1) as i32);
    }

    // Test non-existent verse
    let verse = repo.get_verse("999:999").await.unwrap();
    assert!(verse.is_none(), "Non-existent verse should return None");
}

#[tokio::test]
async fn test_v2_word_queries() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Test get_words_for_verse with sample data
    let words = repo.get_words_for_verse("1:1").await.unwrap();
    assert_eq!(words.len(), 4, "Bismillah has 4 words");

    // Verify words are ordered by position
    assert_eq!(words[0].position, 1);
    assert_eq!(words[0].text_uthmani, "بِسْمِ");
    assert_eq!(words[0].text_simple, Some("بسم".to_string()));
    assert_eq!(words[0].transliteration, Some("bismi".to_string()));

    assert_eq!(words[1].position, 2);
    assert_eq!(words[1].text_uthmani, "ٱللَّهِ");

    assert_eq!(words[2].position, 3);
    assert_eq!(words[3].position, 4);

    // Test get_word by ID
    let word_id = words[0].id;
    let word = repo.get_word(word_id).await.unwrap();
    assert!(word.is_some());
    let word = word.unwrap();
    assert_eq!(word.id, word_id);
    assert_eq!(word.text_uthmani, "بِسْمِ");

    // Test non-existent word
    let word = repo.get_word(99999).await.unwrap();
    assert!(word.is_none(), "Non-existent word should return None");
}

#[tokio::test]
async fn test_v2_language_queries() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Test get_languages
    let languages = repo.get_languages().await.unwrap();
    assert!(
        languages.len() >= 7,
        "Should have at least 7 sample languages"
    );

    // Verify English is present
    let en = languages.iter().find(|l| l.code == "en");
    assert!(en.is_some(), "English should be in sample data");
    let en = en.unwrap();
    assert_eq!(en.english_name, "English");
    assert_eq!(en.native_name, "English");
    assert_eq!(en.direction, "ltr");

    // Verify Arabic is present
    let ar = languages.iter().find(|l| l.code == "ar");
    assert!(ar.is_some(), "Arabic should be in sample data");
    let ar = ar.unwrap();
    assert_eq!(ar.english_name, "Arabic");
    assert_eq!(ar.native_name, "العربية");
    assert_eq!(ar.direction, "rtl");

    // Test get_language by code
    let lang = repo.get_language("en").await.unwrap();
    assert!(lang.is_some());
    assert_eq!(lang.unwrap().code, "en");

    // Test non-existent language
    let lang = repo.get_language("xyz").await.unwrap();
    assert!(lang.is_none());
}

#[tokio::test]
async fn test_v2_translator_queries() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Test get_translators_for_language
    let translators = repo.get_translators_for_language("en").await.unwrap();
    assert_eq!(
        translators.len(),
        5,
        "Should have 5 English translators from sample data"
    );

    // Verify Sahih International is present
    let sahih = translators.iter().find(|t| t.slug == "sahih-intl");
    assert!(sahih.is_some(), "Sahih International should be present");
    let sahih = sahih.unwrap();
    assert_eq!(sahih.full_name, "Sahih International");
    assert_eq!(sahih.language_code, "en");
    assert!(sahih.description.is_some());
    assert_eq!(sahih.license, Some("Public Domain".to_string()));

    // Test get_translator by ID
    let translator_id = sahih.id;
    let translator = repo.get_translator(translator_id).await.unwrap();
    assert!(translator.is_some());
    assert_eq!(translator.unwrap().slug, "sahih-intl");

    // Test get_translator_by_slug
    let translator = repo.get_translator_by_slug("yusuf-ali").await.unwrap();
    assert!(translator.is_some());
    let translator = translator.unwrap();
    assert_eq!(translator.full_name, "Abdullah Yusuf Ali");
    assert_eq!(translator.language_code, "en");

    // Test non-existent translator
    let translator = repo.get_translator_by_slug("nonexistent").await.unwrap();
    assert!(translator.is_none());
}

#[tokio::test]
async fn test_script_content_variants() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);
    let verse_id = nid::encode_verse(1, 1);

    let uthmani = repo.get_script_content(verse_id, "uthmani").await.unwrap();
    assert!(uthmani.is_some());
    assert!(uthmani.unwrap().contains("بِسْمِ"));

    let simple = repo.get_script_content(verse_id, "simple").await.unwrap();
    assert!(simple.is_some());
    assert!(simple.unwrap().contains("بسم الله"));
}

#[tokio::test]
async fn test_v2_translation_queries() {
    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // Get Sahih International translator ID
    let sahih = repo
        .get_translator_by_slug("sahih-intl")
        .await
        .unwrap()
        .unwrap();

    // Test get_verse_translation
    let translation = repo.get_verse_translation("1:1", sahih.id).await.unwrap();
    assert!(
        translation.is_some(),
        "Verse 1:1 should have Sahih International translation"
    );
    let translation = translation.unwrap();
    assert!(translation.contains("Allah"));
    assert!(translation.contains("Merciful"));

    // Test different translator
    let yusuf_ali = repo
        .get_translator_by_slug("yusuf-ali")
        .await
        .unwrap()
        .unwrap();
    let translation = repo
        .get_verse_translation("1:1", yusuf_ali.id)
        .await
        .unwrap();
    assert!(translation.is_some());
    let translation = translation.unwrap();
    assert!(translation.contains("God") || translation.contains("Allah"));

    // Test get_word_translation
    let words = repo.get_words_for_verse("1:1").await.unwrap();
    let first_word = &words[0];
    let word_translation = repo
        .get_word_translation(first_word.id, sahih.id)
        .await
        .unwrap();
    assert!(
        word_translation.is_some(),
        "First word should have translation"
    );
    assert_eq!(word_translation.unwrap(), "In the name");

    // Test non-existent translation
    let translation = repo
        .get_verse_translation("999:999", sahih.id)
        .await
        .unwrap();
    assert!(translation.is_none());
}

#[tokio::test]
async fn test_v2_full_verse_retrieval() {
    // This test demonstrates the complete v2 query flow:
    // Chapter -> Verses -> Words -> Translations

    let pool = init_test_content_db(":memory:").await.unwrap();
    let repo = create_content_repository(pool);

    // 1. Get the chapter
    let chapter = repo.get_chapter(1).await.unwrap().unwrap();
    assert_eq!(chapter.name_arabic, "الفاتحة");

    // 2. Get all verses for the chapter
    let verses = repo.get_verses_for_chapter(chapter.number).await.unwrap();
    assert_eq!(verses.len(), 7);

    // 3. Get words for the first verse
    let first_verse = &verses[0];
    let words = repo.get_words_for_verse(&first_verse.key).await.unwrap();
    assert_eq!(words.len(), 4);

    // 4. Get translations for the first verse from all translators
    let translators = repo.get_translators_for_language("en").await.unwrap();
    assert_eq!(translators.len(), 5);

    let mut translations = Vec::new();
    for translator in &translators {
        if let Some(translation) = repo
            .get_verse_translation(&first_verse.key, translator.id)
            .await
            .unwrap()
        {
            translations.push((translator.full_name.clone(), translation));
        }
    }
    assert_eq!(
        translations.len(),
        5,
        "All 5 translators should have translation for 1:1"
    );

    // Verify different translations
    let sahih_translation = translations
        .iter()
        .find(|(name, _)| name.contains("Sahih"))
        .map(|(_, trans)| trans);
    assert!(sahih_translation.is_some());
    assert!(sahih_translation.unwrap().contains("Entirely Merciful"));

    // 5. Get word-by-word translation for first word
    let first_word = &words[0];
    let sahih = translators.iter().find(|t| t.slug == "sahih-intl").unwrap();
    let word_translation = repo
        .get_word_translation(first_word.id, sahih.id)
        .await
        .unwrap();
    assert_eq!(word_translation, Some("In the name".to_string()));
}

#[tokio::test]
async fn test_v2_schema_version_validation() {
    // Test that schema version is enforced
    let pool = init_test_content_db(":memory:").await.unwrap();

    // Verify schema version is 2.0.0
    let version: String = sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        version, "2.1.0",
        "Schema version should be 2.1.0 for v2 database"
    );
}
