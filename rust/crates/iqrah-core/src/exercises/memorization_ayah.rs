// exercises/memorization_ayah.rs
// Verse-level memorization exercise with word-by-word energy tracking
//
// This is a simpler stateful exercise where users tap/long-press words
// to increase their energy levels.

use crate::domain::node_id as nid;
use crate::ports::{ContentRepository, UserRepository};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Action that can be performed on a word
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemorizationAction {
    /// Quick tap - small energy increase
    Tap,
    /// Long press - larger energy increase
    LongPress,
}

impl MemorizationAction {
    /// Get the energy delta for this action
    pub fn energy_delta(&self) -> f64 {
        match self {
            MemorizationAction::Tap => 0.05,
            MemorizationAction::LongPress => 0.10,
        }
    }

    /// Parse from string (for compatibility with server commands)
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Tap" | "tap" => Some(MemorizationAction::Tap),
            "LongPress" | "longpress" | "long_press" => Some(MemorizationAction::LongPress),
            _ => None,
        }
    }
}

/// A word in the memorization exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorizationWord {
    /// Word node ID (e.g., "WORD:101")
    pub node_id: String,
    /// Arabic text of the word
    pub text: String,
    /// Current energy level (0.0 to 1.0)
    pub energy: f64,
}

/// State for the memorization exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorizationAyahState {
    /// Verse being memorized
    pub verse_node_id: String,
    /// Words in the verse with their energies
    pub words: Vec<MemorizationWord>,
}

/// Stateful verse-level memorization exercise
///
/// User taps/long-presses on words to increase their energy levels.
/// Simple and intuitive - good for initial memorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorizationAyahExercise {
    state: MemorizationAyahState,
    user_id: String,
}

impl MemorizationAyahExercise {
    /// Create a new Memorization exercise from a verse node ID
    ///
    /// Fetches all words in the verse and initializes their energy levels.
    pub async fn new(
        user_id: &str,
        verse_node_id: &str,
        content_repo: &dyn ContentRepository,
        user_repo: &dyn UserRepository,
    ) -> Result<Self> {
        let verse_id = nid::from_ukey(verse_node_id).ok_or_else(|| anyhow!("Invalid verse ID"))?;

        // Get edges from verse to find word children
        let edges = content_repo.get_edges_from(verse_id).await?;

        let mut words = Vec::new();
        for edge in edges {
            let target_ukey = nid::to_ukey(edge.target_id).unwrap_or_default();
            if target_ukey.starts_with(nid::PREFIX_WORD) {
                let word_text = content_repo
                    .get_quran_text(edge.target_id)
                    .await?
                    .unwrap_or_default();

                // Get current energy from user state (default to 0.0)
                let memory_state = user_repo
                    .get_memory_state(user_id, edge.target_id)
                    .await
                    .ok()
                    .flatten();
                let energy = memory_state.map(|s| s.energy).unwrap_or(0.0);

                words.push(MemorizationWord {
                    node_id: target_ukey,
                    text: word_text,
                    energy,
                });
            }
        }

        if words.is_empty() {
            return Err(anyhow!("No words found in verse: {}", verse_node_id));
        }

        Ok(Self {
            state: MemorizationAyahState {
                verse_node_id: verse_node_id.to_string(),
                words,
            },
            user_id: user_id.to_string(),
        })
    }

    /// Create from pre-built state (for deserialization)
    pub fn from_state(user_id: &str, state: MemorizationAyahState) -> Self {
        Self {
            state,
            user_id: user_id.to_string(),
        }
    }

    /// Get the current state for UI rendering
    pub fn state(&self) -> &MemorizationAyahState {
        &self.state
    }

    /// Get the user ID
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Get the verse node ID
    pub fn verse_node_id(&self) -> &str {
        &self.state.verse_node_id
    }

    /// Update a word's energy with an action
    ///
    /// Returns the new energy value, or error if word not found.
    pub fn update_word(&mut self, word_node_id: &str, action: MemorizationAction) -> Result<f64> {
        let word = self
            .state
            .words
            .iter_mut()
            .find(|w| w.node_id == word_node_id)
            .ok_or_else(|| anyhow!("Word not found in session: {}", word_node_id))?;

        let new_energy = (word.energy + action.energy_delta()).min(1.0);
        word.energy = new_energy;
        Ok(new_energy)
    }

    /// Update a word with an action string (for server compatibility)
    pub fn update_word_str(&mut self, word_node_id: &str, action: &str) -> Result<f64> {
        let action = MemorizationAction::from_str(action)
            .ok_or_else(|| anyhow!("Unknown action: {}", action))?;
        self.update_word(word_node_id, action)
    }

    /// Get final energy updates for persistence
    pub fn finalize(&self) -> Vec<(i64, f64)> {
        self.state
            .words
            .iter()
            .filter_map(|w| nid::from_ukey(&w.node_id).map(|id| (id, w.energy)))
            .collect()
    }

    /// Check if all words are mastered (energy >= 0.8)
    pub fn is_complete(&self) -> bool {
        self.state.words.iter().all(|w| w.energy >= 0.8)
    }

    /// Get average energy across all words
    pub fn average_energy(&self) -> f64 {
        if self.state.words.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.state.words.iter().map(|w| w.energy).sum();
        sum / self.state.words.len() as f64
    }

    /// Get count of words at different mastery levels
    pub fn mastery_counts(&self) -> (usize, usize, usize) {
        let mut low = 0; // < 0.3
        let mut medium = 0; // 0.3 - 0.7
        let mut high = 0; // >= 0.7

        for word in &self.state.words {
            if word.energy < 0.3 {
                low += 1;
            } else if word.energy < 0.7 {
                medium += 1;
            } else {
                high += 1;
            }
        }

        (low, medium, high)
    }
}

