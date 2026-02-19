//! Configuration module for Iqrah backend.

use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(String),
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// PostgreSQL connection URL
    pub database_url: String,
    /// JWT secret for token signing
    pub jwt_secret: String,
    /// Path to pack storage directory
    pub pack_storage_path: String,
    /// Google OAuth client ID
    pub google_client_id: String,
    /// Address to bind the server to
    pub bind_address: String,
    /// Base URL for API (used in download URLs)
    pub base_url: String,
    /// Shared admin key for observability endpoints. Empty disables admin endpoints.
    pub admin_api_key: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env_var("DATABASE_URL")?,
            jwt_secret: env_var("JWT_SECRET")?,
            pack_storage_path: env_var_or("PACK_STORAGE_PATH", "./packs"),
            google_client_id: env_var_or("GOOGLE_CLIENT_ID", ""),
            bind_address: env_var_or("BIND_ADDRESS", "0.0.0.0:8080"),
            base_url: env_var_or("BASE_URL", "http://localhost:8080"),
            admin_api_key: env_var_or("ADMIN_API_KEY", ""),
        })
    }
}

fn env_var(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingVar(name.to_string()))
}

fn env_var_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_or_uses_default() {
        let val = env_var_or("NON_EXISTENT_VAR_12345", "default_value");
        assert_eq!(val, "default_value");
    }
}
