use anyhow::Result;
use iqrah_core::{import_translators_from_json, ContentRepository};
use iqrah_storage::{
    content::{init_content_db, node_registry::NodeRegistry, SqliteContentRepository},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct Language {
    code: String,
    english_name: String,
    native_name: String,
    direction: String,
}

#[derive(Debug, Deserialize)]
struct Translator {
    id: i32,
    slug: String,
    full_name: String,
    language_code: String,
    description: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PreferredTranslatorResponse {
    #[allow(dead_code)] // Part of API response
    user_id: String,
    preferred_translator_id: i32,
}

#[derive(Debug, Serialize)]
struct SetTranslatorRequest {
    translator_id: i32,
}

#[derive(Debug, Deserialize)]
struct SetTranslatorResponse {
    user_id: String,
    preferred_translator_id: i32,
    translator_name: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TranslationResponse {
    #[allow(dead_code)] // Part of API response
    verse_key: String,
    #[allow(dead_code)] // Part of API response
    translator_id: i32,
    translation: String,
}

/// List all available languages
pub async fn list_languages(server_url: &str) -> Result<()> {
    let url = format!("{}/languages", server_url);
    let response: Vec<Language> = reqwest::get(&url).await?.json().await?;

    println!("📚 Available Languages:");
    println!();
    for lang in response {
        println!(
            "  {} - {} ({}) [{}]",
            lang.code, lang.english_name, lang.native_name, lang.direction
        );
    }

    Ok(())
}

/// List all translators for a specific language
pub async fn list_translators(server_url: &str, language_code: &str) -> Result<()> {
    let url = format!("{}/translators/{}", server_url, language_code);
    let response: Vec<Translator> = reqwest::get(&url).await?.json().await?;

    println!("📖 Translators for language '{}':", language_code);
    println!();
    for translator in response {
        println!(
            "  [{}] {} ({})",
            translator.id, translator.full_name, translator.slug
        );
        if let Some(desc) = translator.description {
            println!("      {}", desc);
        }
        if let Some(license) = translator.license {
            println!("      License: {}", license);
        }
        println!();
    }

    Ok(())
}

/// Get translator details by ID
pub async fn get_translator(server_url: &str, translator_id: i32) -> Result<()> {
    let url = format!("{}/translator/{}", server_url, translator_id);
    let translator: Translator = reqwest::get(&url).await?.json().await?;

    println!("📖 Translator Details:");
    println!();
    println!("  ID:       {}", translator.id);
    println!("  Name:     {}", translator.full_name);
    println!("  Slug:     {}", translator.slug);
    println!("  Language: {}", translator.language_code);
    if let Some(desc) = translator.description {
        println!("  Description: {}", desc);
    }
    if let Some(license) = translator.license {
        println!("  License: {}", license);
    }

    Ok(())
}

/// Get user's preferred translator
pub async fn get_preferred(server_url: &str, user_id: &str) -> Result<()> {
    let url = format!("{}/users/{}/settings/translator", server_url, user_id);
    let response: PreferredTranslatorResponse = reqwest::get(&url).await?.json().await?;

    println!("👤 User '{}' Preferred Translator:", user_id);
    println!();
    println!("  Translator ID: {}", response.preferred_translator_id);

    Ok(())
}

/// Set user's preferred translator
pub async fn set_preferred(server_url: &str, user_id: &str, translator_id: i32) -> Result<()> {
    let url = format!("{}/users/{}/settings/translator", server_url, user_id);
    let client = reqwest::Client::new();
    let payload = SetTranslatorRequest { translator_id };

    let response: SetTranslatorResponse = client
        .post(&url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    println!("✅ {}", response.message);
    println!();
    println!("  User:       {}", response.user_id);
    println!(
        "  Translator: {} (ID: {})",
        response.translator_name, response.preferred_translator_id
    );

    Ok(())
}

/// Get verse translation for a specific translator
pub async fn get_translation(server_url: &str, verse_key: &str, translator_id: i32) -> Result<()> {
    let url = format!(
        "{}/verses/{}/translations/{}",
        server_url, verse_key, translator_id
    );
    let response: TranslationResponse = reqwest::get(&url).await?.json().await?;

    println!("📝 Verse {} (Translator ID: {}):", verse_key, translator_id);
    println!();
    println!("  {}", response.translation);

    Ok(())
}

/// Import translators from JSON file (direct database access, no server required)
pub async fn import_translators(metadata_file: &str, translations_base: &str) -> Result<()> {
    println!("📥 Importing translators from: {}", metadata_file);
    println!("   Translations base path: {}", translations_base);
    println!();

    // Get database path from environment or use default
    let content_db_path =
        std::env::var("CONTENT_DB_PATH").unwrap_or_else(|_| "data/content.db".to_string());

    println!("   Using content database: {}", content_db_path);
    println!();

    // Initialize database
    let content_pool = init_content_db(&content_db_path).await?;
    let registry = Arc::new(NodeRegistry::new(content_pool.clone()));
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool, registry));

    // Import translators
    let stats = import_translators_from_json(
        content_repo,
        Path::new(metadata_file),
        Path::new(translations_base),
    )
    .await?;

    // Print results
    println!("✅ Import Complete!");
    println!();
    println!("   Translators imported: {}", stats.translators_imported);
    println!("   Translations imported: {}", stats.translations_imported);

    if !stats.errors.is_empty() {
        println!();
        println!("⚠️  Errors encountered:");
        for error in &stats.errors {
            println!("   - {}", error);
        }
    }

    Ok(())
}
