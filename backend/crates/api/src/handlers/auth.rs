//! Auth handlers.

use std::sync::Arc;

use axum::{Json, extract::State};
use jsonwebtoken::{EncodingKey, Header, encode};

use google_jwt_verify::Client as GoogleJwtClient;
use iqrah_backend_domain::{AuthResponse, Claims, DomainError, GoogleAuthRequest, UserProfile};

pub trait IdTokenVerifier: Send + Sync {
    fn verify(&self, id_token: &str) -> Result<String, String>;
}

#[derive(Clone)]
pub struct GoogleIdTokenVerifier {
    client: Arc<GoogleJwtClient>,
}

impl GoogleIdTokenVerifier {
    pub fn new(client_id: &str) -> Self {
        Self {
            client: Arc::new(GoogleJwtClient::new(client_id)),
        }
    }
}

impl IdTokenVerifier for GoogleIdTokenVerifier {
    fn verify(&self, id_token: &str) -> Result<String, String> {
        self.client
            .verify_id_token(id_token)
            .map_err(|e| format!("Google token verification failed: {:?}", e))
            .map(|token| token.get_claims().get_subject())
    }
}

use crate::AppState;
use crate::middleware::auth::AuthUser;

/// Google OAuth login handler.
pub async fn google_auth(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GoogleAuthRequest>,
) -> Result<Json<AuthResponse>, DomainError> {
    // Validate ID token is not empty
    if req.id_token.trim().is_empty() {
        return Err(DomainError::Validation(
            "ID token cannot be empty".to_string(),
        ));
    }

    if state.config.google_client_id.trim().is_empty() {
        tracing::error!("GOOGLE_CLIENT_ID is not configured");
        return Err(DomainError::Internal(anyhow::anyhow!(
            "Google OAuth is not configured"
        )));
    }

    // Verify the ID token with Google's public keys
    let oauth_sub = verify_google_token(&req.id_token, state.id_token_verifier.clone())
        .await
        .map_err(|e| {
            tracing::error!("Token verification failed: {}", e);
            DomainError::Unauthorized(format!("Invalid Google ID token: {}", e))
        })?;

    tracing::info!(oauth_sub = %oauth_sub, "Google token verified successfully");

    // Find or create user
    let user = state
        .user_repo
        .find_or_create(&oauth_sub)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find/create user: {}", e);
            DomainError::Database(e.to_string())
        })?;

    // Issue JWT
    let expires_in = 3600u64; // 1 hour
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user.id.to_string(),
        exp: now + expires_in,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to encode JWT: {}", e);
        DomainError::Internal(anyhow::anyhow!("Failed to generate access token: {}", e))
    })?;

    tracing::info!(user_id = %user.id, "Access token issued successfully");

    Ok(Json(AuthResponse {
        access_token: token,
        user_id: user.id,
        expires_in,
    }))
}

/// Get current user profile.
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<UserProfile>, DomainError> {
    tracing::info!(user_id = %user_id, "Fetching user profile");

    let user = state
        .user_repo
        .get_by_id(user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {}", e);
            DomainError::Database(e.to_string())
        })?
        .ok_or_else(|| DomainError::NotFound(format!("User {} not found", user_id)))?;

    Ok(Json(UserProfile {
        id: user.id,
        created_at: user.created_at,
        last_seen_at: user.last_seen_at,
    }))
}

async fn verify_google_token(
    id_token: &str,
    verifier: Arc<dyn IdTokenVerifier>,
) -> Result<String, String> {
    let token_string = id_token.to_string();
    tokio::task::spawn_blocking(move || verifier.verify(&token_string))
        .await
        .map_err(|e| format!("Token verification task failed: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeVerifier;

    impl IdTokenVerifier for FakeVerifier {
        fn verify(&self, id_token: &str) -> Result<String, String> {
            if id_token == "valid-token" {
                Ok("test-subject".to_string())
            } else {
                Err("invalid token".to_string())
            }
        }
    }

    #[tokio::test]
    async fn verify_google_token_accepts_valid_token() {
        let verifier = Arc::new(FakeVerifier);
        let subject = verify_google_token("valid-token", verifier).await.unwrap();

        assert_eq!(subject, "test-subject");
    }

    #[tokio::test]
    async fn verify_google_token_rejects_invalid_token() {
        let verifier = Arc::new(FakeVerifier);
        let result = verify_google_token("invalid-token", verifier).await;

        assert!(result.is_err());
    }
}
