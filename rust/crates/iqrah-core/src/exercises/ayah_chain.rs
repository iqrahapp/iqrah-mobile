// exercises/ayah_chain.rs
// Exercise 10: Ayah Chain - Continuous verse typing until mistake or completion

use super::memorization::MemorizationExercise;
use super::types::Exercise;
use crate::domain::node_id::{self, PREFIX_CHAPTER};
use crate::{ContentRepository, Verse};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Exercise 10: Ayah Chain
// ============================================================================

/// Stateful exercise for continuous verse typing
/// User types verses in sequence until making a mistake or completing the chain
/// Tracks current position and progress through a chapter or range
#[derive(Debug)]
pub struct AyahChainExercise {
    node_id: i64,
    verses: Vec<Verse>,
    current_index: usize,
    completed_count: usize,
    is_complete: bool,
    mistake_made: bool,
    started_at: DateTime<Utc>,
    last_mistake: Option<MistakeDetails>,
}

/// Details about the most recent mistake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeDetails {
    pub verse_key: String,
    pub user_input: String,
    pub expected: String,
    pub occurred_at: DateTime<Utc>,
}

impl AyahChainExercise {
    /// Create a new Ayah Chain exercise
    ///
    /// Queries the database for all verses in the specified chapter
    /// User will type verses in sequence starting from the first verse
    pub async fn new(chapter_node_id: i64, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Get the node to access its ukey
        let node = content_repo
            .get_node(chapter_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", chapter_node_id))?;

        // Parse chapter number from node_id (format: PREFIX_CHAPTER + "n", e.g., "CHAPTER:1")
        let chapter_num: i32 = node
            .ukey
            .strip_prefix(PREFIX_CHAPTER)
            .ok_or_else(|| anyhow::anyhow!("Invalid chapter node ID: {}", node.ukey))?
            .parse()?;

        // Get all verses for the chapter
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        if verses.is_empty() {
            return Err(anyhow::anyhow!(
                "No verses found for chapter {}",
                chapter_num
            ));
        }

        Ok(Self {
            node_id: chapter_node_id,
            verses,
            current_index: 0,
            completed_count: 0,
            is_complete: false,
            mistake_made: false,
            started_at: Utc::now(),
            last_mistake: None,
        })
    }

    /// Create an Ayah Chain for a specific verse range within a chapter
    pub async fn new_range(
        chapter_num: i32,
        start_verse: i32,
        end_verse: i32,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Get all verses for the chapter
        let all_verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // Filter to requested range
        let verses: Vec<Verse> = all_verses
            .into_iter()
            .filter(|v| v.verse_number >= start_verse && v.verse_number <= end_verse)
            .collect();

        if verses.is_empty() {
            return Err(anyhow::anyhow!(
                "No verses found for chapter {} range {}:{}",
                chapter_num,
                start_verse,
                end_verse
            ));
        }

        // Get chapter node ID
        let chapter_ukey = node_id::chapter(chapter_num as u8);
        let node = content_repo
            .get_node_by_ukey(&chapter_ukey)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Chapter node not found: {}", chapter_ukey))?;
        let node_id = node.id;

        Ok(Self {
            node_id,
            verses,
            current_index: 0,
            completed_count: 0,
            is_complete: false,
            mistake_made: false,
            started_at: Utc::now(),
            last_mistake: None,
        })
    }

    /// Get the current verse being tested
    pub fn current_verse(&self) -> Option<&Verse> {
        if self.is_complete || self.mistake_made {
            return None;
        }
        self.verses.get(self.current_index)
    }

    /// Get the current verse reference (e.g., "1:1")
    pub fn current_verse_ref(&self) -> Option<String> {
        self.current_verse().map(|v| v.key.clone())
    }

    /// Submit an answer for the current verse and advance
    ///
    /// Returns Ok(true) if answer was correct and chain continues
    /// Returns Ok(false) if answer was wrong (chain ends)
    /// Returns Err if exercise is already complete or invalid state
    pub fn submit_answer(&mut self, user_input: &str) -> Result<bool> {
        if self.is_complete {
            return Err(anyhow::anyhow!("Exercise already complete"));
        }

        if self.mistake_made {
            return Err(anyhow::anyhow!("Chain already broken by mistake"));
        }

        let current_verse = self
            .current_verse()
            .ok_or_else(|| anyhow::anyhow!("No current verse"))?;

        // Clone necessary data before mutation
        let verse_key = current_verse.key.clone();
        let verse_text = current_verse.text_uthmani.clone();

        // Check answer using Arabic normalization
        let normalized_input = MemorizationExercise::normalize_arabic(user_input);
        let normalized_correct = MemorizationExercise::normalize_arabic(&verse_text);

        if normalized_input == normalized_correct {
            // Correct! Advance to next verse
            self.completed_count += 1;
            self.current_index += 1;

            // Check if we've completed all verses
            if self.current_index >= self.verses.len() {
                self.is_complete = true;
            }

            Ok(true)
        } else {
            // Mistake! Chain broken
            self.mistake_made = true;

            // Capture mistake details for feedback
            self.last_mistake = Some(MistakeDetails {
                verse_key,
                user_input: user_input.to_string(),
                expected: verse_text,
                occurred_at: Utc::now(),
            });

            Ok(false)
        }
    }

    /// Get completion stats
    pub fn get_stats(&self) -> AyahChainStats {
        let progress_percentage = if self.verses.is_empty() {
            0.0
        } else {
            (self.completed_count as f64 / self.verses.len() as f64) * 100.0
        };

        let elapsed_seconds = (Utc::now() - self.started_at).num_seconds() as u64;

        let current_verse_key = self.current_verse().map(|v| v.key.clone());

        AyahChainStats {
            total_verses: self.verses.len(),
            completed_count: self.completed_count,
            is_complete: self.is_complete,
            mistake_made: self.mistake_made,
            progress_percentage,
            elapsed_seconds,
            current_verse_key,
            last_mistake: self.last_mistake.clone(),
        }
    }

    /// Get the most recent mistake details (if any)
    pub fn get_last_mistake(&self) -> Option<&MistakeDetails> {
        self.last_mistake.as_ref()
    }

    /// Reset the chain to start over
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.completed_count = 0;
        self.is_complete = false;
        self.mistake_made = false;
        self.started_at = Utc::now();
        self.last_mistake = None;
    }
}

