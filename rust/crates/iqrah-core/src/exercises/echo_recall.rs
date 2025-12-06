// exercises/echo_recall.rs
// Exercise 5: Echo Recall - Progressive word obscuring for memorization
//
// This is the core memorization exercise for Quran learning. Words are displayed
// with varying visibility based on learner's energy/mastery level.

use crate::domain::models::{EchoRecallState, EchoRecallStats, EchoRecallWord, WordVisibility};
use crate::domain::node_id as nid;
use crate::ports::{ContentRepository, UserRepository};
use crate::services::{energy_service, recall_model};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Stateful Echo Recall exercise for progressive word memorization
///
/// Words are displayed with context-aware visibility based on the learner's
/// mastery level (energy). As the user recalls words, their energy increases
/// and they transition from Visible → Obscured → Hidden.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoRecallExercise {
    /// Current state of all words in the session
    state: EchoRecallState,
    /// User ID for this session
    user_id: String,
    /// Ayah node IDs this session covers
    ayah_node_ids: Vec<String>,
}

impl EchoRecallExercise {
    /// Create a new Echo Recall exercise from a list of ayah node IDs
    ///
    /// Fetches all words from the specified ayahs, retrieves their current
    /// energy levels from the user repository, and calculates initial visibility.
    ///
    /// # Arguments
    /// * `user_id` - The user ID for fetching/saving memory states
    /// * `ayah_node_ids` - List of ayah node IDs (e.g., ["VERSE:1:1", "VERSE:1:2"])
    /// * `content_repo` - Repository for fetching word data
    /// * `user_repo` - Repository for fetching user memory states
    ///
    /// # Returns
    /// A new `EchoRecallExercise` with initialized word visibility
    pub async fn new(
        user_id: &str,
        ayah_node_ids: Vec<String>,
        content_repo: &dyn ContentRepository,
        user_repo: &dyn UserRepository,
    ) -> Result<Self> {
        // Convert ukeys to node IDs
        let ayah_ids: Vec<i64> = ayah_node_ids
            .iter()
            .filter_map(|id| nid::from_ukey(id))
            .collect();

        if ayah_ids.is_empty() {
            return Err(anyhow!("No valid ayah node IDs provided"));
        }

        // Fetch all words in the specified ayahs
        let words = content_repo.get_words_in_ayahs(&ayah_ids).await?;

        if words.is_empty() {
            return Err(anyhow!("No words found in specified ayahs"));
        }

        // Build energy map for all words
        let mut energy_map: HashMap<i64, f64> = HashMap::new();
        for word in &words {
            let memory_state = user_repo
                .get_memory_state(user_id, word.id)
                .await
                .ok()
                .flatten();
            let energy = memory_state.map(|s| s.energy).unwrap_or(0.0);
            energy_map.insert(word.id, energy);
        }

        // Calculate visibility for each word with neighbor context
        let mut echo_recall_words = Vec::with_capacity(words.len());
        for (i, word) in words.iter().enumerate() {
            let word_text = content_repo
                .get_quran_text(word.id)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();

            let energy = *energy_map.get(&word.id).unwrap_or(&0.0);

            // Get neighbor energies for context-aware hint selection
            let prev_energy = if i > 0 {
                energy_map.get(&words[i - 1].id).copied()
            } else {
                None
            };

            let next_energy = if i < words.len() - 1 {
                energy_map.get(&words[i + 1].id).copied()
            } else {
                None
            };

            // Calculate context-aware visibility
            let visibility = energy_service::map_energy_to_visibility(
                energy,
                &word_text,
                prev_energy,
                next_energy,
            );

            echo_recall_words.push(EchoRecallWord {
                node_id: nid::to_ukey(word.id).unwrap_or_default(),
                text: word_text,
                visibility,
                energy,
            });
        }

        Ok(Self {
            state: EchoRecallState {
                words: echo_recall_words,
            },
            user_id: user_id.to_string(),
            ayah_node_ids,
        })
    }

    /// Create an Echo Recall exercise from pre-built state (for deserialization)
    pub fn from_state(user_id: &str, ayah_node_ids: Vec<String>, state: EchoRecallState) -> Self {
        Self {
            state,
            user_id: user_id.to_string(),
            ayah_node_ids,
        }
    }

