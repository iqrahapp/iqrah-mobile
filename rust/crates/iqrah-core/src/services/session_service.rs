use crate::domain::{node_id, KnowledgeAxis, KnowledgeNode, MemoryState, NodeType};
use crate::{ContentRepository, Node, UserRepository};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, instrument};

/// Scoring weights for session prioritization
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub w_due: f64,   // Weight for days overdue
    pub w_need: f64,  // Weight for mastery gap (1.0 - energy)
    pub w_yield: f64, // Weight for importance/yield
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            w_due: 1.0,
            w_need: 2.0,
            w_yield: 1.5, // Default for foundational mode
        }
    }
}

/// Scored item for session generation
#[derive(Debug, Clone)]
pub struct ScoredItem {
    pub node: Node,
    pub memory_state: MemoryState,
    pub priority_score: f64,
    pub days_overdue: f64,
    pub mastery_gap: f64,
    /// Knowledge axis if this is a knowledge node (Phase 4)
    pub knowledge_axis: Option<KnowledgeAxis>,
    /// Session composition budget assignment.
    pub session_budget: SessionBudget,
    /// Lexical fragility priority when applicable.
    pub lexical_priority: Option<f64>,
}

/// Session composition budget buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionBudget {
    Continuity,
    DueReview,
    Lexical,
}

impl SessionBudget {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionBudget::Continuity => "continuity",
            SessionBudget::DueReview => "due_review",
            SessionBudget::Lexical => "lexical",
        }
    }
}

