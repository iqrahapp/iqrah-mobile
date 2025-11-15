/// Database row types for user.db
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MemoryStateRow {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: i64,  // milliseconds since epoch
    pub due_at: i64,
    pub review_count: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct PropagationEventRow {
    pub id: i64,
    pub source_node_id: String,
    pub event_timestamp: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct PropagationDetailRow {
    pub id: i64,
    pub event_id: i64,
    pub target_node_id: String,
    pub energy_change: f64,
    pub path: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionStateRow {
    pub node_id: String,
    pub session_order: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserStatRow {
    pub key: String,
    pub value: String,
}
