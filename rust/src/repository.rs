use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Node data with metadata for Flutter
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
    pub last_reviewed: i64, // epoch ms
    pub due_at: i64,        // epoch ms
    pub review_count: i32,
}

/// Repository trait with Send + Sync for thread safety
pub trait KnowledgeGraphRepository: Send + Sync {
    fn get_due_items(&self, user_id: &str, limit: u32) -> Result<Vec<NodeData>>;
    fn get_node_data(&self, node_id: &str) -> Result<NodeData>;
    fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState>;
}

/// Service owns a trait object - perfect for testing
pub struct LearningService {
    repo: Arc<dyn KnowledgeGraphRepository>,
}

impl LearningService {
    pub fn new(repo: Arc<dyn KnowledgeGraphRepository>) -> Self {
        Self { repo }
    }

    pub fn get_due_items(&self, user_id: &str, limit: u32) -> Result<Vec<NodeData>> {
        self.repo.get_due_items(user_id, limit)
    }

    pub fn get_node_data(&self, node_id: &str) -> Result<NodeData> {
        self.repo.get_node_data(node_id)
    }

    pub fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        self.repo.process_review(user_id, node_id, grade)
    }
}