/// Session service handles session generation and scoring
pub struct SessionService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl SessionService {
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            content_repo,
            user_repo,
        }
    }

    /// Get due items for a session with priority scoring
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    /// * `now` - Reference instant for computing days-overdue (injectable for deterministic tests)
    /// * `limit` - Maximum number of items to return
    /// * `is_high_yield_mode` - Whether to emphasize high-influence nodes
    /// * `axis_filter` - Optional knowledge axis to filter by (Phase 4)
    #[instrument(skip(self), fields(user_id, limit, is_high_yield_mode))]
    pub async fn get_due_items(
        &self,
        user_id: &str,
        now: DateTime<Utc>,
        limit: u32,
        is_high_yield_mode: bool,
        axis_filter: Option<KnowledgeAxis>,
    ) -> Result<Vec<ScoredItem>> {
        self.get_due_items_for_goal(user_id, now, limit, is_high_yield_mode, None, axis_filter)
            .await
    }

    /// Get due items with optional goal/chunk scope.
    #[instrument(skip(self), fields(user_id, limit, is_high_yield_mode, goal_id = goal_id.unwrap_or("none")))]
    pub async fn get_due_items_for_goal(
        &self,
        user_id: &str,
        now: DateTime<Utc>,
        limit: u32,
        is_high_yield_mode: bool,
        goal_id: Option<&str>,
        axis_filter: Option<KnowledgeAxis>,
    ) -> Result<Vec<ScoredItem>> {
        debug!("Fetching due items");
        if limit == 0 {
            return Ok(Vec::new());
        }

        let weights = if is_high_yield_mode {
            ScoreWeights {
                w_due: 1.0,
                w_need: 2.0,
                w_yield: 10.0,
            }
        } else {
            ScoreWeights::default()
        };

        let goal_scope = self.resolve_goal_scope(goal_id).await?;

        let due_states = self
            .user_repo
            .get_due_states(user_id, now, limit * 3)
            .await?;

        let mut best_by_node: HashMap<i64, MemoryState> = HashMap::new();
        for state in due_states {
            best_by_node
                .entry(state.node_id)
                .and_modify(|existing| {
                    if state.due_at < existing.due_at {
                        *existing = state.clone();
                    }
                })
                .or_insert(state);
        }

        let mut candidates: HashMap<i64, ScoredItem> = HashMap::new();

        for state in best_by_node.into_values() {
            let node = match self.content_repo.get_node(state.node_id).await? {
                Some(node) => node,
                None => continue,
            };

            if !matches!(
                node.node_type,
                NodeType::Word
                    | NodeType::WordInstance
                    | NodeType::Verse
                    | NodeType::Knowledge
                    | NodeType::Root
                    | NodeType::Lemma
            ) {
                continue;
            }

            if !goal_scope.matches(&node) {
                continue;
            }

            let knowledge_axis = resolve_knowledge_axis(&node);

            if let Some(filter_axis) = axis_filter {
                match knowledge_axis {
                    Some(axis) if axis == filter_axis => {}
                    _ => continue,
                }
            }

            let days_overdue = (now.timestamp_millis() - state.due_at.timestamp_millis()) as f64
                / (24.0 * 60.0 * 60.0 * 1000.0);
            let days_overdue = days_overdue.max(0.0);
            let mastery_gap = (1.0 - state.energy).max(0.0);
            let importance = importance_for_node_type(node.node_type);
            let priority_score = weights.w_due * days_overdue
                + weights.w_need * mastery_gap
                + weights.w_yield * importance;
            let lexical_priority = self
                .compute_lexical_priority(&node, &state, mastery_gap, days_overdue)
                .await?;

            candidates.insert(
                node.id,
                ScoredItem {
                    node,
                    memory_state: state,
                    priority_score,
                    days_overdue,
                    mastery_gap,
                    knowledge_axis,
                    session_budget: SessionBudget::DueReview,
                    lexical_priority,
                },
            );
        }

        if candidates.len() < (limit as usize) {
            let needed = (limit as usize) - candidates.len();
            let fetch_limit = (needed + limit as usize * 2).max(needed + 20) as u32;

            if let Ok(default_nodes) = self.content_repo.get_default_intro_nodes(fetch_limit).await
            {
                for node in default_nodes {
                    if candidates.len() >= (limit as usize) * 3 {
                        break;
                    }

                    if candidates.contains_key(&node.id) || !goal_scope.matches(&node) {
                        continue;
                    }

                    if let Ok(Some(_)) = self.user_repo.get_memory_state(user_id, node.id).await {
                        continue;
                    }

                    let knowledge_axis = resolve_knowledge_axis(&node);
                    if let Some(filter_axis) = axis_filter {
                        match knowledge_axis {
                            Some(axis) if axis == filter_axis => {}
                            _ => continue,
                        }
                    }

                    let state = MemoryState::new_for_node(user_id.to_string(), node.id);
                    let days_overdue = 0.0;
                    let mastery_gap = 1.0;
                    let importance = importance_for_node_type(node.node_type);
                    let priority_score =
                        weights.w_need * mastery_gap + weights.w_yield * importance;
                    let lexical_priority = self
                        .compute_lexical_priority(&node, &state, mastery_gap, days_overdue)
                        .await?;
                    let session_budget = if is_lexical_candidate(&node, knowledge_axis) {
                        SessionBudget::Lexical
                    } else {
                        SessionBudget::Continuity
                    };

                    candidates.insert(
                        node.id,
                        ScoredItem {
                            node,
                            memory_state: state,
                            priority_score,
                            days_overdue,
                            mastery_gap,
                            knowledge_axis,
                            session_budget,
                            lexical_priority,
                        },
                    );
                }
            }
        }

        let mut all_candidates: Vec<ScoredItem> = candidates.into_values().collect();
        if all_candidates.is_empty() {
            return Ok(Vec::new());
        }

        all_candidates.sort_by(desc_priority);

        let (continuity_target, due_target, lexical_target) =
            session_budget_targets(limit as usize);

        let mut selected = Vec::new();
        let mut selected_ids: HashSet<i64> = HashSet::new();

        let mut due_pool: Vec<ScoredItem> = all_candidates
            .iter()
            .filter(|item| {
                item.session_budget == SessionBudget::DueReview
                    || item.days_overdue > 0.0
                    || item.memory_state.review_count > 0
            })
            .cloned()
            .collect();
        due_pool.sort_by(desc_priority);
        if due_pool.is_empty() {
            due_pool = all_candidates.clone();
        }

        let mut continuity_pool: Vec<ScoredItem> = all_candidates
            .iter()
            .filter(|item| is_continuity_candidate(&item.node, item.knowledge_axis))
            .cloned()
            .collect();
        continuity_pool.sort_by(desc_priority);
        if continuity_pool.is_empty() {
            continuity_pool = all_candidates.clone();
        }

        let mut lexical_pool: Vec<ScoredItem> = all_candidates
            .iter()
            .filter(|item| is_lexical_candidate(&item.node, item.knowledge_axis))
            .cloned()
            .collect();
        lexical_pool.sort_by(desc_lexical_then_priority);
        if lexical_pool.is_empty() {
            lexical_pool = all_candidates.clone();
            lexical_pool.sort_by(desc_lexical_then_priority);
        }

        take_from_pool(
            &mut selected,
            &mut selected_ids,
            &continuity_pool,
            continuity_target,
            SessionBudget::Continuity,
        );
        take_from_pool(
            &mut selected,
            &mut selected_ids,
            &lexical_pool,
            lexical_target,
            SessionBudget::Lexical,
        );
        take_from_pool(
            &mut selected,
            &mut selected_ids,
            &due_pool,
            due_target,
            SessionBudget::DueReview,
        );

        if selected.len() < (limit as usize) {
            for item in all_candidates {
                if selected.len() >= (limit as usize) {
                    break;
                }
                if selected_ids.insert(item.node.id) {
                    selected.push(item);
                }
            }
        }

        selected.truncate(limit as usize);
        Ok(selected)
    }

    async fn resolve_goal_scope(&self, goal_id: Option<&str>) -> Result<GoalScope> {
        let Some(raw_goal_id) = goal_id else {
            return Ok(GoalScope::default());
        };

        let normalized = raw_goal_id.trim();
        if normalized.is_empty() || normalized == "daily_review" {
            return Ok(GoalScope::default());
        }

        let scoped_nodes = self.content_repo.get_nodes_for_goal(normalized).await?;
        if !scoped_nodes.is_empty() {
            return Ok(GoalScope {
                allowed_node_ids: Some(scoped_nodes.into_iter().collect()),
                chapter_scope: None,
            });
        }

        Ok(GoalScope {
            allowed_node_ids: None,
            chapter_scope: extract_chapter_scope(normalized),
        })
    }

    async fn compute_lexical_priority(
        &self,
        node: &Node,
        state: &MemoryState,
        mastery_gap: f64,
        days_overdue: f64,
    ) -> Result<Option<f64>> {
        let axis = resolve_knowledge_axis(node);
        if !is_lexical_candidate(node, axis) {
            return Ok(None);
        }

        let fragility = (mastery_gap
            + if state.review_count == 0 { 0.35 } else { 0.0 }
            + (days_overdue.clamp(0.0, 14.0) / 28.0))
            .max(0.1);

        let frequency_weight = self
            .metadata_weight(
                node.id,
                &[
                    "frequency_weight",
                    "lexical_frequency_weight",
                    "word_frequency_weight",
                    "word_frequency",
                ],
            )
            .await?
            .unwrap_or(1.0);

        let spread_weight = self
            .metadata_weight(
                node.id,
                &[
                    "spread_weight",
                    "cross_surah_spread",
                    "lexical_spread_weight",
                ],
            )
            .await?
            .unwrap_or(1.0);

        let prayer_boost = self
            .metadata_weight(node.id, &["prayer_boost", "salah_boost"])
            .await?
            .unwrap_or_else(|| default_prayer_boost(node));

        Ok(Some(
            fragility.max(0.05)
                * frequency_weight.max(0.1)
                * spread_weight.max(0.1)
                * prayer_boost.max(0.1),
        ))
    }

    async fn metadata_weight(&self, node_id: i64, keys: &[&str]) -> Result<Option<f64>> {
        for key in keys {
            if let Some(value) = self.content_repo.get_metadata(node_id, key).await? {
                if let Ok(parsed) = value.parse::<f64>() {
                    return Ok(Some(parsed));
                }
            }
        }
        Ok(None)
    }

    /// Get session state (for resuming)
    pub async fn get_session_state(&self) -> Result<Vec<i64>> {
        self.user_repo.get_session_state().await
    }

    /// Save session state (for persistence)
    pub async fn save_session_state(&self, node_ids: &[i64]) -> Result<()> {
        self.user_repo.save_session_state(node_ids).await
    }

    /// Clear session state
    pub async fn clear_session_state(&self) -> Result<()> {
        self.user_repo.clear_session_state().await
    }

    /// Get user statistics
    pub async fn get_stat(&self, key: &str) -> Result<Option<String>> {
        self.user_repo.get_stat(key).await
    }

    /// Set user statistics
    pub async fn set_stat(&self, key: &str, value: &str) -> Result<()> {
        self.user_repo.set_stat(key, value).await
    }

    /// Increment a stat (like reviews_today)
    pub async fn increment_stat(&self, key: &str) -> Result<u32> {
        let current = self
            .get_stat(key)
            .await?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let new_value = current + 1;
        self.set_stat(key, &new_value.to_string()).await?;

        Ok(new_value)
    }
}

