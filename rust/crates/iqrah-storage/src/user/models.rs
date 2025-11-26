/// Database row types for user.db
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MemoryStateRow {
    pub user_id: String,
    pub content_key: String, // Renamed from node_id in v2 migration
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: i64, // milliseconds since epoch
    pub due_at: i64,
    pub review_count: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct PropagationEventRow {
    pub id: i64,
    pub source_content_key: String, // Renamed from source_node_id in v2 migration
    pub event_timestamp: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct PropagationDetailRow {
    pub id: i64,
    pub event_id: i64,
    pub target_content_key: String, // Renamed from target_node_id in v2 migration
    pub energy_change: f64,
    pub path: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionStateRow {
    pub content_key: String, // Renamed from node_id in v2 migration
    #[allow(dead_code)]
    pub session_order: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserStatRow {
    #[allow(dead_code)]
    pub key: String,
    pub value: String,
}

// ============================================================================
// Scheduler v2.0 Models
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct ParentEnergyRow {
    pub node_id: String,
    pub energy: f32,
}

/// Memory basics for scheduler (energy + next_due_ts)
#[derive(Debug, Clone, FromRow)]
pub struct MemoryBasicsRow {
    pub node_id: i64,
    pub energy: f32,
    pub next_due_ts: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct BanditArmRow {
    pub profile_name: String,
    pub successes: f32,
    pub failures: f32,
}
