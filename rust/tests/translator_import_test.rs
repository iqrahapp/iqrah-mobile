use anyhow::Result;
use iqrah_core::{import_translators_from_json, ContentRepository};
use iqrah_storage::{
    content::{init_content_db, SqliteContentRepository},
    user::{init_user_db, SqliteUserRepository},
};
use std::path::Path;
use std::sync::Arc;

#[tokio::test]
async fn test_translator_import() -> Result<()> {
    // Initialize in-memory databases for testing
    let content_pool = init_content_db(":memory:").await?;
    let _user_pool = init_user_db(":memory:").await?;

    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));

    // Note: This test requires the actual translation files
    // For CI/CD, we would either:
    // 1. Include minimal test files
    // 2. Mock the file system
    // 3. Skip this test in environments without data files

    let metadata_path = Path::new("research_and_dev/data/translations/translator-metadata-example.json");
    let translations_base = Path::new("research_and_dev/data/translations");

    // Skip test if files don't exist (e.g., in CI)
    if !metadata_path.exists() {
        eprintln!("Skipping test: metadata file not found");
        return Ok(());
    }

    // Run the import
    let stats = import_translators_from_json(
        Arc::clone(&content_repo),
        metadata_path,
        translations_base,
    )
    .await?;

    // Verify stats
    println!("Import stats: {:#?}", stats);
    assert!(stats.translators_imported > 0, "Should import at least one translator");
    assert!(stats.translations_imported > 0, "Should import at least one translation");

    // Verify translators were created
    let languages = content_repo.get_languages().await?;
    assert!(!languages.is_empty(), "Should have languages");

    let en_translators = content_repo.get_translators_for_language("en").await?;
    assert!(!en_translators.is_empty(), "Should have English translators");

    // Verify a specific translator
    let sahih_intl = content_repo.get_translator_by_slug("sahih-intl").await?;
    assert!(sahih_intl.is_some(), "Sahih International should exist");
    let sahih_intl = sahih_intl.unwrap();
    assert_eq!(sahih_intl.full_name, "Sahih International");
    assert_eq!(sahih_intl.language_code, "en");

    // Verify translations were imported
    let translation_1_1 = content_repo
        .get_verse_translation("1:1", sahih_intl.id)
        .await?;
    assert!(translation_1_1.is_some(), "Should have translation for 1:1");

    let translation_text = translation_1_1.unwrap();
    assert!(translation_text.contains("Allāh") || translation_text.contains("Allah"),
        "Translation should contain 'Allah'");

    // Verify footnotes were extracted (main text should not contain [[]])
    assert!(!translation_text.contains("[["), "Footnotes should be extracted from main text");

    println!("✅ Translator import test passed!");
    Ok(())
}

#[tokio::test]
async fn test_translator_insert_and_query() -> Result<()> {
    // Initialize in-memory database
    let content_pool = init_content_db(":memory:").await?;
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));

    // Insert a test translator
    let translator_id = content_repo
        .insert_translator(
            "test-translator",
            "Test Translator",
            "en",
            Some("A test translation"),
            Some("Test Author"),
            Some("CC0"),
            Some("https://example.com"),
            Some("1.0"),
        )
        .await?;

    assert!(translator_id > 0, "Should return valid translator ID");

    // Query it back
    let translator = content_repo.get_translator(translator_id).await?;
    assert!(translator.is_some(), "Should find the translator");

    let translator = translator.unwrap();
    assert_eq!(translator.slug, "test-translator");
    assert_eq!(translator.full_name, "Test Translator");
    assert_eq!(translator.language_code, "en");

    // Query by slug
    let by_slug = content_repo.get_translator_by_slug("test-translator").await?;
    assert!(by_slug.is_some(), "Should find by slug");
    assert_eq!(by_slug.unwrap().id, translator_id);

    // Insert a verse translation
    content_repo
        .insert_verse_translation("1:1", translator_id, "In the name of God", Some("[\"Footnote 1\"]"))
        .await?;

    // Query it back
    let translation = content_repo.get_verse_translation("1:1", translator_id).await?;
    assert!(translation.is_some(), "Should find translation");
    assert_eq!(translation.unwrap(), "In the name of God");

    // Update it
    content_repo
        .insert_verse_translation("1:1", translator_id, "In the name of Allah", Some("[\"Updated footnote\"]"))
        .await?;

    // Verify update
    let updated = content_repo.get_verse_translation("1:1", translator_id).await?;
    assert_eq!(updated.unwrap(), "In the name of Allah");

    println!("✅ Translator insert and query test passed!");
    Ok(())
}
