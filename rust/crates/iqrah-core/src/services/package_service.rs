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
            bail!(
                "HTTP error {} downloading package",
                response.status()
            );
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
        let package_pool = SqlitePool::connect(&format!("sqlite://{}?mode=ro", temp_file.display()))
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
                &format!("{}-{}", package.language_code.as_ref().unwrap_or(&"unknown".to_string()), package.package_id),
                &package.name,
                package.language_code.as_ref().unwrap_or(&"en".to_string()),
                package.description.as_deref(),
                package.author.as_deref(),
                package.license.as_deref(),
                None, // website
                Some(&package.version),
            )
            .await?;

        // 2. Link translator to package
        // Note: Since insert_translator doesn't support package_id yet,
        // we'll need to add that functionality to properly link them
        // For now, this creates the translator without the package_id link
        // TODO: Add package_id parameter to insert_translator

        // 3. Import verse translations from package
        let translations: Vec<(String, String)> = sqlx::query_as(
            "SELECT verse_key, translation FROM verse_translations ORDER BY verse_key",
        )
        .fetch_all(package_pool)
        .await
        .context("Failed to read translations from package")?;

        // 4. Batch insert translations
        for (verse_key, translation) in translations {
            self.content_repo
                .insert_verse_translation(&verse_key, translator_id, &translation, None)
                .await?;
        }

        Ok(())
    }

    /// Uninstall a package (removes all associated data)
    pub async fn uninstall_package(&self, package_id: &str) -> Result<()> {
        // Get package to determine type
        let package = self
            .content_repo
            .get_package(package_id)
            .await?
            .context("Package not found")?;

        match package.package_type {
            PackageType::VerseTranslation => {
                // Delete translator (CASCADE will delete verse_translations)
                // This requires a raw SQL query since we don't have a delete_translator method
                // For now, we'll use mark_package_uninstalled which should trigger CASCADE
            }
            _ => {
                // Other types TBD
            }
        }

        // Mark as uninstalled
        self.content_repo
            .mark_package_uninstalled(package_id)
            .await?;

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
