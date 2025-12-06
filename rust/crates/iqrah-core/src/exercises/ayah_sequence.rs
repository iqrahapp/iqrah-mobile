// exercises/ayah_sequence.rs
// Exercise 4: Ayah Sequence (MCQ) - "Which verse comes next?"

use super::types::Exercise;
use crate::domain::node_id::{self, PREFIX_VERSE};
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

// ============================================================================
// Exercise 4: Ayah Sequence (MCQ)
// ============================================================================

/// Exercise for testing knowledge of verse sequence within a chapter
/// Tests the user's memorization of verse order in a Surah
#[derive(Debug)]
pub struct AyahSequenceExercise {
    node_id: i64,
    current_verse_text: String,
    #[allow(dead_code)] // Used in tests and for potential future features
    current_verse_key: String,
    correct_next_verse_text: String,
    correct_next_verse_key: String,
    options: Vec<(String, String)>, // (verse_key, verse_text) pairs
}

impl AyahSequenceExercise {
    /// Create a new Ayah Sequence exercise
    ///
    /// Queries the database for:
    /// - Current verse text
    /// - Next verse in sequence (correct answer)
    /// - Other verses from same chapter (distractors)
    pub async fn new(verse_node_id: i64, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Get the node to access its ukey
        let node = content_repo
            .get_node(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", verse_node_id))?;

        // Parse knowledge node
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node.ukey) {
            kn.base_node_id
        } else {
            node.ukey.clone()
        };

        // Extract verse key from node ID
        // Format: "VERSE:chapter:verse"
        let current_verse_key = base_node_id
            .strip_prefix(PREFIX_VERSE)
            .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", base_node_id))?
            .to_string();

        // Get current verse text
        let current_verse_text = content_repo
            .get_quran_text(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", verse_node_id))?;

        // Parse verse key to get chapter and verse number
        let parts: Vec<&str> = current_verse_key.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid verse key format: {}",
                current_verse_key
            ));
        }

        let chapter_num: i32 = parts[0].parse()?;
        let verse_num: i32 = parts[1].parse()?;

        // Get all verses from the chapter
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // Find the next verse
        let next_verse_num = verse_num + 1;
        let next_verse = verses
            .iter()
            .find(|v| v.verse_number == next_verse_num)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No next verse found for {}:{} (end of chapter)",
                    chapter_num,
                    verse_num
                )
            })?;

        let correct_next_verse_key = next_verse.key.clone();
        // We need the ID of the next verse to get its text using get_quran_text(i64)
        // But we only have the verse key.
        // We can construct the ukey "VERSE:key" and look it up.
        let correct_next_verse_ukey = node_id::verse_from_key(&correct_next_verse_key);
        let next_verse_node = content_repo
            .get_node_by_ukey(&correct_next_verse_ukey)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Next verse node not found: {}", correct_next_verse_ukey)
            })?;

        let correct_next_verse_text = content_repo
            .get_quran_text(next_verse_node.id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Next verse text not found: {}", correct_next_verse_ukey)
            })?;

        // Generate distractors from same chapter (but not current or next verse)
        let distractors = Self::generate_distractors(
            &current_verse_key,
            &correct_next_verse_key,
            &verses,
            content_repo,
        )
        .await?;

        // Combine correct answer with distractors and shuffle
        let mut options = vec![(
            correct_next_verse_key.clone(),
            correct_next_verse_text.clone(),
        )];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        Ok(Self {
            node_id: verse_node_id,
            current_verse_text,
            current_verse_key,
            correct_next_verse_text,
            correct_next_verse_key,
            options,
        })
    }

    /// Generate distractor verses from the same chapter
    async fn generate_distractors(
        current_verse_key: &str,
        correct_next_verse_key: &str,
        all_verses: &[crate::Verse],
        content_repo: &dyn ContentRepository,
    ) -> Result<Vec<(String, String)>> {
        let mut distractors = Vec::new();

        // Filter out current and next verse, then shuffle
        let mut candidate_verses: Vec<_> = all_verses
            .iter()
            .filter(|v| v.key != current_verse_key && v.key != correct_next_verse_key)
            .collect();

        candidate_verses.shuffle(&mut rand::thread_rng());

        // Take first 3 candidates
        for verse in candidate_verses.iter().take(3) {
            let verse_ukey = node_id::verse_from_key(&verse.key);
            if let Some(node) = content_repo.get_node_by_ukey(&verse_ukey).await? {
                if let Some(text) = content_repo.get_quran_text(node.id).await? {
                    distractors.push((verse.key.clone(), text));
                    if distractors.len() >= 3 {
                        break;
                    }
                }
            }
        }

        // Ensure we have 3 distractors
        if distractors.len() < 3 {
            return Err(anyhow::anyhow!(
                "Not enough verses in chapter to generate 3 distractors"
            ));
        }

        Ok(distractors)
    }

    /// Get the MCQ options (verse key, text pairs)
    pub fn get_options(&self) -> &[(String, String)] {
        &self.options
    }

    /// Get the correct answer verse key
    pub fn get_correct_verse_key(&self) -> &str {
        &self.correct_next_verse_key
    }
}