/// Convert state to JSON (for server compatibility)
impl MemorizationAyahState {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "verse_node_id": self.verse_node_id,
            "words": self.words.iter().map(|w| {
                serde_json::json!({
                    "node_id": w.node_id,
                    "text": w.text,
                    "energy": w.energy,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DistributionType, Edge, EdgeType};
    use crate::testing::{MockContentRepository, MockUserRepository};
    use crate::MemoryState;
    use chrono::Utc;
    use mockall::predicate::*;
    use std::collections::HashMap;

    fn create_mock_content_repo() -> MockContentRepository {
        let mut mock = MockContentRepository::new();

        // Return edges from verse to words
        mock.expect_get_edges_from().returning(|_| {
            Ok(vec![
                Edge {
                    source_id: nid::encode_verse(1, 1),
                    target_id: nid::encode_word(101),
                    edge_type: EdgeType::Dependency,
                    distribution_type: DistributionType::Const,
                    param1: 1.0,
                    param2: 0.0,
                },
                Edge {
                    source_id: nid::encode_verse(1, 1),
                    target_id: nid::encode_word(102),
                    edge_type: EdgeType::Dependency,
                    distribution_type: DistributionType::Const,
                    param1: 1.0,
                    param2: 0.0,
                },
                Edge {
                    source_id: nid::encode_verse(1, 1),
                    target_id: nid::encode_word(103),
                    edge_type: EdgeType::Dependency,
                    distribution_type: DistributionType::Const,
                    param1: 1.0,
                    param2: 0.0,
                },
            ])
        });

        // Setup get_quran_text
        mock.expect_get_quran_text()
            .with(eq(nid::encode_word(101)))
            .returning(|_| Ok(Some("بِسْمِ".to_string())));
        mock.expect_get_quran_text()
            .with(eq(nid::encode_word(102)))
            .returning(|_| Ok(Some("ٱللَّهِ".to_string())));
        mock.expect_get_quran_text()
            .with(eq(nid::encode_word(103)))
            .returning(|_| Ok(Some("ٱلرَّحْمَٰنِ".to_string())));

        mock
    }

    fn create_mock_user_repo(energies: HashMap<i64, f64>) -> MockUserRepository {
        let mut mock = MockUserRepository::new();

        mock.expect_get_memory_state()
            .returning(move |user_id, node_id| {
                if let Some(&energy) = energies.get(&node_id) {
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
        let user_repo = create_mock_user_repo(HashMap::new());

        let exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        assert_eq!(exercise.state().words.len(), 3);
        assert_eq!(exercise.verse_node_id(), "VERSE:1:1");
    }

    #[tokio::test]
    async fn test_tap_increases_energy() {
        let content_repo = create_mock_content_repo();
        let user_repo = create_mock_user_repo(HashMap::new());

        let mut exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        // Initial energy is 0
        assert_eq!(exercise.state().words[0].energy, 0.0);

        // Tap adds 0.05
        let new_energy = exercise
            .update_word("WORD:101", MemorizationAction::Tap)
            .unwrap();
        assert!((new_energy - 0.05).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_long_press_increases_more() {
        let content_repo = create_mock_content_repo();
        let user_repo = create_mock_user_repo(HashMap::new());

        let mut exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        // LongPress adds 0.10
        let new_energy = exercise
            .update_word("WORD:101", MemorizationAction::LongPress)
            .unwrap();
        assert!((new_energy - 0.10).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_energy_capped_at_one() {
        let content_repo = create_mock_content_repo();

        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.98);
        let user_repo = create_mock_user_repo(energies);

        let mut exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        // LongPress at 0.98 should cap at 1.0
        let new_energy = exercise
            .update_word("WORD:101", MemorizationAction::LongPress)
            .unwrap();
        assert!((new_energy - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_update_word_str() {
        let content_repo = create_mock_content_repo();
        let user_repo = create_mock_user_repo(HashMap::new());

        let mut exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        // Test string-based API
        exercise.update_word_str("WORD:101", "Tap").unwrap();
        assert!((exercise.state().words[0].energy - 0.05).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_unknown_word_errors() {
        let content_repo = create_mock_content_repo();
        let user_repo = create_mock_user_repo(HashMap::new());

        let mut exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        let result = exercise.update_word("WORD:999", MemorizationAction::Tap);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_complete() {
        let content_repo = create_mock_content_repo();

        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.9);
        energies.insert(nid::encode_word(102), 0.85);
        energies.insert(nid::encode_word(103), 0.8);
        let user_repo = create_mock_user_repo(energies);

        let exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        assert!(exercise.is_complete());
    }

    #[tokio::test]
    async fn test_average_energy() {
        let content_repo = create_mock_content_repo();

        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.3);
        energies.insert(nid::encode_word(102), 0.6);
        energies.insert(nid::encode_word(103), 0.9);
        let user_repo = create_mock_user_repo(energies);

        let exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        let avg = exercise.average_energy();
        assert!((avg - 0.6).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mastery_counts() {
        let content_repo = create_mock_content_repo();

        let mut energies = HashMap::new();
        energies.insert(nid::encode_word(101), 0.2); // Low
        energies.insert(nid::encode_word(102), 0.5); // Medium
        energies.insert(nid::encode_word(103), 0.9); // High
        let user_repo = create_mock_user_repo(energies);

        let exercise =
            MemorizationAyahExercise::new("test_user", "VERSE:1:1", &content_repo, &user_repo)
                .await
                .unwrap();

        let (low, medium, high) = exercise.mastery_counts();
        assert_eq!(low, 1);
        assert_eq!(medium, 1);
        assert_eq!(high, 1);
    }
}
