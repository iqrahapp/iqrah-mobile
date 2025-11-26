use crate::{ContentPackage, ContentRepository, InstalledPackage, PackageType};
use anyhow::{bail, Context, Result};
use sha2::Digest;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Service for managing downloadable content packages
pub struct PackageService {
    content_repo: Arc<dyn ContentRepository>,
    http_client: reqwest::Client,
}

impl PackageService {
    pub fn new(content_repo: Arc<dyn ContentRepository>) -> Self {
        Self {
            content_repo,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get all available packages (optionally filtered)
    pub async fn get_available_packages(
        &self,
        package_type: Option<PackageType>,
        language_code: Option<&str>,
    ) -> Result<Vec<ContentPackage>> {
        self.content_repo
            .get_available_packages(package_type, language_code)
            .await
    }

    /// Get a specific package by ID
    pub async fn get_package(&self, package_id: &str) -> Result<Option<ContentPackage>> {
        self.content_repo.get_package(package_id).await
    }

    /// Download a package from its download_url
    pub async fn download_package(&self, package_id: &str) -> Result<Vec<u8>> {
        // Get package metadata
        let package = self
            .content_repo
            .get_package(package_id)
            .await?
            .context("Package not found in catalog")?;

        let download_url = package
            .download_url
            .as_ref()
            .context("Package has no download URL")?;

        // Download package data
        let response = self
            .http_client
            .get(download_url)
            .send()
            .await
            .context("Failed to download package")?;

        if !response.status().is_success() {
            bail!("HTTP error {} downloading package", response.status());
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read package bytes")?
            .to_vec();

        // Verify checksum if provided
        if let Some(expected_checksum) = &package.checksum {
            let actual_checksum = format!("{:x}", sha2::Sha256::digest(&bytes));
            if &actual_checksum != expected_checksum {
                bail!(
                    "Checksum mismatch! Expected: {}, Got: {}",
                    expected_checksum,
                    actual_checksum
                );
            }
        }

        Ok(bytes)
    }

    /// Install a package from downloaded SQLite data
    pub async fn install_package(&self, package_id: &str, package_data: Vec<u8>) -> Result<()> {
        // Get package metadata
        let package = self
            .content_repo
            .get_package(package_id)
            .await?
            .context("Package not found in catalog")?;

        // Write package data to temporary file
        let temp_file = std::env::temp_dir().join(format!("{}.db", package_id));
        std::fs::write(&temp_file, &package_data).context("Failed to write package file")?;

        // Open package database
        let package_pool =
            SqlitePool::connect(&format!("sqlite://{}?mode=ro", temp_file.display()))
                .await
                .context("Failed to open package database")?;

        // Install based on package type
        match package.package_type {
            PackageType::VerseTranslation => {
                self.install_translation_package(&package, &package_pool)
                    .await?;
            }
            PackageType::WordTranslation => {
                bail!("Word translation packages not yet supported");
            }
            PackageType::TextVariant => {
                bail!("Text variant packages not yet supported");
            }
            PackageType::VerseRecitation => {
                bail!("Verse recitation packages not yet supported");
            }
            PackageType::WordAudio => {
                bail!("Word audio packages not yet supported");
            }
            PackageType::Transliteration => {
                bail!("Transliteration packages not yet supported");
            }
        }

        // Close package database
        package_pool.close().await;

        // Delete temporary file
        std::fs::remove_file(&temp_file).ok();

        // Mark package as installed
        self.content_repo.mark_package_installed(package_id).await?;

        Ok(())
    }

    /// Install a translation package
    async fn install_translation_package(
        &self,
        package: &ContentPackage,
        package_pool: &SqlitePool,
    ) -> Result<()> {
        // 1. Create translator entry linked to package
        let translator_id = self
            .content_repo
            .insert_translator(
                &format!(
                    "{}-{}",
                    package
                        .language_code
                        .as_ref()
                        .unwrap_or(&"unknown".to_string()),
                    package.package_id
                ),
                &package.name,
                package.language_code.as_ref().unwrap_or(&"en".to_string()),
                package.description.as_deref(),
                package.author.as_deref(),
                package.license.as_deref(),
                None, // website
                Some(&package.version),
                Some(&package.package_id), // Link translator to this package
            )
            .await?;

        // 2. Import verse translations from package
        let translations: Vec<(String, String)> = sqlx::query_as(
            "SELECT verse_key, translation FROM verse_translations ORDER BY verse_key",
        )
        .fetch_all(package_pool)
        .await
        .context("Failed to read translations from package")?;

        // 3. Batch insert translations
        for (verse_key, translation) in translations {
            self.content_repo
                .insert_verse_translation(&verse_key, translator_id, &translation, None)
                .await?;
        }

        Ok(())
    }

    /// Uninstall a package (removes all associated data)
    pub async fn uninstall_package(&self, package_id: &str) -> Result<()> {
        // First mark as uninstalled
        self.content_repo
            .mark_package_uninstalled(package_id)
            .await?;

        // Then delete the package from catalog
        // This will CASCADE delete translators and all verse_translations
        // due to the foreign key constraints in the schema
        self.content_repo.delete_package(package_id).await?;

        Ok(())
    }

    /// Enable a package
    pub async fn enable_package(&self, package_id: &str) -> Result<()> {
        self.content_repo.enable_package(package_id).await
    }

    /// Disable a package
    pub async fn disable_package(&self, package_id: &str) -> Result<()> {
        self.content_repo.disable_package(package_id).await
    }

    /// Get all installed packages
    pub async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        self.content_repo.get_installed_packages().await
    }

    /// Check if a package is installed
    pub async fn is_installed(&self, package_id: &str) -> Result<bool> {
        self.content_repo.is_package_installed(package_id).await
    }

    /// Add a package to the catalog
    pub async fn add_to_catalog(&self, package: &ContentPackage) -> Result<()> {
        self.content_repo.upsert_package(package).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContentPackage, InstalledPackage, PackageType};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock ContentRepository for testing
    struct MockContentRepo {
        packages: Mutex<HashMap<String, ContentPackage>>,
        installed: Mutex<HashMap<String, InstalledPackage>>,
        translators: Mutex<Vec<(String, String, i32)>>, // (slug, package_id, translator_id)
        verse_translations: Mutex<Vec<(String, i32, String)>>, // (verse_key, translator_id, translation)
    }

    impl MockContentRepo {
        fn new() -> Self {
            Self {
                packages: Mutex::new(HashMap::new()),
                installed: Mutex::new(HashMap::new()),
                translators: Mutex::new(Vec::new()),
                verse_translations: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ContentRepository for MockContentRepo {
        async fn get_package(&self, package_id: &str) -> Result<Option<ContentPackage>> {
            Ok(self.packages.lock().unwrap().get(package_id).cloned())
        }

        async fn get_available_packages(
            &self,
            package_type: Option<PackageType>,
            language_code: Option<&str>,
        ) -> Result<Vec<ContentPackage>> {
            let packages = self.packages.lock().unwrap();
            let filtered: Vec<ContentPackage> = packages
                .values()
                .filter(|p| {
                    let type_match = package_type.as_ref().is_none_or(|t| &p.package_type == t);
                    let lang_match = language_code
                        .is_none_or(|l| p.language_code.as_ref().is_some_and(|code| code == l));
                    type_match && lang_match
                })
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn upsert_package(&self, package: &ContentPackage) -> Result<()> {
            self.packages
                .lock()
                .unwrap()
                .insert(package.package_id.clone(), package.clone());
            Ok(())
        }

        async fn is_package_installed(&self, package_id: &str) -> Result<bool> {
            Ok(self.installed.lock().unwrap().contains_key(package_id))
        }

        async fn mark_package_installed(&self, package_id: &str) -> Result<()> {
            self.installed.lock().unwrap().insert(
                package_id.to_string(),
                InstalledPackage {
                    package_id: package_id.to_string(),
                    installed_at: chrono::Utc::now(),
                    enabled: true,
                },
            );
            Ok(())
        }

        async fn mark_package_uninstalled(&self, package_id: &str) -> Result<()> {
            self.installed.lock().unwrap().remove(package_id);
            Ok(())
        }

        async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
            Ok(self.installed.lock().unwrap().values().cloned().collect())
        }

        async fn insert_translator(
            &self,
            slug: &str,
            _full_name: &str,
            _language_code: &str,
            _description: Option<&str>,
            _copyright_holder: Option<&str>,
            _license: Option<&str>,
            _website: Option<&str>,
            _version: Option<&str>,
            package_id: Option<&str>,
        ) -> Result<i32> {
            let translator_id = (self.translators.lock().unwrap().len() + 1) as i32;
            self.translators.lock().unwrap().push((
                slug.to_string(),
                package_id.unwrap_or("").to_string(),
                translator_id,
            ));
            Ok(translator_id)
        }

        async fn insert_verse_translation(
            &self,
            verse_key: &str,
            translator_id: i32,
            translation: &str,
            _footnotes: Option<&str>,
        ) -> Result<()> {
            self.verse_translations.lock().unwrap().push((
                verse_key.to_string(),
                translator_id,
                translation.to_string(),
            ));
            Ok(())
        }

        // Stub implementations for other required methods
        async fn get_node(&self, _node_id: &str) -> Result<Option<crate::Node>> {
            Ok(None)
        }
        async fn get_edges_from(&self, _source_id: &str) -> Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_edges_to(&self, _target_id: &str) -> Result<Vec<crate::Edge>> {
            Ok(vec![])
        }
        async fn get_quran_text(&self, _node_id: &str) -> Result<Option<String>> {
            Ok(None)
        }
        async fn get_translation(&self, _node_id: &str, _lang: &str) -> Result<Option<String>> {
            Ok(None)
        }
        async fn get_metadata(&self, _node_id: &str, _key: &str) -> Result<Option<String>> {
            Ok(None)
        }
        async fn get_all_metadata(&self, _node_id: &str) -> Result<HashMap<String, String>> {
            Ok(HashMap::new())
        }
        async fn node_exists(&self, _node_id: &str) -> Result<bool> {
            Ok(false)
        }
        async fn get_all_nodes(&self) -> Result<Vec<crate::Node>> {
            Ok(vec![])
        }
        async fn get_nodes_by_type(&self, _node_type: crate::NodeType) -> Result<Vec<crate::Node>> {
            Ok(vec![])
        }
        async fn insert_nodes_batch(&self, _nodes: &[crate::ImportedNode]) -> Result<()> {
            Ok(())
        }
        async fn insert_edges_batch(&self, _edges: &[crate::ImportedEdge]) -> Result<()> {
            Ok(())
        }
        async fn get_words_in_ayahs(&self, _ayah_node_ids: &[String]) -> Result<Vec<crate::Node>> {
            Ok(vec![])
        }
        async fn get_adjacent_words(
            &self,
            _word_node_id: &str,
        ) -> Result<(Option<crate::Node>, Option<crate::Node>)> {
            Ok((None, None))
        }
        async fn get_chapter(&self, _chapter_number: i32) -> Result<Option<crate::Chapter>> {
            Ok(None)
        }
        async fn get_chapters(&self) -> Result<Vec<crate::Chapter>> {
            Ok(vec![])
        }
        async fn get_verse(&self, _verse_key: &str) -> Result<Option<crate::Verse>> {
            Ok(None)
        }
        async fn get_verses_for_chapter(&self, _chapter_number: i32) -> Result<Vec<crate::Verse>> {
            Ok(vec![])
        }
        async fn get_words_for_verse(&self, _verse_key: &str) -> Result<Vec<crate::Word>> {
            Ok(vec![])
        }
        async fn get_word(&self, _word_id: i32) -> Result<Option<crate::Word>> {
            Ok(None)
        }
        async fn get_verses_batch(
            &self,
            verse_keys: &[String],
        ) -> Result<HashMap<String, crate::Verse>> {
            let mut result = HashMap::new();
            for key in verse_keys {
                if let Some(verse) = self.get_verse(key).await? {
                    result.insert(key.clone(), verse);
                }
            }
            Ok(result)
        }
        async fn get_words_batch(&self, word_ids: &[i32]) -> Result<HashMap<i32, crate::Word>> {
            let mut result = HashMap::new();
            for &id in word_ids {
                if let Some(word) = self.get_word(id).await? {
                    result.insert(id, word);
                }
            }
            Ok(result)
        }
        async fn get_languages(&self) -> Result<Vec<crate::Language>> {
            Ok(vec![])
        }
        async fn get_language(&self, _code: &str) -> Result<Option<crate::Language>> {
            Ok(None)
        }
        async fn get_translators_for_language(
            &self,
            _language_code: &str,
        ) -> Result<Vec<crate::Translator>> {
            Ok(vec![])
        }
        async fn get_translator(&self, _translator_id: i32) -> Result<Option<crate::Translator>> {
            Ok(None)
        }
        async fn get_translator_by_slug(&self, _slug: &str) -> Result<Option<crate::Translator>> {
            Ok(None)
        }
        async fn get_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
        ) -> Result<Option<String>> {
            Ok(None)
        }
        async fn get_word_translation(
            &self,
            _word_id: i32,
            _translator_id: i32,
        ) -> Result<Option<String>> {
            Ok(None)
        }
        async fn delete_package(&self, package_id: &str) -> Result<()> {
            self.packages.lock().unwrap().remove(package_id);
            Ok(())
        }
        async fn enable_package(&self, package_id: &str) -> Result<()> {
            if let Some(pkg) = self.installed.lock().unwrap().get_mut(package_id) {
                pkg.enabled = true;
            }
            Ok(())
        }
        async fn disable_package(&self, package_id: &str) -> Result<()> {
            if let Some(pkg) = self.installed.lock().unwrap().get_mut(package_id) {
                pkg.enabled = false;
            }
            Ok(())
        }
        async fn get_enabled_packages(&self) -> Result<Vec<InstalledPackage>> {
            Ok(self
                .installed
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.enabled)
                .cloned()
                .collect())
        }

        async fn get_morphology_for_word(
            &self,
            _word_id: i32,
        ) -> Result<Vec<crate::MorphologySegment>> {
            Ok(vec![])
        }

        async fn get_root_by_id(&self, _root_id: &str) -> Result<Option<crate::Root>> {
            Ok(None)
        }

        async fn get_lemma_by_id(&self, _lemma_id: &str) -> Result<Option<crate::Lemma>> {
            Ok(None)
        }

        async fn get_scheduler_candidates(
            &self,
            _goal_id: &str,
            _user_id: &str,
            _now_ts: i64,
        ) -> Result<Vec<crate::scheduler_v2::CandidateNode>> {
            Ok(vec![])
        }

        async fn get_prerequisite_parents(
            &self,
            _node_ids: &[String],
        ) -> Result<HashMap<String, Vec<String>>> {
            Ok(HashMap::new())
        }

        async fn get_goal(
            &self,
            _goal_id: &str,
        ) -> Result<Option<crate::ports::content_repository::SchedulerGoal>> {
            Ok(None)
        }

        async fn get_nodes_for_goal(&self, _goal_id: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_get_available_packages() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        // Add test packages
        let pkg1 = ContentPackage {
            package_id: "translation-en-test".to_string(),
            package_type: PackageType::VerseTranslation,
            name: "Test Translation".to_string(),
            language_code: Some("en".to_string()),
            author: Some("Test Author".to_string()),
            version: "1.0".to_string(),
            description: Some("Test package".to_string()),
            file_size: Some(1024),
            download_url: None,
            checksum: None,
            license: Some("MIT".to_string()),
        };

        repo.upsert_package(&pkg1).await.unwrap();

        // Test get all packages
        let packages = service.get_available_packages(None, None).await.unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package_id, "translation-en-test");

        // Test filter by type
        let packages = service
            .get_available_packages(Some(PackageType::VerseTranslation), None)
            .await
            .unwrap();
        assert_eq!(packages.len(), 1);

        // Test filter by language
        let packages = service
            .get_available_packages(None, Some("en"))
            .await
            .unwrap();
        assert_eq!(packages.len(), 1);
    }

    #[tokio::test]
    async fn test_get_package() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        let pkg = ContentPackage {
            package_id: "test-pkg".to_string(),
            package_type: PackageType::VerseTranslation,
            name: "Test".to_string(),
            language_code: Some("en".to_string()),
            author: None,
            version: "1.0".to_string(),
            description: None,
            file_size: None,
            download_url: None,
            checksum: None,
            license: None,
        };

        repo.upsert_package(&pkg).await.unwrap();

        let retrieved = service.get_package("test-pkg").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().package_id, "test-pkg");

        let not_found = service.get_package("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_is_installed() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        assert!(!service.is_installed("test-pkg").await.unwrap());

        repo.mark_package_installed("test-pkg").await.unwrap();

        assert!(service.is_installed("test-pkg").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_installed_packages() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        let installed = service.get_installed_packages().await.unwrap();
        assert_eq!(installed.len(), 0);

        repo.mark_package_installed("pkg1").await.unwrap();
        repo.mark_package_installed("pkg2").await.unwrap();

        let installed = service.get_installed_packages().await.unwrap();
        assert_eq!(installed.len(), 2);
    }

    #[tokio::test]
    async fn test_add_to_catalog() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        let pkg = ContentPackage {
            package_id: "new-pkg".to_string(),
            package_type: PackageType::VerseTranslation,
            name: "New Package".to_string(),
            language_code: Some("ar".to_string()),
            author: None,
            version: "1.0".to_string(),
            description: None,
            file_size: None,
            download_url: None,
            checksum: None,
            license: None,
        };

        service.add_to_catalog(&pkg).await.unwrap();

        let retrieved = repo.get_package("new-pkg").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "New Package");
    }

    #[tokio::test]
    async fn test_enable_disable_package() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        repo.mark_package_installed("test-pkg").await.unwrap();

        // Should be enabled by default
        let enabled = repo.get_enabled_packages().await.unwrap();
        assert_eq!(enabled.len(), 1);

        // Disable
        service.disable_package("test-pkg").await.unwrap();
        let enabled = repo.get_enabled_packages().await.unwrap();
        assert_eq!(enabled.len(), 0);

        // Re-enable
        service.enable_package("test-pkg").await.unwrap();
        let enabled = repo.get_enabled_packages().await.unwrap();
        assert_eq!(enabled.len(), 1);
    }

    #[tokio::test]
    async fn test_uninstall_package() {
        let repo = Arc::new(MockContentRepo::new());
        let service = PackageService::new(repo.clone());

        // Setup: add package to catalog and mark as installed
        let pkg = ContentPackage {
            package_id: "uninstall-test".to_string(),
            package_type: PackageType::VerseTranslation,
            name: "Uninstall Test".to_string(),
            language_code: Some("en".to_string()),
            author: None,
            version: "1.0".to_string(),
            description: None,
            file_size: None,
            download_url: None,
            checksum: None,
            license: None,
        };

        repo.upsert_package(&pkg).await.unwrap();
        repo.mark_package_installed("uninstall-test").await.unwrap();

        assert!(repo.is_package_installed("uninstall-test").await.unwrap());

        // Uninstall
        service.uninstall_package("uninstall-test").await.unwrap();

        assert!(!repo.is_package_installed("uninstall-test").await.unwrap());
    }
}