    /// Get the current state for UI rendering
    pub fn state(&self) -> &EchoRecallState {
        &self.state
    }

    /// Get the user ID for this session
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Get the ayah node IDs this session covers
    pub fn ayah_node_ids(&self) -> &[String] {
        &self.ayah_node_ids
    }

    /// Submit a word recall and update state
    ///
    /// Calculates energy change based on recall time, updates the word's
    /// energy and visibility, and recalculates neighbor visibility.
    ///
    /// # Arguments
    /// * `word_node_id` - The node ID of the recalled word (e.g., "WORD:1:1:1")
    /// * `recall_time_ms` - Time taken to recall in milliseconds
    ///
    /// # Returns
    /// The new energy value for the word, or error if word not found
    pub fn submit_recall(&mut self, word_node_id: &str, recall_time_ms: u32) -> Result<f64> {
        // Find the word index
        let word_index = self
            .state
            .words
            .iter()
            .position(|w| w.node_id == word_node_id)
            .ok_or_else(|| anyhow!("Word not found in session: {}", word_node_id))?;

        // Calculate energy change based on recall time
        let energy_delta = recall_model::calculate_energy_change(recall_time_ms);

        // Update the target word's energy
        let new_energy = (self.state.words[word_index].energy + energy_delta).clamp(0.0, 1.0);
        self.state.words[word_index].energy = new_energy;

        // Recalculate visibility for the target word and its neighbors
        self.recalculate_visibility_around(word_index);

        Ok(new_energy)
    }

    /// Recalculate visibility for a word and its immediate neighbors
    fn recalculate_visibility_around(&mut self, center_index: usize) {
        // Collect indices to update (avoid duplicates for edge cases)
        let mut indices_to_update = HashSet::new();
        if center_index > 0 {
            indices_to_update.insert(center_index - 1);
        }
        indices_to_update.insert(center_index);
        if center_index + 1 < self.state.words.len() {
            indices_to_update.insert(center_index + 1);
        }

        for &i in &indices_to_update {
            let prev_energy = if i > 0 {
                Some(self.state.words[i - 1].energy)
            } else {
                None
            };

            let next_energy = if i < self.state.words.len() - 1 {
                Some(self.state.words[i + 1].energy)
            } else {
                None
            };

            self.state.words[i].visibility = energy_service::map_energy_to_visibility(
                self.state.words[i].energy,
                &self.state.words[i].text,
                prev_energy,
                next_energy,
            );
        }
    }

    /// Get session statistics for UI display
    pub fn get_stats(&self) -> EchoRecallStats {
        self.state.get_stats()
    }

    /// Get final energy updates for persistence
    ///
    /// Returns a list of (node_id, energy) pairs to save to the database.
    /// The node_id is the numeric ID, not the ukey string.
    pub fn finalize(&self) -> Vec<(i64, f64)> {
        self.state
            .words
            .iter()
            .filter_map(|w| nid::from_ukey(&w.node_id).map(|id| (id, w.energy)))
            .collect()
    }

    /// Check if all words are fully mastered (hidden)
    pub fn is_complete(&self) -> bool {
        self.state
            .words
            .iter()
            .all(|w| matches!(w.visibility, WordVisibility::Hidden))
    }

