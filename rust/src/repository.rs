use anyhow::Result;
use async_trait::async_trait;
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Node data with metadata for Flutter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub arabic: String,
    pub translation: String,
}

// Review grades (will map to FSRS later)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewGrade {
    Again, // 1 - Complete failure, no recall
    Hard,  // 2 - Recalled with significant difficulty/hesitation
    Good,  // 3 - Recalled correctly with some effort
    Easy,  // 4 - Perfect recall, no hesitation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,        // mastery 0-1 scale
    pub last_reviewed: i64, // epoch ms
    pub due_at: i64,        // epoch ms
    pub review_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DueItem {
    pub node_id: String,
    pub arabic: Option<String>,
    pub state: MemoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStats {
    pub due_today: u32,
    pub total_reviewed: u32,
    pub avg_energy: f64,
    pub next_due_items: Vec<DueItem>,
}
/// Repository trait with Send + Sync for thread safety
#[frb(ignore)]
#[async_trait]
pub trait KnowledgeGraphRepository: Send + Sync {
    async fn seed(&self) -> Result<()>;
    async fn get_due_items(&self, user_id: &str, limit: u32) -> Result<Vec<NodeData>>;
    async fn get_node_data(&self, node_id: &str) -> Result<NodeData>;
    async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState>;
    async fn get_debug_stats(&self, user_id: &str) -> Result<DebugStats>;
}

/// Service owns a trait object - perfect for testing
#[frb(ignore)]
pub struct LearningService {
    repo: Arc<dyn KnowledgeGraphRepository>,
}

#[frb(ignore)]
impl LearningService {
    pub fn new(repo: Arc<dyn KnowledgeGraphRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_due_items(&self, user_id: &str, limit: u32) -> Result<Vec<NodeData>> {
        self.repo.get_due_items(user_id, limit).await
    }

    pub async fn get_node_data(&self, node_id: &str) -> Result<NodeData> {
        self.repo.get_node_data(node_id).await
    }

    pub async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        self.repo.process_review(user_id, node_id, grade).await
    }

    pub async fn get_debug_stats(&self, user_id: &str) -> Result<DebugStats> {
        self.repo.get_debug_stats(user_id).await
    }

    // FIXME[low]: this is a hack to expose seeding via the service - should be in AdminService
    pub async fn seed(&self) -> Result<()> {
        self.repo.seed().await
    }
}