#[derive(Debug, Default)]
struct GoalScope {
    allowed_node_ids: Option<HashSet<i64>>,
    chapter_scope: Option<u8>,
}

impl GoalScope {
    fn matches(&self, node: &Node) -> bool {
        if let Some(ids) = &self.allowed_node_ids {
            return ids.contains(&node.id);
        }

        if let Some(chapter) = self.chapter_scope {
            return chapter_for_node(node).is_some_and(|value| value == chapter);
        }

        true
    }
}

fn resolve_knowledge_axis(node: &Node) -> Option<KnowledgeAxis> {
    KnowledgeNode::parse(&node.ukey).map(|kn| kn.axis).or(
        if matches!(node.node_type, NodeType::Verse) {
            Some(KnowledgeAxis::Memorization)
        } else {
            None
        },
    )
}

fn importance_for_node_type(node_type: NodeType) -> f64 {
    match node_type {
        NodeType::Word | NodeType::WordInstance | NodeType::Root | NodeType::Lemma => 0.65,
        NodeType::Knowledge => 0.6,
        NodeType::Verse => 0.35,
        NodeType::Chapter => 0.3,
    }
}

fn chapter_for_node(node: &Node) -> Option<u8> {
    if let Some((chapter, _)) = node_id::decode_verse(node.id) {
        return Some(chapter);
    }
    if let Some((chapter, _, _)) = node_id::decode_word_instance(node.id) {
        return Some(chapter);
    }
    if let Some(chapter) = node_id::decode_chapter(node.id) {
        return Some(chapter);
    }

    if let Some(base) = node.ukey.strip_prefix("VERSE:") {
        return node_id::parse_verse(&format!("VERSE:{base}"))
            .ok()
            .map(|(chapter, _)| chapter);
    }
    if node.ukey.starts_with("WORD_INSTANCE:") {
        return node_id::parse_word_instance(&node.ukey)
            .ok()
            .map(|(chapter, _, _)| chapter);
    }

    None
}

fn extract_chapter_scope(goal_id: &str) -> Option<u8> {
    if let Some(value) = goal_id.strip_prefix("surah:") {
        if let Ok(chapter) = value.parse::<u8>() {
            return Some(chapter);
        }
    }

    if let Some(idx) = goal_id.find("surah-") {
        let suffix = &goal_id[idx + "surah-".len()..];
        let chapter_str: String = suffix.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(chapter) = chapter_str.parse::<u8>() {
            return Some(chapter);
        }
    }

    None
}

fn is_continuity_candidate(node: &Node, axis: Option<KnowledgeAxis>) -> bool {
    if matches!(node.node_type, NodeType::Verse | NodeType::Chapter) {
        return true;
    }

    matches!(
        axis,
        Some(KnowledgeAxis::Memorization | KnowledgeAxis::ContextualMemorization)
    )
}

fn is_lexical_candidate(node: &Node, axis: Option<KnowledgeAxis>) -> bool {
    if matches!(
        node.node_type,
        NodeType::Word | NodeType::WordInstance | NodeType::Root | NodeType::Lemma
    ) {
        return true;
    }

    matches!(
        axis,
        Some(KnowledgeAxis::Translation | KnowledgeAxis::Meaning | KnowledgeAxis::Tafsir)
    )
}

fn default_prayer_boost(node: &Node) -> f64 {
    if chapter_for_node(node).is_some_and(|chapter| matches!(chapter, 1 | 112 | 113 | 114)) {
        1.4
    } else {
        1.0
    }
}