    /// Get count of words at each visibility level
    pub fn visibility_counts(&self) -> (usize, usize, usize) {
        let mut visible = 0;
        let mut obscured = 0;
        let mut hidden = 0;

        for word in &self.state.words {
            match word.visibility {
                WordVisibility::Visible => visible += 1,
                WordVisibility::Obscured { .. } => obscured += 1,
                WordVisibility::Hidden => hidden += 1,
            }
        }

        (visible, obscured, hidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{MockContentRepository, MockUserRepository};
    use crate::{MemoryState, Node, NodeType};
    use chrono::Utc;
    use mockall::predicate::*;

    /// Helper to create test nodes for words in Bismillah (1:1)
    fn create_test_nodes() -> Vec<Node> {
        // Use properly encoded word IDs so nid::to_ukey() works correctly
        vec![
            Node {
                id: nid::encode_word(101),
                ukey: "WORD:101".to_string(),
                node_type: NodeType::Word,
            },
            Node {
                id: nid::encode_word(102),
                ukey: "WORD:102".to_string(),
                node_type: NodeType::Word,
            },
            Node {
                id: nid::encode_word(103),
                ukey: "WORD:103".to_string(),
                node_type: NodeType::Word,
            },
            Node {
                id: nid::encode_word(104),
                ukey: "WORD:104".to_string(),
                node_type: NodeType::Word,
            },
        ]
    }

    /// Word texts for the test nodes
    fn get_word_texts() -> HashMap<i64, String> {
        let mut texts = HashMap::new();
        texts.insert(nid::encode_word(101), "بِسْمِ".to_string());
        texts.insert(nid::encode_word(102), "ٱللَّهِ".to_string());
        texts.insert(nid::encode_word(103), "ٱلرَّحْمَٰنِ".to_string());
        texts.insert(nid::encode_word(104), "ٱلرَّحِيمِ".to_string());
        texts
    }

    /// Helper to create a mock content repository with test data
    fn create_mock_content_repo() -> MockContentRepository {
        let mut mock = MockContentRepository::new();

        // Return test nodes for get_words_in_ayahs
        let nodes = create_test_nodes();
        mock.expect_get_words_in_ayahs()
            .returning(move |_| Ok(nodes.clone()));

        // Setup get_quran_text for each word
        let texts = get_word_texts();
        for (id, text) in texts {
            mock.expect_get_quran_text()
                .with(eq(id))
                .returning(move |_| Ok(Some(text.clone())));
        }

        mock
    }

    /// Helper to create a mock user repository
    fn create_mock_user_repo(energies: HashMap<i64, f64>) -> MockUserRepository {
        let mut mock = MockUserRepository::new();

        let energies_for_closure = energies;
        mock.expect_get_memory_state()
            .returning(move |user_id, node_id| {
                if let Some(&energy) = energies_for_closure.get(&node_id) {
                    Ok(Some(MemoryState {
                        user_id: user_id.to_string(),
                        node_id,
                        energy,
                        stability: 1.0,
                        difficulty: 0.3,
                        last_reviewed: Utc::now(),
                        due_at: Utc::now(),
                        review_count: 1,
                    }))
                } else {
                    Ok(None)
                }
            });

        mock
    }

    #[tokio::test]
    async fn test_exercise_creation() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new(); // All words start at 0 energy
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        assert_eq!(exercise.state().words.len(), 4);
        assert_eq!(exercise.user_id(), "test_user");
        assert_eq!(exercise.ayah_node_ids(), &["VERSE:1:1".to_string()]);
    }

    #[tokio::test]
    async fn test_new_words_are_visible() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new(); // All 0 energy
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        // All words should be visible at 0 energy
        for word in &exercise.state().words {
            assert_eq!(word.visibility, WordVisibility::Visible);
            assert_eq!(word.energy, 0.0);
        }
    }

