// Type definitions for backwards compatibility with Flutter
// Implementations are now in the iqrah-api crate

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: String,
    pub node_type: NodeType,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewGrade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i32,
    pub scheduled_days: i32,
    pub reps: i32,
    pub lapses: i32,
    pub last_review: DateTime<Utc>,
    pub state: i32,
}

#[derive(Debug, Clone)]
pub struct DueItem {
    pub node_data: NodeData,
    pub memory_state: MemoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStats {
    pub total_nodes_count: u32,
    pub total_edges_count: u32,
    pub user_memory_states_count: u32,
    pub due_now_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPreview {
    pub node_id: String,
    pub text_preview: String,
    pub due_in_hours: f64,
    pub priority_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub reviews_today: u32,
    pub streak_days: u32,
    pub due_count: u32,
    pub total_reviews: u32,
}

// NodeType enum for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Root,
    Lemma,
    Word,
    WordInstance,
    Verse,
    Chapter,
    Knowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub days_overdue: f64,
    pub mastery_gap: f64,
    pub importance: f64,
    pub weights: ScoreWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub w_due: f64,
    pub w_need: f64,
    pub w_yield: f64,
}