/// Statistics for Ayah Chain performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AyahChainStats {
    /// Total number of verses in the chain
    pub total_verses: usize,
    /// Number of verses successfully completed
    pub completed_count: usize,
    /// Whether the entire chain has been completed
    pub is_complete: bool,
    /// Whether a mistake was made (chain broken)
    pub mistake_made: bool,
    /// Progress as a percentage (0.0 to 100.0)
    pub progress_percentage: f64,
    /// Time elapsed since start in seconds
    pub elapsed_seconds: u64,
    /// Current verse key being tested (if any)
    pub current_verse_key: Option<String>,
    /// Details about the last mistake (if any)
    pub last_mistake: Option<MistakeDetails>,
}

impl Exercise for AyahChainExercise {
    fn generate_question(&self) -> String {
        if let Some(verse) = self.current_verse() {
            format!(
                "Ayah Chain: {}/{}\n\nType verse {}:",
                self.completed_count + 1,
                self.verses.len(),
                verse.key
            )
        } else if self.is_complete {
            format!(
                "🎉 Chain Complete! You successfully typed all {} verses!",
                self.verses.len()
            )
        } else if self.mistake_made {
            format!(
                "Chain broken. You completed {}/{} verses.",
                self.completed_count,
                self.verses.len()
            )
        } else {
            "No current verse".to_string()
        }
    }

    fn check_answer(&self, answer: &str) -> bool {
        if let Some(verse) = self.current_verse() {
            let normalized_input = MemorizationExercise::normalize_arabic(answer);
            let normalized_correct = MemorizationExercise::normalize_arabic(&verse.text_uthmani);
            normalized_input == normalized_correct
        } else {
            false
        }
    }

