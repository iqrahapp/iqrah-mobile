use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{
    cbor_import::{
        import_cbor_graph_from_bytes, ImportStats, ImportedEdge, ImportedNode, NodeType,
    },
    propagation::EdgeForPropagation,
};

// Node data with metadata for Flutter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
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
    pub energy: f64,        // mastery [-1, 1] scale
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
    pub total_nodes_count: usize,
    pub total_edges_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPreview {
    pub node_id: String,
    pub arabic: Option<String>,
    pub translation: Option<String>,
    pub priority_score: f64,
    pub score_breakdown: ScoreBreakdown,
    pub memory_state: MemoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub days_overdue: f64,
    pub mastery_gap: f64, // 1.0 - energy
    pub importance: f64,  // foundational_score (direct value, not multiplied)
    pub weights: ScoreWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub w_due: f64,   // 1.0
    pub w_need: f64,  // 2.0
    pub w_yield: f64, // 1.5
}

// Lightweight context to build MCQ exercises for a word instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInstanceContext {
    pub node_id: String,      // WORD_INSTANCE:ch:ayah:idx
    pub arabic: String,       // target word
    pub translation: String,  // target translation
    pub verse_arabic: String, // full verse
    pub surah_number: i32,
    pub ayah_number: i32,
    pub word_index: i32,                 // 1-based in verse
    pub verse_word_ar_list: Vec<String>, // all words in verse (arabic)
    pub verse_word_en_list: Vec<String>, // all translations in verse (parallel)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub reviews_today: u32,
    pub streak_days: u32,
}