fn desc_priority(a: &ScoredItem, b: &ScoredItem) -> std::cmp::Ordering {
    b.priority_score
        .partial_cmp(&a.priority_score)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn desc_lexical_then_priority(a: &ScoredItem, b: &ScoredItem) -> std::cmp::Ordering {
    match b
        .lexical_priority
        .unwrap_or(0.0)
        .partial_cmp(&a.lexical_priority.unwrap_or(0.0))
        .unwrap_or(std::cmp::Ordering::Equal)
    {
        std::cmp::Ordering::Equal => desc_priority(a, b),
        other => other,
    }
}

fn take_from_pool(
    selected: &mut Vec<ScoredItem>,
    selected_ids: &mut HashSet<i64>,
    pool: &[ScoredItem],
    target: usize,
    budget: SessionBudget,
) {
    let mut added = 0usize;
    for item in pool {
        if added >= target {
            break;
        }

        if selected_ids.insert(item.node.id) {
            let mut adjusted = item.clone();
            adjusted.session_budget = budget;
            selected.push(adjusted);
            added += 1;
        }
    }
}

fn session_budget_targets(limit: usize) -> (usize, usize, usize) {
    if limit == 0 {
        return (0, 0, 0);
    }
    if limit == 1 {
        return (1, 0, 0);
    }
    if limit == 2 {
        return (1, 1, 0);
    }

    let mut continuity = ((limit as f64) * 0.4).round() as usize;
    let mut due_review = ((limit as f64) * 0.3).round() as usize;
    let mut lexical = limit.saturating_sub(continuity + due_review);

    continuity = continuity.max(1);
    due_review = due_review.max(1);
    lexical = lexical.max(1);

    while continuity + due_review + lexical > limit {
        if continuity >= due_review && continuity >= lexical && continuity > 1 {
            continuity -= 1;
        } else if due_review >= continuity && due_review >= lexical && due_review > 1 {
            due_review -= 1;
        } else if lexical > 1 {
            lexical -= 1;
        } else {
            break;
        }
    }

    while continuity + due_review + lexical < limit {
        lexical += 1;
    }

    (continuity, due_review, lexical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{MockContentRepository, MockUserRepository};
    use crate::{MemoryState, Node, NodeType};
    use chrono::{DateTime, Duration, Utc};
    use std::sync::Arc;

    /// Helper to create a mock ContentRepository with basic node setup
    fn create_content_mock() -> MockContentRepository {
        let mut mock = MockContentRepository::new();

        // Setup get_node for nodes 1-4 of different types
        mock.expect_get_node().returning(|node_id| {
            let node_type = match node_id {
                1 => NodeType::WordInstance,
                2 => NodeType::WordInstance,
                3 => NodeType::Verse,
                4 => NodeType::Chapter,
                _ => NodeType::WordInstance,
            };
            Ok(Some(Node {
                id: node_id,
                ukey: format!("node_{}", node_id),
                node_type,
            }))
        });

        mock.expect_get_default_intro_nodes()
            .returning(|_| Ok(vec![]));
        mock.expect_get_metadata().returning(|_, _| Ok(None));

        mock
    }

    /// Helper to create a mock UserRepository with configurable due states
    fn create_user_mock_with_due_states(states: Vec<MemoryState>) -> MockUserRepository {
        let mut mock = MockUserRepository::new();

        // Clone states for get_due_states
        let states_clone = states.clone();
        mock.expect_get_due_states()
            .returning(move |_, _, _| Ok(states_clone.clone()));

        mock.expect_get_memory_state().returning(|_, _| Ok(None));

        // Session state management
        let session_state = std::sync::Arc::new(std::sync::Mutex::new(Vec::<i64>::new()));
        let session_state_save = session_state.clone();
        let session_state_get = session_state.clone();
        let session_state_clear = session_state.clone();

        mock.expect_save_session_state().returning(move |node_ids| {
            let mut state = session_state_save.lock().unwrap();
            *state = node_ids.to_vec();
            Ok(())
        });

        mock.expect_get_session_state().returning(move || {
            let state = session_state_get.lock().unwrap();
            Ok(state.clone())
        });

        mock.expect_clear_session_state().returning(move || {
            let mut state = session_state_clear.lock().unwrap();
            state.clear();
            Ok(())
        });

        // Stat management
        let stats = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
            String,
            String,
        >::new()));
        let stats_set = stats.clone();
        let stats_get = stats.clone();

        mock.expect_set_stat().returning(move |key, value| {
            let mut s = stats_set.lock().unwrap();
            s.insert(key.to_string(), value.to_string());
            Ok(())
        });

        mock.expect_get_stat().returning(move |key| {
            let s = stats_get.lock().unwrap();
            Ok(s.get(key).cloned())
        });

        mock
    }

    #[tokio::test]
    async fn test_get_due_items_returns_items() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 1,
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.3,
            last_reviewed: now,
            due_at: now,
            review_count: 3,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].node.id, 1);
    }

    #[tokio::test]
    async fn test_filters_by_node_type() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        // Create states with different node types (1=WordInstance, 3=Verse, 4=Chapter)
        let states = vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 1, // WordInstance
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 3, // Verse
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 2,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 4, // Chapter (should be filtered out)
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
        ];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2, "Should filter out Chapter type");

        for item in &items {
            assert!(
                matches!(
                    item.node.node_type,
                    NodeType::WordInstance | NodeType::Verse
                ),
                "Only WordInstance and Verse should be included"
            );
        }
    }

    #[tokio::test]
    async fn test_defaults_axis_for_verse_nodes() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 3, // Verse
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.3,
            last_reviewed: now,
            due_at: now,
            review_count: 2,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].knowledge_axis, Some(KnowledgeAxis::Memorization));
    }

    #[tokio::test]
    async fn test_sorts_by_priority_descending() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();
        let very_overdue = now - Duration::try_days(10).unwrap();
        let slightly_overdue = now - Duration::try_days(1).unwrap();

        // Node 2 should have higher priority (low energy + very overdue)
        let states = vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 1,
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.9, // High energy, low need
                last_reviewed: slightly_overdue,
                due_at: slightly_overdue,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 2,
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.1, // Low energy, high need
                last_reviewed: very_overdue,
                due_at: very_overdue,
                review_count: 1,
            },
        ];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].node.id, 2,
            "Node 2 should be first (higher priority)"
        );
        assert!(items[0].priority_score > items[1].priority_score);
    }

    #[tokio::test]
    async fn test_respects_limit() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 1,
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 2,
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.6,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
        ];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 1, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1, "Should respect limit parameter");
    }

    #[tokio::test]
    async fn test_session_state_management() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
        let service = SessionService::new(content_repo, user_repo);

        let node_ids = vec![1, 2];

        // Act - Save session state
        let save_result = service.save_session_state(&node_ids).await;
        assert!(save_result.is_ok());

        // Act - Get session state
        let get_result = service.get_session_state().await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), node_ids);

        // Act - Clear session state
        let clear_result = service.clear_session_state().await;
        assert!(clear_result.is_ok());

        let empty_result = service.get_session_state().await;
        assert!(empty_result.is_ok());
        assert_eq!(empty_result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_stat_management() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
        let service = SessionService::new(content_repo, user_repo);

        // Act - Set stat
        let set_result = service.set_stat("reviews_today", "5").await;
        assert!(set_result.is_ok());

        // Act - Get stat
        let get_result = service.get_stat("reviews_today").await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), Some("5".to_string()));

        // Act - Get non-existent stat
        let none_result = service.get_stat("nonexistent").await;
        assert!(none_result.is_ok());
        assert_eq!(none_result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_increment_stat() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
        let service = SessionService::new(content_repo, user_repo);

        // Act - Increment from 0
        let first_result = service.increment_stat("reviews_today").await;
        assert!(first_result.is_ok());
        assert_eq!(first_result.unwrap(), 1);

        // Act - Increment again
        let second_result = service.increment_stat("reviews_today").await;
        assert!(second_result.is_ok());
        assert_eq!(second_result.unwrap(), 2);

        // Verify final value
        let get_result = service.get_stat("reviews_today").await;
        assert_eq!(get_result.unwrap(), Some("2".to_string()));
    }

    #[tokio::test]
    async fn test_calculates_mastery_gap_correctly() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 1,
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.2, // mastery_gap should be 0.8
            last_reviewed: now,
            due_at: now,
            review_count: 3,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);

        let mastery_gap = items[0].mastery_gap;
        assert!(
            (mastery_gap - 0.8).abs() < 0.001,
            "Mastery gap should be 1.0 - energy"
        );
    }

    #[tokio::test]
    async fn test_calculates_days_overdue_correctly() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();
        let three_days_ago = now - Duration::try_days(3).unwrap();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 1,
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.5,
            last_reviewed: three_days_ago,
            due_at: three_days_ago,
            review_count: 3,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);

        let days_overdue = items[0].days_overdue;
        assert!(
            (2.9..=3.1).contains(&days_overdue),
            "Days overdue should be approximately 3, got {}",
            days_overdue
        );
    }

    // ========================================================================
    // C-001: Deterministic Golden Session Scenarios
    //
    // These tests use fixed input data and assert exact outputs.
    // They serve as regression anchors: any behavioral change will break them.
    // Scenarios: cold-start, due-review, chunk/axis-filtered mode.
    // ========================================================================
    mod golden_scenarios {
        use super::*;

        /// Golden scenario 1: Cold-start — brand-new user with zero memory states.
        ///
        /// C-003 requirement: session must never be empty on cold start.
        #[tokio::test]
        async fn test_golden_cold_start_non_empty_session() {
            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|_| Ok(None));
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| {
                    Ok(vec![
                        Node {
                            id: 200,
                            ukey: "VERSE:1:1".to_string(),
                            node_type: NodeType::Verse,
                        },
                        Node {
                            id: 201,
                            ukey: "WORD_INSTANCE:1:1:1".to_string(),
                            node_type: NodeType::WordInstance,
                        },
                    ])
                });

            let content_repo = Arc::new(content_mock);
            let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
            let service = SessionService::new(content_repo, user_repo);
            let now = Utc::now();

            let result = service
                .get_due_items("new_user", now, 20, false, None)
                .await;

            assert!(result.is_ok());
            let items = result.unwrap();
            assert!(
                !items.is_empty(),
                "Cold-start must always return a non-empty session (C-003)"
            );
            assert!(
                items
                    .iter()
                    .any(|item| item.session_budget == SessionBudget::Continuity),
                "Cold-start session should include continuity budget items"
            );
            assert!(
                items
                    .iter()
                    .any(|item| item.session_budget == SessionBudget::Lexical),
                "Cold-start session should include lexical budget items"
            );
        }

        /// Golden scenario 2: Due-review — deterministic priority ordering with fixed data.
        ///
        /// Three WordInstance nodes with known energies and overdue durations are
        /// scored with default weights (w_due=1.0, w_need=2.0, w_yield=1.5).
        /// Expected priority scores (importance=0.65 for WordInstance):
        ///   Node 10: 1.0×7 + 2.0×0.9 + 1.5×0.65 = 9.775  (highest)
        ///   Node 11: 1.0×3 + 2.0×0.5 + 1.5×0.65 = 4.975  (medium)
        ///   Node 12: 1.0×1 + 2.0×0.2 + 1.5×0.65 = 2.375  (lowest)
        #[tokio::test]
        async fn test_golden_due_review_deterministic_priority() {
            // Fixed reference point — makes priority scores fully deterministic across
            // any CI environment. Using a pinned Unix timestamp avoids wall-clock drift.
            let fixed_now = DateTime::from_timestamp(1_740_000_000, 0).unwrap();
            let seven_days_ago = fixed_now - Duration::try_days(7).unwrap();
            let three_days_ago = fixed_now - Duration::try_days(3).unwrap();
            let one_day_ago = fixed_now - Duration::try_days(1).unwrap();

            let states = vec![
                MemoryState {
                    user_id: "user_golden".to_string(),
                    node_id: 10,
                    stability: 5.0,
                    difficulty: 6.0,
                    energy: 0.1, // mastery_gap = 0.9
                    last_reviewed: seven_days_ago,
                    due_at: seven_days_ago, // overdue by exactly 7 days at fixed_now
                    review_count: 2,
                },
                MemoryState {
                    user_id: "user_golden".to_string(),
                    node_id: 11,
                    stability: 8.0,
                    difficulty: 5.0,
                    energy: 0.5, // mastery_gap = 0.5
                    last_reviewed: three_days_ago,
                    due_at: three_days_ago, // overdue by exactly 3 days at fixed_now
                    review_count: 5,
                },
                MemoryState {
                    user_id: "user_golden".to_string(),
                    node_id: 12,
                    stability: 12.0,
                    difficulty: 4.0,
                    energy: 0.8, // mastery_gap = 0.2
                    last_reviewed: one_day_ago,
                    due_at: one_day_ago, // overdue by exactly 1 day at fixed_now
                    review_count: 10,
                },
            ];

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|node_id| {
                if matches!(node_id, 10..=12) {
                    Ok(Some(Node {
                        id: node_id,
                        ukey: format!("WORD_INSTANCE:1:{}:1", node_id),
                        node_type: NodeType::WordInstance,
                    }))
                } else {
                    Ok(None)
                }
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| Ok(vec![]));
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service
                .get_due_items("user_golden", fixed_now, 10, false, None)
                .await;
            assert!(result.is_ok());
            let items = result.unwrap();

            // GOLDEN: all three items returned
            assert_eq!(items.len(), 3, "All three due items must be returned");

            // GOLDEN: exact ordering by priority score
            assert_eq!(
                items[0].node.id, 10,
                "Node 10 must rank first (7 days overdue, energy=0.1)"
            );
            assert_eq!(
                items[1].node.id, 11,
                "Node 11 must rank second (3 days overdue, energy=0.5)"
            );
            assert_eq!(
                items[2].node.id, 12,
                "Node 12 must rank last (1 day overdue, energy=0.8)"
            );

            // GOLDEN: priority scores are strictly descending
            assert!(
                items[0].priority_score > items[1].priority_score,
                "Score[0]={:.3} must exceed score[1]={:.3}",
                items[0].priority_score,
                items[1].priority_score
            );
            assert!(
                items[1].priority_score > items[2].priority_score,
                "Score[1]={:.3} must exceed score[2]={:.3}",
                items[1].priority_score,
                items[2].priority_score
            );

            // GOLDEN: approximate score values (regression guard)
            assert!(
                (items[0].priority_score - 9.775).abs() < 0.2,
                "Node 10 priority ≈ 9.775, got {:.3}",
                items[0].priority_score
            );
            assert!(
                (items[1].priority_score - 4.975).abs() < 0.2,
                "Node 11 priority ≈ 4.975, got {:.3}",
                items[1].priority_score
            );
            assert!(
                (items[2].priority_score - 2.375).abs() < 0.2,
                "Node 12 priority ≈ 2.375, got {:.3}",
                items[2].priority_score
            );
        }

        /// Golden scenario 3: Chunk/axis-filtered mode — Memorization axis only.
        ///
        /// Simulates a user selecting a contiguous Quran chunk for memorization practice.
        /// Only nodes whose axis resolves to Memorization are returned.
        ///
        /// Input nodes:
        ///   20: Verse         → auto-assigned Memorization axis  → INCLUDED
        ///   21: Knowledge     → ukey ends in ":memorization"     → INCLUDED
        ///   22: Knowledge     → ukey ends in ":translation"      → EXCLUDED
        ///   23: WordInstance  → no axis suffix                   → EXCLUDED
        #[tokio::test]
        async fn test_golden_chunk_mode_memorization_axis_filter() {
            let now = Utc::now();

            let states: Vec<MemoryState> = vec![20_i64, 21, 22, 23]
                .into_iter()
                .map(|id| MemoryState {
                    user_id: "chunk_user".to_string(),
                    node_id: id,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                let (ukey, node_type) = match id {
                    20 => ("VERSE:1:1".to_string(), NodeType::Verse),
                    21 => (
                        "WORD_INSTANCE:1:1:1:memorization".to_string(),
                        NodeType::Knowledge,
                    ),
                    22 => ("VERSE:1:2:translation".to_string(), NodeType::Knowledge),
                    23 => ("WORD_INSTANCE:1:1:1".to_string(), NodeType::WordInstance),
                    _ => return Ok(None),
                };
                Ok(Some(Node {
                    id,
                    ukey,
                    node_type,
                }))
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| Ok(vec![]));
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service
                .get_due_items(
                    "chunk_user",
                    now,
                    10,
                    false,
                    Some(KnowledgeAxis::Memorization),
                )
                .await;
            assert!(result.is_ok());
            let items = result.unwrap();

            let returned_ids: Vec<i64> = items.iter().map(|i| i.node.id).collect();

            // GOLDEN: exact inclusion/exclusion
            assert!(
                returned_ids.contains(&20),
                "Node 20 (Verse → auto-Memorization) must be included"
            );
            assert!(
                returned_ids.contains(&21),
                "Node 21 (Knowledge:memorization) must be included"
            );
            assert!(
                !returned_ids.contains(&22),
                "Node 22 (Knowledge:translation) must be excluded in Memorization chunk mode"
            );
            assert!(
                !returned_ids.contains(&23),
                "Node 23 (WordInstance, no axis) must be excluded in Memorization chunk mode"
            );
            assert_eq!(
                items.len(),
                2,
                "Exactly 2 nodes match the Memorization axis filter"
            );

            // GOLDEN: every returned item carries the Memorization axis
            for item in &items {
                assert_eq!(
                    item.knowledge_axis,
                    Some(KnowledgeAxis::Memorization),
                    "Every item in chunk mode must carry the requested axis"
                );
            }
        }
    }

    // ========================================================================
    // C-004/C-005/C-006 Runtime Behavior Tests
    // ========================================================================
    mod core_fixes {
        use super::*;

        #[tokio::test]
        async fn test_goal_scope_changes_candidate_pool() {
            let now = Utc::now();
            let states = vec![
                MemoryState {
                    user_id: "goal_user".to_string(),
                    node_id: 500,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.4,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 2,
                },
                MemoryState {
                    user_id: "goal_user".to_string(),
                    node_id: 501,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.4,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 2,
                },
            ];

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                Ok(Some(Node {
                    id,
                    ukey: format!("VERSE:1:{}", id),
                    node_type: NodeType::Verse,
                }))
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| Ok(vec![]));
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));
            content_mock
                .expect_get_nodes_for_goal()
                .returning(|goal_id| {
                    let scoped = match goal_id {
                        "goal_a" => vec![500],
                        "goal_b" => vec![501],
                        _ => vec![],
                    };
                    Ok(scoped)
                });

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let pool_a = service
                .get_due_items_for_goal("goal_user", now, 10, false, Some("goal_a"), None)
                .await
                .unwrap();
            let pool_b = service
                .get_due_items_for_goal("goal_user", now, 10, false, Some("goal_b"), None)
                .await
                .unwrap();

            let ids_a: Vec<i64> = pool_a.iter().map(|item| item.node.id).collect();
            let ids_b: Vec<i64> = pool_b.iter().map(|item| item.node.id).collect();

            assert_eq!(ids_a, vec![500], "goal_a should scope to node 500");
            assert_eq!(ids_b, vec![501], "goal_b should scope to node 501");
            assert_ne!(ids_a, ids_b, "Different goals must produce different pools");
        }

        #[tokio::test]
        async fn test_three_budget_composition_present_for_standard_session() {
            let now = Utc::now();
            let states = vec![
                MemoryState {
                    user_id: "budget_user".to_string(),
                    node_id: 700,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.3,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 3,
                },
                MemoryState {
                    user_id: "budget_user".to_string(),
                    node_id: 701,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 3,
                },
                MemoryState {
                    user_id: "budget_user".to_string(),
                    node_id: 702,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.6,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 2,
                },
            ];

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                let (ukey, node_type) = match id {
                    700 => ("VERSE:2:1".to_string(), NodeType::Verse),
                    701 => ("WORD_INSTANCE:2:1:1".to_string(), NodeType::WordInstance),
                    702 => ("WORD_INSTANCE:2:1:2".to_string(), NodeType::WordInstance),
                    _ => return Ok(None),
                };
                Ok(Some(Node {
                    id,
                    ukey,
                    node_type,
                }))
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| {
                    Ok(vec![
                        Node {
                            id: 703,
                            ukey: "VERSE:2:2".to_string(),
                            node_type: NodeType::Verse,
                        },
                        Node {
                            id: 704,
                            ukey: "WORD_INSTANCE:2:2:1".to_string(),
                            node_type: NodeType::WordInstance,
                        },
                    ])
                });
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let items = service
                .get_due_items("budget_user", now, 6, false, None)
                .await
                .unwrap();

            assert!(
                items
                    .iter()
                    .any(|item| item.session_budget == SessionBudget::Continuity),
                "Session must include continuity budget items (C-005)"
            );
            assert!(
                items
                    .iter()
                    .any(|item| item.session_budget == SessionBudget::DueReview),
                "Session must include due-review budget items (C-005)"
            );
            assert!(
                items
                    .iter()
                    .any(|item| item.session_budget == SessionBudget::Lexical),
                "Session must include lexical budget items (C-005)"
            );
        }

        #[tokio::test]
        async fn test_lexical_priority_uses_frequency_spread_and_prayer_boost() {
            let now = Utc::now();
            let states = vec![
                MemoryState {
                    user_id: "lex_user".to_string(),
                    node_id: 801,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.8,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "lex_user".to_string(),
                    node_id: 802,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.8,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "lex_user".to_string(),
                    node_id: 803,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.2,
                    last_reviewed: now - Duration::try_days(10).unwrap(),
                    due_at: now - Duration::try_days(10).unwrap(),
                    review_count: 5,
                },
            ];

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                let (ukey, node_type) = match id {
                    801 => ("WORD_INSTANCE:1:1:1".to_string(), NodeType::WordInstance),
                    802 => ("WORD_INSTANCE:2:1:1".to_string(), NodeType::WordInstance),
                    803 => ("VERSE:2:1".to_string(), NodeType::Verse),
                    _ => return Ok(None),
                };
                Ok(Some(Node {
                    id,
                    ukey,
                    node_type,
                }))
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| Ok(vec![]));
            content_mock
                .expect_get_metadata()
                .returning(|node_id, key| {
                    let value = match (node_id, key) {
                        (801, "frequency_weight") => Some("2.0".to_string()),
                        (801, "spread_weight") => Some("1.8".to_string()),
                        (802, "frequency_weight") => Some("1.0".to_string()),
                        (802, "spread_weight") => Some("1.0".to_string()),
                        _ => None,
                    };
                    Ok(value)
                });

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let items = service
                .get_due_items("lex_user", now, 4, false, None)
                .await
                .unwrap();

            let by_id: HashMap<i64, ScoredItem> =
                items.into_iter().map(|item| (item.node.id, item)).collect();
            let lexical_801 = by_id
                .get(&801)
                .and_then(|item| item.lexical_priority)
                .unwrap_or(0.0);
            let lexical_802 = by_id
                .get(&802)
                .and_then(|item| item.lexical_priority)
                .unwrap_or(0.0);

            assert!(
                lexical_801 > lexical_802,
                "Lexical score should prioritize higher frequency/spread/prayer value (C-006)"
            );
        }
    }

    // ========================================================================
    // C-002: Scheduler Invariants Test Suite
    //
    // These tests verify properties that must hold for ALL inputs:
    //   1. No duplicate node IDs in session output
    //   2. Session size never exceeds the requested limit
    //   3. Verse nodes always receive the Memorization axis
    //   4. Chapter nodes are never included in session output
    //   5. Axis filter excludes all non-matching nodes without exception
    // ========================================================================
    mod scheduler_invariants {
        use super::*;

        /// Invariant 1: No duplicate node IDs in session output.
        ///
        /// Even if the repository defensively returns duplicate states for the
        /// same node, the service must deduplicate before returning.
        #[tokio::test]
        async fn test_invariant_no_duplicate_node_ids() {
            let content_repo = Arc::new(create_content_mock());
            let now = Utc::now();

            // Simulate a repository returning the same node_id twice
            let states = vec![
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 1,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 1, // same node — duplicate
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 2,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
            ];

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(content_repo, user_repo);

            let result = service.get_due_items("user1", now, 10, false, None).await;
            assert!(result.is_ok());
            let items = result.unwrap();

            let ids: Vec<i64> = items.iter().map(|i| i.node.id).collect();
            let unique_count = ids
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<i64>>()
                .len();

            // INVARIANT: no duplicate node IDs
            assert_eq!(
                ids.len(),
                unique_count,
                "Session must not contain duplicate node IDs; got {:?}",
                ids
            );
            // Expect exactly 2 unique nodes (1 and 2), not 3
            assert_eq!(
                items.len(),
                2,
                "Duplicate state for node 1 must be collapsed to one item"
            );
        }

        /// Invariant 2: Session size never exceeds the requested limit.
        ///
        /// Tests multiple limit values against a pool of 20 candidate items.
        #[tokio::test]
        async fn test_invariant_limit_never_exceeded() {
            let content_repo = Arc::new(create_content_mock());
            let now = Utc::now();

            // 20 states with node IDs 1-20; node 4 is Chapter (filtered out → 19 valid)
            let states: Vec<MemoryState> = (1_i64..=20)
                .map(|i| MemoryState {
                    user_id: "user1".to_string(),
                    node_id: i,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(content_repo, user_repo);

            for limit in [1u32, 3, 5, 10, 15, 20] {
                let result = service
                    .get_due_items("user1", now, limit, false, None)
                    .await;
                assert!(result.is_ok(), "get_due_items failed for limit={}", limit);
                let items = result.unwrap();
                assert!(
                    items.len() <= limit as usize,
                    "INVARIANT VIOLATED: limit={} but got {} items",
                    limit,
                    items.len()
                );
            }
        }

        /// Invariant 3: Every Verse node always receives the Memorization axis.
        ///
        /// Verse nodes do not carry an explicit axis suffix in their ukey, so
        /// the service must assign Memorization as the default axis.
        #[tokio::test]
        async fn test_invariant_verse_always_memorization_axis() {
            let now = Utc::now();

            let states: Vec<MemoryState> = (30_i64..=34)
                .map(|i| MemoryState {
                    user_id: "user1".to_string(),
                    node_id: i,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                Ok(Some(Node {
                    id,
                    ukey: format!("VERSE:1:{}", id),
                    node_type: NodeType::Verse,
                }))
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| Ok(vec![]));
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service.get_due_items("user1", now, 10, false, None).await;
            assert!(result.is_ok());
            let items = result.unwrap();

            assert!(!items.is_empty(), "Verse items should be returned");
            assert_eq!(items.len(), 5, "All 5 Verse nodes should be included");

            // INVARIANT: every Verse carries the Memorization axis
            for item in &items {
                assert_eq!(
                    item.knowledge_axis,
                    Some(KnowledgeAxis::Memorization),
                    "Verse node {} must always have Memorization axis, got {:?}",
                    item.node.id,
                    item.knowledge_axis
                );
            }
        }

        /// Invariant 4: Chapter nodes are never included in session output.
        ///
        /// Chapter is not a reviewable node type and must be filtered out
        /// regardless of its position in the due-states list.
        #[tokio::test]
        async fn test_invariant_chapter_nodes_never_in_session() {
            // create_content_mock: node 4 → Chapter, nodes 1 and 3 → reviewable
            let content_repo = Arc::new(create_content_mock());
            let now = Utc::now();

            let states = vec![
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 1,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 4, // Chapter — must be excluded
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 3, // Verse — must be included
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
            ];

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(content_repo, user_repo);

            let result = service.get_due_items("user1", now, 10, false, None).await;
            assert!(result.is_ok());
            let items = result.unwrap();

            // INVARIANT: no Chapter node in session output
            for item in &items {
                assert_ne!(
                    item.node.node_type,
                    NodeType::Chapter,
                    "Chapter node {} must never appear in session output",
                    item.node.id
                );
            }
            // Only nodes 1 and 3 should pass through
            assert_eq!(
                items.len(),
                2,
                "Exactly 2 reviewable nodes (excluding Chapter) expected"
            );
        }

        /// Invariant 5: Axis filter excludes ALL non-matching nodes without exception.
        ///
        /// When an axis filter is active, only nodes whose resolved axis matches
        /// the filter are included. No node slips through on any other basis.
        #[tokio::test]
        async fn test_invariant_axis_filter_excludes_all_non_matching() {
            let now = Utc::now();

            // Four nodes covering all axis-resolution code paths:
            //   40: Verse           → Memorization (auto)   — matches filter
            //   41: Knowledge       → Translation (parsed)  — excluded
            //   42: WordInstance    → no axis               — excluded
            //   43: Knowledge       → Tafsir (parsed)       — excluded
            let states: Vec<MemoryState> = vec![40_i64, 41, 42, 43]
                .into_iter()
                .map(|id| MemoryState {
                    user_id: "user1".to_string(),
                    node_id: id,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                let (ukey, node_type) = match id {
                    40 => ("VERSE:1:1".to_string(), NodeType::Verse),
                    41 => ("VERSE:1:2:translation".to_string(), NodeType::Knowledge),
                    42 => ("WORD_INSTANCE:1:1:1".to_string(), NodeType::WordInstance),
                    43 => ("VERSE:2:1:tafsir".to_string(), NodeType::Knowledge),
                    _ => return Ok(None),
                };
                Ok(Some(Node {
                    id,
                    ukey,
                    node_type,
                }))
            });
            content_mock
                .expect_get_default_intro_nodes()
                .returning(|_| Ok(vec![]));
            content_mock
                .expect_get_metadata()
                .returning(|_, _| Ok(None));

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service
                .get_due_items("user1", now, 10, false, Some(KnowledgeAxis::Memorization))
                .await;
            assert!(result.is_ok());
            let items = result.unwrap();

            // INVARIANT: every returned item matches the axis filter exactly
            for item in &items {
                assert_eq!(
                    item.knowledge_axis,
                    Some(KnowledgeAxis::Memorization),
                    "Axis filter must exclude all non-matching items; node {} has axis {:?}",
                    item.node.id,
                    item.knowledge_axis
                );
            }

            // GOLDEN sub-check: only node 40 (Verse → Memorization) should pass
            assert_eq!(
                items.len(),
                1,
                "Only 1 node matches the Memorization axis filter"
            );
            assert_eq!(
                items[0].node.id, 40,
                "Only the Verse node (id=40) should pass the filter"
            );
        }
    }
}
