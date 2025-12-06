use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::ports::ContentRepository;

/// Metadata for a single translator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorMetadata {
    pub slug: String,
    pub full_name: String,
    pub language_code: String,
    pub description: Option<String>,
    pub copyright_holder: Option<String>,
    pub license: Option<String>,
    pub website: Option<String>,
    pub version: Option<String>,
    pub translation_file: String,
}

/// Root structure for translator import file
#[derive(Debug, Serialize, Deserialize)]
pub struct TranslatorImportFile {
    pub translators: Vec<TranslatorMetadata>,
}

/// A single verse translation
#[derive(Debug, Serialize, Deserialize)]
pub struct VerseTranslationEntry {
    /// Translation text (may include [[footnotes]])
    pub t: String,
}

/// Statistics from translator import
#[derive(Debug)]
pub struct TranslatorImportStats {
    pub translators_imported: usize,
    pub translations_imported: usize,
    pub errors: Vec<String>,
}

/// Import translators and their translations from JSON files
///
/// # Arguments
/// * `content_repo` - Content repository to write to
/// * `metadata_path` - Path to translator metadata JSON file
/// * `translations_base_path` - Base directory for translation files
pub async fn import_translators_from_json(
    content_repo: Arc<dyn ContentRepository>,
    metadata_path: &Path,
    translations_base_path: &Path,
) -> Result<TranslatorImportStats> {
    let mut stats = TranslatorImportStats {
        translators_imported: 0,
        translations_imported: 0,
        errors: Vec::new(),
    };

    // Read metadata file
    let metadata_content = std::fs::read_to_string(metadata_path)
        .context("Failed to read translator metadata file")?;

    let import_file: TranslatorImportFile = serde_json::from_str(&metadata_content)
        .context("Failed to parse translator metadata JSON")?;

    tracing::info!(
        "Found {} translators to import",
        import_file.translators.len()
    );

    for translator_meta in import_file.translators {
        match import_single_translator(&content_repo, &translator_meta, translations_base_path)
            .await
        {
            Ok(translation_count) => {
                stats.translators_imported += 1;
                stats.translations_imported += translation_count;
                tracing::info!(
                    "✓ Imported translator '{}' with {} translations",
                    translator_meta.full_name,
                    translation_count
                );
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to import translator '{}': {}",
                    translator_meta.full_name, e
                );
                tracing::error!("{}", error_msg);
                stats.errors.push(error_msg);
            }
        }
    }

    Ok(stats)
}

/// Import a single translator and their translations
async fn import_single_translator(
    content_repo: &Arc<dyn ContentRepository>,
    metadata: &TranslatorMetadata,
    translations_base_path: &Path,
) -> Result<usize> {
    // First, check if language exists
    let language = content_repo
        .get_language(&metadata.language_code)
        .await?
        .context(format!(
            "Language '{}' not found. Please import languages first.",
            metadata.language_code
        ))?;

    tracing::debug!(
        "Language '{}' found: {}",
        language.code,
        language.english_name
    );

    // Check if translator already exists by slug
    let existing = content_repo.get_translator_by_slug(&metadata.slug).await?;

    let translator_id = if let Some(existing_translator) = existing {
        tracing::info!(
            "Translator '{}' already exists (ID: {}), skipping creation",
            metadata.slug,
            existing_translator.id
        );
        existing_translator.id
    } else {
        // Create new translator
        tracing::info!("Creating new translator: {}", metadata.full_name);

        content_repo
            .insert_translator(
                &metadata.slug,
                &metadata.full_name,
                &metadata.language_code,
                metadata.description.clone(),
                metadata.copyright_holder.clone(),
                metadata.license.clone(),
                metadata.website.clone(),
                metadata.version.clone(),
                None, // package_id - built-in translators don't belong to packages
            )
            .await?
    };

    // Load translation file
    let translation_file_path = translations_base_path.join(&metadata.translation_file);
    let translation_content = std::fs::read_to_string(&translation_file_path).context(format!(
        "Failed to read translation file: {}",
        translation_file_path.display()
    ))?;

    let translations: HashMap<String, VerseTranslationEntry> =
        serde_json::from_str(&translation_content).context(format!(
            "Failed to parse translation JSON: {}",
            translation_file_path.display()
        ))?;

    tracing::info!(
        "Loaded {} verse translations from {}",
        translations.len(),
        metadata.translation_file
    );

    // Import translations
    let mut imported_count = 0;
    for (verse_key, entry) in translations {
        // Extract footnotes if present and store main text
        let (main_text, footnotes) = extract_footnotes(&entry.t);

        // Insert translation
        content_repo
            .insert_verse_translation(&verse_key, translator_id, &main_text, footnotes)
            .await?;

        imported_count += 1;

        // Log progress every 100 verses
        if imported_count % 100 == 0 {
            tracing::debug!("Imported {} translations...", imported_count);
        }
    }

    Ok(imported_count)
}

/// Extract footnotes from translation text
///
/// Footnotes are marked with [[footnote text]] in the translation
/// Returns (main_text, footnotes_json)
fn extract_footnotes(text: &str) -> (String, Option<String>) {
    let mut footnotes = Vec::new();
    let mut main_text = String::new();
    let mut chars = text.chars().peekable();
    let mut in_footnote = false;
    let mut current_footnote = String::new();

    while let Some(ch) = chars.next() {
        if ch == '[' && chars.peek() == Some(&'[') {
            // Start of footnote
            chars.next(); // consume second '['
            in_footnote = true;
            current_footnote.clear();
        } else if ch == ']' && chars.peek() == Some(&']') {
            // End of footnote
            chars.next(); // consume second ']'
            in_footnote = false;
            if !current_footnote.is_empty() {
                footnotes.push(current_footnote.clone());
            }
        } else if in_footnote {
            current_footnote.push(ch);
        } else {
            main_text.push(ch);
        }
    }

    let footnotes_json = if footnotes.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&footnotes).unwrap_or_default())
    };

    (main_text, footnotes_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_footnotes() {
        let text = "In the name of Allāh,[[Footnote 1]] the Merciful.[[Footnote 2]]";
        let (main, footnotes) = extract_footnotes(text);

        assert_eq!(main, "In the name of Allāh, the Merciful.");
        assert!(footnotes.is_some());

        let footnotes_vec: Vec<String> = serde_json::from_str(&footnotes.unwrap()).unwrap();
        assert_eq!(footnotes_vec.len(), 2);
        assert_eq!(footnotes_vec[0], "Footnote 1");
        assert_eq!(footnotes_vec[1], "Footnote 2");
    }

    #[test]
    fn test_extract_no_footnotes() {
        let text = "Simple translation text.";
        let (main, footnotes) = extract_footnotes(text);

        assert_eq!(main, "Simple translation text.");
        assert!(footnotes.is_none());
    }
}