#[derive(Debug, Clone)]
pub struct PropagationLogDetail {
    pub target_node_id: String,
    pub energy_change: f64,
    pub path: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PropagationDetailRecord {
    pub event_timestamp: i64,
    pub source_node_id: String,
    pub target_node_id: String,
    pub source_text: Option<String>,
    pub target_text: Option<String>,
    pub energy_change: f64,
    pub path: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct PropagationQueryOptions {
    pub start_time_secs: Option<i64>,
    pub end_time_secs: Option<i64>,
    pub limit: u32,
}

/// Repository trait with Send + Sync for thread safety
#[frb(ignore)]
#[async_trait]
pub trait KnowledgeGraphRepository: Send + Sync {
    async fn get_due_items(
        &self,
        user_id: &str,
        limit: u32,
        surah_filter: Option<i32>,
        is_high_yield_mode: bool,
    ) -> Result<Vec<NodeData>>;
    async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<(MemoryState, f64)>;
    async fn get_debug_stats(&self, user_id: &str) -> Result<DebugStats>;
    async fn get_knowledge_edges(&self, source_node_id: &str) -> Result<Vec<EdgeForPropagation>>;
    async fn get_node_energy(&self, user_id: &str, node_id: &str) -> Result<Option<f64>>;
    async fn update_node_energies(&self, user_id: &str, updates: &[(String, f64)]) -> Result<()>;
    async fn log_propagation_event(
        &self,
        source_node_id: &str,
        event_timestamp: i64,
        details: &[PropagationLogDetail],
    ) -> Result<()>;
    async fn query_propagation_details(
        &self,
        filter: PropagationQueryOptions,
    ) -> Result<Vec<PropagationDetailRecord>>;
    async fn sync_user_nodes(&self, user_id: &str) -> Result<()>;
    async fn reset_user_progress(&self, user_id: &str) -> Result<()>;
    async fn refresh_all_priority_scores(&self, user_id: &str) -> Result<()>;
    async fn get_session_preview(
        &self,
        user_id: &str,
        limit: u32,
        surah_filter: Option<i32>,
        is_high_yield_mode: bool,
    ) -> Result<Vec<ItemPreview>>;
    async fn get_available_surahs(&self) -> Result<Vec<(i32, String)>>;

    // Context lookups for exercise building
    async fn get_word_instance_context(&self, node_id: &str) -> Result<WordInstanceContext>;

    // Node search and fetch
    async fn search_nodes(&self, query: &str, limit: u32) -> Result<Vec<NodeData>>;
    async fn get_node_with_metadata(&self, node_id: &str) -> Result<Option<NodeData>>;

    // Batch operations for setup/import
    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> Result<()>;
    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> Result<()>;

    // Session persistence
    async fn save_session(&self, node_ids: &[String]) -> Result<()>;
    async fn get_existing_session(&self) -> Result<Option<Vec<NodeData>>>;
    async fn remove_from_session(&self, node_id: &str) -> Result<()>;
    async fn clear_session(&self) -> Result<()>;

    // User statistics
    async fn get_dashboard_stats(&self, user_id: &str) -> Result<DashboardStats>;
    async fn update_user_stats(
        &self,
        reviews_today: u32,
        streak_days: u32,
        last_review_date: &str,
    ) -> Result<()>;
    async fn get_last_review_date(&self) -> Result<Option<String>>;
}

/// Service owns a trait object - perfect for testing
#[frb(ignore)]
pub struct LearningService {
    pub repo: Arc<dyn KnowledgeGraphRepository>,
}

#[frb(ignore)]
impl LearningService {
    const PROPAGATION_TRIGGER_DELTA: f64 = 0.0001;

    pub fn new(repo: Arc<dyn KnowledgeGraphRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_due_items(
        &self,
        user_id: &str,
        limit: u32,
        surah_filter: Option<i32>,
        is_high_yield_mode: bool,
    ) -> Result<Vec<NodeData>> {
        self.repo.get_due_items(user_id, limit, surah_filter, is_high_yield_mode).await
    }

    pub async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        // Step 1: Process the main review
        let grade_for_log = grade.clone();
        let (new_state, energy_delta) = self.repo.process_review(user_id, node_id, grade).await?;

        let mut log_details: Vec<PropagationLogDetail> = Vec::new();

        if energy_delta.abs() > 1e-6 {
            let reason = format!("Direct({})", Self::grade_label(&grade_for_log));
            log_details.push(PropagationLogDetail {
                target_node_id: node_id.to_string(),
                energy_change: energy_delta,
                path: Some("Self".to_string()),
                reason: Some(reason),
            });
        }

        // Step 2: Propagate if the energy change is significant
        if energy_delta.abs() > Self::PROPAGATION_TRIGGER_DELTA {
            let outcome = crate::propagation::propagate_energy(
                &*self.repo,
                user_id,
                node_id,
                energy_delta as f32,
            )
            .await?;

            if !outcome.details.is_empty() {
                log_details.extend(outcome.details);
            }
        }

        if !log_details.is_empty() {
            let timestamp = Utc::now().timestamp();
            self.repo
                .log_propagation_event(node_id, timestamp, &log_details)
                .await?;
        }

        // Step 3: Update user stats (reviews today, streak)
        self.update_stats_after_review(user_id).await?;

        // Step 4: Remove item from session
        self.repo.remove_from_session(node_id).await?;

        Ok(new_state)
    }

    async fn update_stats_after_review(&self, user_id: &str) -> Result<()> {
        use chrono::NaiveDate;

        let stats = self.repo.get_dashboard_stats(user_id).await?;
        let today = Utc::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        // Get last review date from DB
        let last_date_str = self.get_last_review_date().await?;

        let (new_reviews_today, new_streak) = if let Some(last_str) = last_date_str {
            let last_date = NaiveDate::parse_from_str(&last_str, "%Y-%m-%d")
                .unwrap_or(today);

            let days_diff = (today - last_date).num_days();

            match days_diff {
                0 => {
                    // Same day - increment reviews, keep streak
                    (stats.reviews_today + 1, stats.streak_days)
                }
                1 => {
                    // Yesterday - reset reviews, increment streak
                    (1, stats.streak_days + 1)
                }
                _ => {
                    // Gap - reset both
                    (1, 1)
                }
            }
        } else {
            // First ever review
            (1, 1)
        };

        self.repo
            .update_user_stats(new_reviews_today, new_streak, &today_str)
            .await?;

        Ok(())
    }

    async fn get_last_review_date(&self) -> Result<Option<String>> {
        self.repo.get_last_review_date().await
    }

    fn grade_label(grade: &ReviewGrade) -> &'static str {
        match grade {
            ReviewGrade::Again => "Again",
            ReviewGrade::Hard => "Hard",
            ReviewGrade::Good => "Good",
            ReviewGrade::Easy => "Easy",
        }
    }

    pub async fn get_debug_stats(&self, user_id: &str) -> Result<DebugStats> {
        self.repo.get_debug_stats(user_id).await
    }

    pub async fn sync_user_nodes(&self, user_id: &str) -> Result<()> {
        self.repo.sync_user_nodes(user_id).await
    }

    pub async fn reset_user_progress(&self, user_id: &str) -> Result<()> {
        self.repo.reset_user_progress(user_id).await
    }

    pub async fn refresh_all_priority_scores(&self, user_id: &str) -> Result<()> {
        self.repo.refresh_all_priority_scores(user_id).await
    }

    pub async fn import_cbor_graph_from_bytes(&self, data: Vec<u8>) -> Result<ImportStats> {
        import_cbor_graph_from_bytes(&*self.repo, data).await
    }

    pub async fn get_session_preview(
        &self,
        user_id: &str,
        limit: u32,
        surah_filter: Option<i32>,
        is_high_yield_mode: bool,
    ) -> Result<Vec<ItemPreview>> {
        self.repo
            .get_session_preview(user_id, limit, surah_filter, is_high_yield_mode)
            .await
    }

    pub async fn get_available_surahs(&self) -> Result<Vec<(i32, String)>> {
        self.repo.get_available_surahs().await
    }

    pub async fn get_word_instance_context(
        &self,
        node_id: &str,
    ) -> Result<crate::repository::WordInstanceContext> {
        self.repo.get_word_instance_context(node_id).await
    }

    pub async fn search_nodes(&self, query: &str, limit: u32) -> Result<Vec<NodeData>> {
        self.repo.search_nodes(query, limit).await
    }

    pub async fn get_node_with_metadata(&self, node_id: &str) -> Result<Option<NodeData>> {
        self.repo.get_node_with_metadata(node_id).await
    }

    pub async fn query_propagation_details(
        &self,
        filter: PropagationQueryOptions,
    ) -> Result<Vec<PropagationDetailRecord>> {
        self.repo.query_propagation_details(filter).await
    }
}