    fn get_hint(&self) -> Option<String> {
        self.current_verse().map(|verse| {
            // Show first word and provide context
            let words: Vec<&str> = verse.text_uthmani.split_whitespace().collect();
            let first_word = words.first().copied().unwrap_or("");
            let word_count = words.len();

            format!(
                "Hint: First word is '{}' ({} words total)",
                first_word, word_count
            )
        })
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "ayah_chain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockContentRepository;
    use crate::{Node, NodeType};
    use mockall::predicate::*;

    /// Helper function to create a mock with Al-Fatihah data
    fn create_mock_with_fatihah() -> MockContentRepository {
        let mut mock = MockContentRepository::new();

        // Setup get_node for chapter lookup
        mock.expect_get_node().with(eq(1_i64)).returning(|_| {
            Ok(Some(Node {
                id: 1,
                ukey: "CHAPTER:1".to_string(),
                node_type: NodeType::Chapter,
            }))
        });

        // Setup get_node_by_ukey for chapter lookup
        mock.expect_get_node_by_ukey()
            .with(eq("CHAPTER:1"))
            .returning(|_| {
                Ok(Some(Node {
                    id: 1,
                    ukey: "CHAPTER:1".to_string(),
                    node_type: NodeType::Chapter,
                }))
            });

        // Setup get_verses_for_chapter with Al-Fatihah verses
        mock.expect_get_verses_for_chapter()
            .with(eq(1_i32))
            .returning(|_| {
                Ok(vec![
                    Verse {
                        key: "1:1".to_string(),
                        chapter_number: 1,
                        verse_number: 1,
                        text_uthmani: "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                    Verse {
                        key: "1:2".to_string(),
                        chapter_number: 1,
                        verse_number: 2,
                        text_uthmani: "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                    Verse {
                        key: "1:3".to_string(),
                        chapter_number: 1,
                        verse_number: 3,
                        text_uthmani: "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                    Verse {
                        key: "1:4".to_string(),
                        chapter_number: 1,
                        verse_number: 4,
                        text_uthmani: "مَٰلِكِ يَوْمِ ٱلدِّينِ".to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                    Verse {
                        key: "1:5".to_string(),
                        chapter_number: 1,
                        verse_number: 5,
                        text_uthmani: "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ".to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                    Verse {
                        key: "1:6".to_string(),
                        chapter_number: 1,
                        verse_number: 6,
                        text_uthmani: "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ".to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                    Verse {
                        key: "1:7".to_string(),
                        chapter_number: 1,
                        verse_number: 7,
                        text_uthmani: "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ"
                            .to_string(),
                        text_simple: None,
                        juz: 1,
                        page: 1,
                    },
                ])
            });

        mock
    }

    #[tokio::test]
    async fn test_ayah_chain_creation() {
        let mock = create_mock_with_fatihah();
        let exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        let stats = exercise.get_stats();
        assert_eq!(stats.total_verses, 7); // Al-Fatihah has 7 verses
        assert_eq!(stats.completed_count, 0);
        assert!(!stats.is_complete);
        assert!(!stats.mistake_made);
    }

    #[tokio::test]
    async fn test_ayah_chain_first_verse() {
        let mock = create_mock_with_fatihah();
        let exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        let current = exercise.current_verse().unwrap();
        assert_eq!(current.key, "1:1");
        assert_eq!(current.text_uthmani, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
    }

    #[tokio::test]
    async fn test_ayah_chain_correct_answer_advances() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Submit correct answer for verse 1:1
        let result = exercise.submit_answer("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ").unwrap();
        assert!(result); // Correct answer

        // Should advance to verse 1:2
        let current = exercise.current_verse().unwrap();
        assert_eq!(current.key, "1:2");

        let stats = exercise.get_stats();
        assert_eq!(stats.completed_count, 1);
        assert!(!stats.is_complete);
    }

    #[tokio::test]
    async fn test_ayah_chain_incorrect_answer_breaks_chain() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Submit incorrect answer
        let result = exercise.submit_answer("wrong answer").unwrap();
        assert!(!result); // Incorrect answer

        let stats = exercise.get_stats();
        assert_eq!(stats.completed_count, 0);
        assert!(stats.mistake_made);
        assert!(!stats.is_complete);

        // Chain is broken, no current verse
        assert!(exercise.current_verse().is_none());
    }

    #[tokio::test]
    async fn test_ayah_chain_normalization() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Submit answer without tashkeel - should still be correct
        let result = exercise.submit_answer("بسم الله الرحمان الرحيم").unwrap();
        assert!(result); // Should be correct due to normalization
    }

    #[tokio::test]
    async fn test_ayah_chain_complete_all_verses() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Submit all 7 verses correctly
        let verses = [
            "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
            "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ",
            "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
            "مَٰلِكِ يَوْمِ ٱلدِّينِ",
            "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ",
            "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ",
            "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ",
        ];

        for verse in &verses {
            let result = exercise.submit_answer(verse).unwrap();
            assert!(result);
        }

        let stats = exercise.get_stats();
        assert_eq!(stats.completed_count, 7);
        assert!(stats.is_complete);
        assert!(!stats.mistake_made);

        // No current verse after completion
        assert!(exercise.current_verse().is_none());
    }

    #[tokio::test]
    async fn test_ayah_chain_partial_completion() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Complete first 3 verses
        exercise.submit_answer("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ").unwrap();
        exercise.submit_answer("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ").unwrap();
        exercise.submit_answer("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ").unwrap();

        // Make a mistake on verse 4
        let result = exercise.submit_answer("wrong").unwrap();
        assert!(!result);

        let stats = exercise.get_stats();
        assert_eq!(stats.completed_count, 3);
        assert!(stats.mistake_made);
        assert!(!stats.is_complete);
    }

