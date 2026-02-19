//! Sync types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Sync push request.
#[derive(Debug, Deserialize, Validate)]
pub struct SyncPushRequest {
    pub device_id: Uuid,
    #[validate(nested)]
    pub changes: SyncChanges,
    /// Device OS (e.g., "Android 14", "iOS 17.2"). Max 50 characters.
    #[validate(length(max = 50))]
    pub device_os: Option<String>,
    /// Device model (e.g., "Pixel 8 Pro", "iPhone 15"). Max 100 characters.
    #[validate(length(max = 100))]
    pub device_model: Option<String>,
    /// App version (e.g., "1.2.3"). Max 20 characters.
    #[validate(length(max = 20))]
    pub app_version: Option<String>,
}

/// Sync pull request.
#[derive(Debug, Deserialize, Validate)]
pub struct SyncPullRequest {
    pub device_id: Uuid,
    /// Timestamp in milliseconds since epoch. Returns changes after this time.
    #[validate(range(min = 0))]
    pub since: i64,
    /// Max records per batch. Default: 1000, Maximum allowed: 10000.
    #[validate(range(min = 1, max = 10000))]
    #[serde(default = "default_limit")]
    pub limit: Option<usize>,
    /// Per-entity pagination cursor (optional).
    #[serde(default)]
    #[validate(nested)]
    pub cursor: Option<SyncPullCursor>,
    /// Device OS (e.g., "Android 14", "iOS 17.2"). Max 50 characters.
    #[validate(length(max = 50))]
    pub device_os: Option<String>,
    /// Device model (e.g., "Pixel 8 Pro", "iPhone 15"). Max 100 characters.
    #[validate(length(max = 100))]
    pub device_model: Option<String>,
    /// App version (e.g., "1.2.3"). Max 20 characters.
    #[validate(length(max = 20))]
    pub app_version: Option<String>,
}

fn default_limit() -> Option<usize> {
    Some(1000)
}

/// Per-entity cursor for paginated sync pulls.
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SyncPullCursor {
    #[serde(default)]
    #[validate(nested)]
    pub settings: Option<SyncCursorSetting>,
    #[serde(default)]
    #[validate(nested)]
    pub memory_states: Option<SyncCursorMemoryState>,
    #[serde(default)]
    #[validate(nested)]
    pub sessions: Option<SyncCursorSession>,
    #[serde(default)]
    #[validate(nested)]
    pub session_items: Option<SyncCursorSessionItem>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SyncCursorSetting {
    #[validate(range(min = 0))]
    pub updated_at: i64,
    #[validate(length(min = 1, max = 255))]
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SyncCursorMemoryState {
    #[validate(range(min = 0))]
    pub updated_at: i64,
    pub node_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SyncCursorSession {
    #[validate(range(min = 0))]
    pub updated_at: i64,
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SyncCursorSessionItem {
    #[validate(range(min = 0))]
    pub updated_at: i64,
    pub id: Uuid,
}

/// Collection of sync changes.
#[derive(Debug, Default, Serialize, Deserialize, Validate)]
pub struct SyncChanges {
    #[serde(default)]
    #[validate(nested)]
    pub settings: Vec<SettingChange>,
    #[serde(default)]
    #[validate(nested)]
    pub memory_states: Vec<MemoryStateChange>,
    #[serde(default)]
    #[validate(nested)]
    pub sessions: Vec<SessionChange>,
    #[serde(default)]
    #[validate(nested)]
    pub session_items: Vec<SessionItemChange>,
}

/// Setting change.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SettingChange {
    #[validate(length(min = 1, max = 255))]
    pub key: String,
    pub value: serde_json::Value,
}

/// Memory state change.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MemoryStateChange {
    pub node_id: i64,
    #[validate(range(min = 0.0, max = 1.0))]
    pub energy: f32,
    #[validate(range(min = 0.0))]
    pub fsrs_stability: Option<f32>,
    #[validate(range(min = 0.0))]
    pub fsrs_difficulty: Option<f32>,
    #[validate(range(min = 0))]
    pub last_reviewed_at: Option<i64>,
    #[validate(range(min = 0))]
    pub next_review_at: Option<i64>,
}

/// Session change.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SessionChange {
    pub id: Uuid,
    #[validate(length(max = 255))]
    pub goal_id: Option<String>,
    #[validate(range(min = 0))]
    pub started_at: i64,
    #[validate(range(min = 0))]
    pub completed_at: Option<i64>,
    #[validate(range(min = 0))]
    pub items_completed: i32,
}

/// Session item change.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SessionItemChange {
    pub id: Uuid,
    pub session_id: Uuid,
    pub node_id: i64,
    #[validate(length(min = 1, max = 100))]
    pub exercise_type: String,
    #[validate(range(min = 0, max = 5))]
    pub grade: Option<i32>,
    #[validate(range(min = 0))]
    pub duration_ms: Option<i32>,
}

/// Sync push response.
#[derive(Debug, Serialize)]
pub struct SyncPushResponse {
    /// Number of changes accepted and written (LWW won).
    pub applied: u64,
    /// Number of changes silently rejected because the server had a newer version (LWW lost).
    pub skipped: u64,
    pub server_time: i64,
}

/// Sync pull response.
#[derive(Debug, Serialize)]
pub struct SyncPullResponse {
    pub server_time: i64,
    pub changes: SyncChanges,
    pub has_more: bool, // true if there are more records available
    pub next_cursor: Option<SyncPullCursor>,
}