impl Exercise for AyahSequenceExercise {
    fn generate_question(&self) -> String {
        format!(
            "After this verse:\n\n{}\n\nWhich verse comes next?",
            self.current_verse_text
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Normalize for comparison - accept either verse key or verse text
        let normalized_answer = answer.trim();

        // Check if answer matches verse key
        if normalized_answer == self.correct_next_verse_key {
            return true;
        }

        // Check if answer matches verse text
        if normalized_answer == self.correct_next_verse_text {
            return true;
        }

        false
    }

    fn get_hint(&self) -> Option<String> {
        // Show the verse number of the next verse
        let parts: Vec<&str> = self.correct_next_verse_key.split(':').collect();
        if let Some(verse_num) = parts.get(1) {
            return Some(format!("It's verse {}", verse_num));
        }
        None
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "ayah_sequence"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockContentRepository;
    use crate::{Node, NodeType, Verse};
    use mockall::predicate::*;

    /// Helper function to create mock with Al-Fatihah data for ayah sequence tests
    fn create_mock_for_ayah_sequence() -> MockContentRepository {
        let mut mock = MockContentRepository::new();

        // Setup get_node for verse lookups (node IDs 11-17 for verses 1:1-1:7)
        mock.expect_get_node().returning(|node_id| {
            let verse_num = (node_id - 10) as i32;
            if verse_num >= 1 && verse_num <= 7 {
                Ok(Some(Node {
                    id: node_id,
                    ukey: format!("VERSE:1:{}", verse_num),
                    node_type: NodeType::Verse,
                }))
            } else {
                Ok(None)
            }
        });

        // Setup get_node_by_ukey
        mock.expect_get_node_by_ukey().returning(|ukey| {
            if ukey.starts_with("VERSE:1:") {
                let verse_num: i64 = ukey.strip_prefix("VERSE:1:").unwrap().parse().unwrap();
                Ok(Some(Node {
                    id: 10 + verse_num,
                    ukey: ukey.to_string(),
                    node_type: NodeType::Verse,
                }))
            } else {
                Ok(None)
            }
        });

        // Setup get_quran_text
        mock.expect_get_quran_text().returning(|node_id| {
            let texts = [
                (11, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
                (12, "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"),
                (13, "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
                (14, "مَٰلِكِ يَوْمِ ٱلدِّينِ"),
                (15, "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ"),
                (16, "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ"),
                (17, "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ"),
            ];
            for (id, text) in texts {
                if node_id == id {
                    return Ok(Some(text.to_string()));
                }
            }
            Ok(None)
        });

        // Setup get_verses_for_chapter
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
    async fn test_ayah_sequence_basic() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        assert_eq!(exercise.get_node_id(), 11);
        assert_eq!(exercise.get_type_name(), "ayah_sequence");

        let question = exercise.generate_question();
        assert!(question.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
        assert!(question.contains("Which verse comes next"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_finds_correct_next_verse() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        assert_eq!(exercise.get_correct_verse_key(), "1:2");
        assert!(exercise.correct_next_verse_text.contains("ٱلْحَمْدُ لِلَّهِ"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_has_four_options() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        assert_eq!(exercise.get_options().len(), 4);
    }

    #[tokio::test]
    async fn test_ayah_sequence_options_contain_correct() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        let correct_key = exercise.get_correct_verse_key();
        let options = exercise.get_options();
        let correct_in_options = options.iter().any(|(key, _)| key == correct_key);
        assert!(correct_in_options, "Correct answer must be in options");
    }

    #[tokio::test]
    async fn test_ayah_sequence_check_answer_by_key() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        assert!(exercise.check_answer("1:2"));
        assert!(!exercise.check_answer("1:3"));
        assert!(!exercise.check_answer("1:1"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_check_answer_by_text() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        assert!(exercise.check_answer("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"));
        assert!(!exercise.check_answer("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_hint() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        let hint = exercise.get_hint();
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("2"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_middle_verse() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(13, &mock).await.unwrap();

        assert_eq!(exercise.get_correct_verse_key(), "1:4");
        assert!(exercise.check_answer("1:4"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_last_verse_fails() {
        let mock = create_mock_for_ayah_sequence();
        let result = AyahSequenceExercise::new(17, &mock).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No next verse found"));
    }

    #[tokio::test]
    async fn test_ayah_sequence_distractors_are_different() {
        let mock = create_mock_for_ayah_sequence();
        let exercise = AyahSequenceExercise::new(11, &mock).await.unwrap();

        let options = exercise.get_options();
        let correct_key = exercise.get_correct_verse_key();

        // Verify all options are unique
        let mut keys: Vec<&String> = options.iter().map(|(k, _)| k).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 4, "All 4 options should be unique");

        // Verify exactly one option is the correct answer
        let correct_count = options.iter().filter(|(k, _)| k == correct_key).count();
        assert_eq!(correct_count, 1, "Exactly one option should be correct");
    }
}