    #[tokio::test]
    async fn test_ayah_chain_reset() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Complete first verse and make mistake on second
        exercise.submit_answer("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ").unwrap();
        exercise.submit_answer("wrong").unwrap();

        let stats = exercise.get_stats();
        assert_eq!(stats.completed_count, 1);
        assert!(stats.mistake_made);

        // Reset
        exercise.reset();

        let stats = exercise.get_stats();
        assert_eq!(stats.completed_count, 0);
        assert!(!stats.mistake_made);
        assert!(!stats.is_complete);

        // Should be back to verse 1:1
        let current = exercise.current_verse().unwrap();
        assert_eq!(current.key, "1:1");
    }

    #[tokio::test]
    async fn test_ayah_chain_range() {
        let mock = create_mock_with_fatihah();
        let exercise = AyahChainExercise::new_range(1, 1, 3, &mock).await.unwrap();

        let stats = exercise.get_stats();
        assert_eq!(stats.total_verses, 3); // Only verses 1-3

        let current = exercise.current_verse().unwrap();
        assert_eq!(current.key, "1:1");
    }

    #[tokio::test]
    async fn test_ayah_chain_question_format() {
        let mock = create_mock_with_fatihah();
        let exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        let question = exercise.generate_question();
        assert!(question.contains("1/7")); // Progress indicator
        assert!(question.contains("1:1")); // Verse reference
    }

    #[tokio::test]
    async fn test_ayah_chain_hint() {
        let mock = create_mock_with_fatihah();
        let exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        let hint = exercise.get_hint().unwrap();
        // Shows first word + word count
        assert!(hint.contains("بِسْمِ")); // First word
        assert!(hint.contains("4 words total")); // Word count (verse 1:1 has 4 words)
    }

    #[tokio::test]
    async fn test_ayah_chain_check_answer_method() {
        let mock = create_mock_with_fatihah();
        let exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Test Exercise trait check_answer method
        assert!(exercise.check_answer("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
        assert!(!exercise.check_answer("wrong answer"));
    }

    #[tokio::test]
    async fn test_ayah_chain_cannot_submit_after_complete() {
        let mock = create_mock_with_fatihah();
        let mut exercise = AyahChainExercise::new(1, &mock).await.unwrap();

        // Complete all verses
        let verses = vec![
            "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
            "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ",
            "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
            "مَٰلِكِ يَوْمِ ٱلدِّينِ",
            "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ",
            "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ",
            "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ",
        ];

        for verse in &verses {
            exercise.submit_answer(verse).unwrap();
        }

        // Try to submit another answer
        let result = exercise.submit_answer("anything");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already complete"));
    }
}