    #[tokio::test]
    async fn test_high_energy_words_are_hidden() {
        let content_repo = create_mock_content_repo();

        // All words at high energy
        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.9);
        energies.insert(nid::encode_word(102), 0.9);
        energies.insert(nid::encode_word(103), 0.9);
        energies.insert(nid::encode_word(104), 0.9);
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        // All words should be hidden at 0.9 energy (>= 0.85 threshold)
        for word in &exercise.state().words {
            assert_eq!(word.visibility, WordVisibility::Hidden);
        }
    }

    #[tokio::test]
    async fn test_submit_recall_increases_energy() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new();
        let user_repo = create_mock_user_repo(energies);

        let mut exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        let initial_energy = exercise.state().words[0].energy;
        assert_eq!(initial_energy, 0.0);

        // Fast recall (500ms) should increase energy
        let new_energy = exercise.submit_recall("WORD:101", 500).unwrap();

        assert!(new_energy > initial_energy);
        assert_eq!(exercise.state().words[0].energy, new_energy);
    }

    #[tokio::test]
    async fn test_slow_recall_decreases_energy() {
        let content_repo = create_mock_content_repo();

        // Start with medium energy
        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.5);
        let user_repo = create_mock_user_repo(energies);

        let mut exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        let initial_energy = exercise.state().words[0].energy;

        // Very slow recall (5000ms) should decrease energy
        let new_energy = exercise.submit_recall("WORD:101", 5000).unwrap();

        assert!(new_energy < initial_energy);
    }

    #[tokio::test]
    async fn test_submit_recall_unknown_word_errors() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new();
        let user_repo = create_mock_user_repo(energies);

        let mut exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        let result = exercise.submit_recall("WORD:99:99:99", 500);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_finalize_returns_all_energies() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new();
        let user_repo = create_mock_user_repo(energies);

        let mut exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        // Do some recalls
        exercise.submit_recall("WORD:101", 500).unwrap();
        exercise.submit_recall("WORD:102", 600).unwrap();

        let updates = exercise.finalize();
        assert_eq!(updates.len(), 4);

        // First two should have increased energy
        let (id1, energy1) = updates
            .iter()
            .find(|(id, _)| *id == nid::encode_word(101))
            .unwrap();
        assert_eq!(*id1, nid::encode_word(101));
        assert!(*energy1 > 0.0);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new();
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        let stats = exercise.get_stats();
        assert_eq!(stats.total_words, 4);
        assert_eq!(stats.visible_count, 4);
        assert_eq!(stats.obscured_count, 0);
        assert_eq!(stats.hidden_count, 0);
        assert_eq!(stats.average_energy, 0.0);
        assert_eq!(stats.mastery_percentage, 0.0);
    }

    #[tokio::test]
    async fn test_visibility_counts() {
        let content_repo = create_mock_content_repo();

        // Mixed energies
        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.0); // Visible
        energies.insert(nid::encode_word(102), 0.5); // Obscured
        energies.insert(nid::encode_word(103), 0.5); // Obscured
        energies.insert(nid::encode_word(104), 0.9); // Hidden
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        let (visible, obscured, hidden) = exercise.visibility_counts();
        assert_eq!(visible, 1);
        assert_eq!(obscured, 2);
        assert_eq!(hidden, 1);
    }

    #[tokio::test]
    async fn test_is_complete_all_hidden() {
        let content_repo = create_mock_content_repo();

        // All high energy
        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.9);
        energies.insert(nid::encode_word(102), 0.9);
        energies.insert(nid::encode_word(103), 0.9);
        energies.insert(nid::encode_word(104), 0.9);
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        assert!(exercise.is_complete());
    }

    #[tokio::test]
    async fn test_is_complete_not_all_hidden() {
        let content_repo = create_mock_content_repo();
        let energies = HashMap::new();
        let user_repo = create_mock_user_repo(energies);

        let exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        assert!(!exercise.is_complete());
    }

    #[tokio::test]
    async fn test_from_state_preserves_state() {
        let state = EchoRecallState {
            words: vec![EchoRecallWord {
                node_id: "WORD:1:1:1".to_string(),
                text: "بِسْمِ".to_string(),
                visibility: WordVisibility::Visible,
                energy: 0.3,
            }],
        };

        let exercise = EchoRecallExercise::from_state(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            state.clone(),
        );

        assert_eq!(exercise.state().words.len(), 1);
        assert_eq!(exercise.state().words[0].energy, 0.3);
    }

    #[tokio::test]
    async fn test_energy_clamped_to_bounds() {
        let content_repo = create_mock_content_repo();

        // Start at very high energy
        let mut energies = HashMap::new();
        energies.insert(101, 0.99);
        let user_repo = create_mock_user_repo(energies);

        let mut exercise = EchoRecallExercise::new(
            "test_user",
            vec!["VERSE:1:1".to_string()],
            &content_repo,
            &user_repo,
        )
        .await
        .unwrap();

        // Fast recall should try to increase past 1.0 but get clamped
        let new_energy = exercise.submit_recall("WORD:101", 100).unwrap();
        assert!(new_energy <= 1.0);
    }

    #[tokio::test]
    async fn test_empty_ayahs_errors() {
        let mock = MockContentRepository::new();
        let user_mock = MockUserRepository::new();

        let result = EchoRecallExercise::new(
            "test_user",
            vec![], // No ayahs
            &mock,
            &user_mock,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No valid"));
    }
}
