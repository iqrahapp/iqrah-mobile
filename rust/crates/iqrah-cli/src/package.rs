use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct PackageListItem {
    package_id: String,
    package_type: String,
    name: String,
    language_code: Option<String>,
    author: Option<String>,
    version: String,
    description: Option<String>,
    file_size: Option<i64>,
    download_url: Option<String>,
    checksum: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PackageDetails {
    package_id: String,
    package_type: String,
    name: String,
    language_code: Option<String>,
    author: Option<String>,
    version: String,
    description: Option<String>,
    file_size: Option<i64>,
    download_url: Option<String>,
    checksum: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InstalledPackage {
    package_id: String,
    installed_at: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct UpsertPackageRequest {
    package_type: String,
    name: String,
    language_code: Option<String>,
    author: Option<String>,
    version: String,
    description: Option<String>,
    file_size: Option<i64>,
    download_url: Option<String>,
    checksum: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    message: String,
    package_id: String,
}

/// List all available packages
pub async fn list_packages(server_url: &str) -> Result<()> {
    let url = format!("{}/packages", server_url);
    let response: Vec<PackageListItem> = reqwest::get(&url).await?.json().await?;

    println!("📦 Available Packages:");
    println!();
    for package in response {
        println!("  [{}] {}", package.package_id, package.name);
        println!("      Type: {}", package.package_type);
        if let Some(lang) = package.language_code {
            println!("      Language: {}", lang);
        }
        if let Some(author) = package.author {
            println!("      Author: {}", author);
        }
        println!("      Version: {}", package.version);
        if let Some(desc) = package.description {
            println!("      Description: {}", desc);
        }
        if let Some(size) = package.file_size {
            println!("      Size: {} bytes", size);
        }
        println!();
    }

    Ok(())
}

/// Get package details by ID
pub async fn get_package(server_url: &str, package_id: &str) -> Result<()> {
    let url = format!("{}/packages/{}", server_url, package_id);
    let package: PackageDetails = reqwest::get(&url).await?.json().await?;

    println!("📦 Package Details:");
    println!();
    println!("  ID:          {}", package.package_id);
    println!("  Name:        {}", package.name);
    println!("  Type:        {}", package.package_type);
    println!("  Version:     {}", package.version);
    if let Some(lang) = package.language_code {
        println!("  Language:    {}", lang);
    }
    if let Some(author) = package.author {
        println!("  Author:      {}", author);
    }
    if let Some(desc) = package.description {
        println!("  Description: {}", desc);
    }
    if let Some(size) = package.file_size {
        println!("  Size:        {} bytes", size);
    }
    if let Some(url) = package.download_url {
        println!("  Download URL: {}", url);
    }
    if let Some(checksum) = package.checksum {
        println!("  Checksum:    {}", checksum);
    }
    if let Some(license) = package.license {
        println!("  License:     {}", license);
    }

    Ok(())
}

/// List installed packages
pub async fn list_installed(server_url: &str) -> Result<()> {
    let url = format!("{}/packages/installed", server_url);
    let response: Vec<InstalledPackage> = reqwest::get(&url).await?.json().await?;

    println!("📦 Installed Packages:");
    println!();
    if response.is_empty() {
        println!("  No packages installed.");
    } else {
        for package in response {
            let status = if package.enabled { "✓ enabled" } else { "✗ disabled" };
            println!("  [{}] {}", package.package_id, status);
            println!("      Installed at: {}", package.installed_at);
            println!();
        }
    }

    Ok(())
}

/// Install a package
pub async fn install_package(server_url: &str, package_id: &str) -> Result<()> {
    let url = format!("{}/packages/installed/{}", server_url, package_id);
    let client = reqwest::Client::new();
    let response: ApiResponse = client.post(&url).send().await?.json().await?;

    println!("✅ {}", response.message);
    println!("   Package ID: {}", response.package_id);

    Ok(())
}

/// Uninstall a package
pub async fn uninstall_package(server_url: &str, package_id: &str) -> Result<()> {
    let url = format!("{}/packages/installed/{}", server_url, package_id);
    let client = reqwest::Client::new();
    let response: ApiResponse = client.delete(&url).send().await?.json().await?;

    println!("✅ {}", response.message);
    println!("   Package ID: {}", response.package_id);

    Ok(())
}

/// Enable a package
pub async fn enable_package(server_url: &str, package_id: &str) -> Result<()> {
    let url = format!("{}/packages/installed/{}/enable", server_url, package_id);
    let client = reqwest::Client::new();
    let response: ApiResponse = client.post(&url).send().await?.json().await?;

    println!("✅ {}", response.message);
    println!("   Package ID: {}", response.package_id);

    Ok(())
}

/// Disable a package
pub async fn disable_package(server_url: &str, package_id: &str) -> Result<()> {
    let url = format!("{}/packages/installed/{}/disable", server_url, package_id);
    let client = reqwest::Client::new();
    let response: ApiResponse = client.post(&url).send().await?.json().await?;

    println!("✅ {}", response.message);
    println!("   Package ID: {}", response.package_id);

    Ok(())
}
