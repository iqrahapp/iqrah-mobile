use anyhow::Result;
use iqrah_core::{ContentPackage, ContentRepository, PackageType};
use iqrah_storage::{
    content::{init_content_db, SqliteContentRepository},
    user::{init_user_db, SqliteUserRepository},
};
use std::sync::Arc;

#[tokio::test]
async fn test_package_management() -> Result<()> {
    // Initialize in-memory databases for testing
    let content_pool = init_content_db(":memory:").await?;
    let _user_pool = init_user_db(":memory:").await?;

    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));

    // Test 1: Upsert a package
    let package = ContentPackage {
        package_id: "test-package-v1".to_string(),
        package_type: PackageType::VerseTranslation,
        name: "Test Translation Package".to_string(),
        language_code: Some("en".to_string()),
        author: Some("Test Author".to_string()),
        version: "1.0".to_string(),
        description: Some("A test package for integration testing".to_string()),
        file_size: Some(1024),
        download_url: Some("https://example.com/package.db".to_string()),
        checksum: Some("abc123".to_string()),
        license: Some("MIT".to_string()),
    };

    content_repo.upsert_package(&package).await?;
    println!("✓ Package upserted successfully");

    // Test 2: Get the package
    let retrieved = content_repo.get_package("test-package-v1").await?;
    assert!(retrieved.is_some(), "Package should exist");
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.package_id, "test-package-v1");
    assert_eq!(retrieved.name, "Test Translation Package");
    println!("✓ Package retrieved successfully");

    // Test 3: List all packages
    let all_packages = content_repo.get_available_packages(None, None).await?;
    assert!(!all_packages.is_empty(), "Should have at least one package");
    println!("✓ Listed {} package(s)", all_packages.len());

    // Test 4: Filter by package type
    let translation_packages = content_repo
        .get_available_packages(Some(PackageType::VerseTranslation), None)
        .await?;
    assert!(!translation_packages.is_empty(), "Should have translation packages");
    println!("✓ Filtered packages by type");

    // Test 5: Mark package as installed
    content_repo.mark_package_installed("test-package-v1").await?;
    println!("✓ Package marked as installed");

    // Test 6: Check if package is installed
    let is_installed = content_repo.is_package_installed("test-package-v1").await?;
    assert!(is_installed, "Package should be installed");
    println!("✓ Verified package installation");

    // Test 7: List installed packages
    let installed = content_repo.get_installed_packages().await?;
    assert_eq!(installed.len(), 1, "Should have exactly one installed package");
    assert_eq!(installed[0].package_id, "test-package-v1");
    assert!(installed[0].enabled, "Package should be enabled by default");
    println!("✓ Listed installed packages");

    // Test 8: Disable package
    content_repo.disable_package("test-package-v1").await?;
    let installed_after_disable = content_repo.get_installed_packages().await?;
    assert!(!installed_after_disable[0].enabled, "Package should be disabled");
    println!("✓ Package disabled successfully");

    // Test 9: Enable package
    content_repo.enable_package("test-package-v1").await?;
    let installed_after_enable = content_repo.get_installed_packages().await?;
    assert!(installed_after_enable[0].enabled, "Package should be enabled");
    println!("✓ Package enabled successfully");

    // Test 10: Get enabled packages only
    let enabled_packages = content_repo.get_enabled_packages().await?;
    assert_eq!(enabled_packages.len(), 1, "Should have one enabled package");
    println!("✓ Listed enabled packages");

    // Test 11: Uninstall package
    content_repo.mark_package_uninstalled("test-package-v1").await?;
    let is_installed_after_uninstall = content_repo.is_package_installed("test-package-v1").await?;
    assert!(!is_installed_after_uninstall, "Package should not be installed");
    println!("✓ Package uninstalled successfully");

    // Test 12: Delete package
    content_repo.delete_package("test-package-v1").await?;
    let deleted_package = content_repo.get_package("test-package-v1").await?;
    assert!(deleted_package.is_none(), "Package should be deleted");
    println!("✓ Package deleted successfully");

    println!("\n✅ All package management tests passed!");
    Ok(())
}
