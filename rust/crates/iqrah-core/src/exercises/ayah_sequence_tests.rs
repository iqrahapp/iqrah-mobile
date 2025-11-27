// ayah_sequence_tests.rs
// Comprehensive tests for Exercise 4: Ayah Sequence

use crate::exercises::ayah_sequence::AyahSequenceExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, MorphologySegment, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    verses_text: HashMap<i64, String>, // node_id -> text
    verses: HashMap<i32, Vec<Verse>>,  // chapter_num -> verses
}

impl MockContentRepo {
    fn new() -> Self {
        let mut verses_text = HashMap::new();
        let mut verses = HashMap::new();

        // Chapter 1 (Al-Fatihah) - 7 verses
        verses_text.insert(11, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string());
        verses_text.insert(12, "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string());
        verses_text.insert(13, "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string());
        verses_text.insert(14, "مَٰلِكِ يَوْمِ ٱلدِّينِ".to_string());
        verses_text.insert(15, "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ".to_string());
        verses_text.insert(16, "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ".to_string());
        verses_text.insert(
            17,
            "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ".to_string(),
        );

        // Define verses for chapter 1
        let chapter_1 = vec![
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
                text_uthmani: "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ".to_string(),
                text_simple: None,
                juz: 1,
                page: 1,
            },
        ];

        verses.insert(1, chapter_1);

        Self {
            verses_text,
            verses,
        }
    }
}

#[async_trait]
impl ContentRepository for MockContentRepo {
    async fn get_node(&self, node_id: i64) -> anyhow::Result<Option<crate::Node>> {
        // Mock nodes based on ID
        let ukey = match node_id {
            11 => "VERSE:1:1",
            12 => "VERSE:1:2",
            13 => "VERSE:1:3",
            14 => "VERSE:1:4",
            15 => "VERSE:1:5",
            16 => "VERSE:1:6",
            17 => "VERSE:1:7",
            _ => return Ok(None),
        };

        Ok(Some(crate::Node {
            id: node_id,
            ukey: ukey.to_string(),
            node_type: crate::NodeType::Verse,
        }))
    }

    async fn get_node_by_ukey(&self, ukey: &str) -> anyhow::Result<Option<crate::Node>> {
        let id = match ukey {
            "VERSE:1:1" => 11,
            "VERSE:1:2" => 12,
            "VERSE:1:3" => 13,
            "VERSE:1:4" => 14,
            "VERSE:1:5" => 15,
            "VERSE:1:6" => 16,
            "VERSE:1:7" => 17,
            _ => return Ok(None),
        };

        Ok(Some(crate::Node {
            id,
            ukey: ukey.to_string(),
            node_type: crate::NodeType::Verse,
        }))
    }

    async fn get_edges_from(&self, _source_id: i64) -> anyhow::Result<Vec<crate::Edge>> {
        Ok(vec![])
    }

    async fn get_quran_text(&self, node_id: i64) -> anyhow::Result<Option<String>> {
        Ok(self.verses_text.get(&node_id).cloned())
    }

    async fn get_translation(&self, _node_id: i64, _lang: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn get_metadata(&self, _node_id: i64, _key: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn get_all_metadata(&self, _node_id: i64) -> anyhow::Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    async fn node_exists(&self, _node_id: i64) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn get_all_nodes(&self) -> anyhow::Result<Vec<crate::Node>> {
        Ok(vec![])
    }

    async fn get_nodes_by_type(
        &self,
        _node_type: crate::NodeType,
    ) -> anyhow::Result<Vec<crate::Node>> {
        Ok(vec![])
    }

    async fn get_words_in_ayahs(&self, _ayah_node_ids: &[i64]) -> anyhow::Result<Vec<crate::Node>> {
        Ok(vec![])
    }

    async fn get_adjacent_words(
        &self,
        _word_node_id: i64,
    ) -> anyhow::Result<(Option<crate::Node>, Option<crate::Node>)> {
        Ok((None, None))
    }

    async fn get_chapter(&self, _chapter_number: i32) -> anyhow::Result<Option<crate::Chapter>> {
        Ok(None)
    }

    async fn get_chapters(&self) -> anyhow::Result<Vec<crate::Chapter>> {
        Ok(vec![])
    }

    async fn get_verse(&self, _verse_key: &str) -> anyhow::Result<Option<Verse>> {
        Ok(None)
    }

    async fn get_verses_for_chapter(&self, chapter_number: i32) -> anyhow::Result<Vec<Verse>> {
        Ok(self
            .verses
            .get(&chapter_number)
            .cloned()
            .unwrap_or_default())
    }

    async fn get_words_for_verse(&self, _verse_key: &str) -> anyhow::Result<Vec<Word>> {
        Ok(vec![])
    }

    async fn get_word(&self, _word_id: i32) -> anyhow::Result<Option<Word>> {
        Ok(None)
    }

    async fn get_languages(&self) -> anyhow::Result<Vec<crate::Language>> {
        Ok(vec![])
    }

    async fn get_language(&self, _code: &str) -> anyhow::Result<Option<crate::Language>> {
        Ok(None)
    }

    async fn get_translators_for_language(
        &self,
        _language_code: &str,
    ) -> anyhow::Result<Vec<crate::Translator>> {
        Ok(vec![])
    }

    async fn get_translator(
        &self,
        _translator_id: i32,
    ) -> anyhow::Result<Option<crate::Translator>> {
        Ok(None)
    }

    async fn get_translator_by_slug(
        &self,
        _slug: &str,
    ) -> anyhow::Result<Option<crate::Translator>> {
        Ok(None)
    }

    async fn get_verse_translation(
        &self,
        _verse_key: &str,
        _translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn get_word_translation(
        &self,
        _word_id: i32,
        _translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn insert_translator(
        &self,
        _slug: &str,
        _full_name: &str,
        _language_code: &str,
        _description: Option<&str>,
        _copyright_holder: Option<&str>,
        _license: Option<&str>,
        _website: Option<&str>,
        _version: Option<&str>,
        _package_id: Option<&str>,
    ) -> anyhow::Result<i32> {
        Ok(0)
    }

    async fn insert_verse_translation(
        &self,
        _verse_key: &str,
        _translator_id: i32,
        _translation: &str,
        _footnotes: Option<&str>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_available_packages(
        &self,
        _package_type: Option<crate::PackageType>,
        _language_code: Option<&str>,
    ) -> anyhow::Result<Vec<crate::ContentPackage>> {
        Ok(vec![])
    }

    async fn get_package(
        &self,
        _package_id: &str,
    ) -> anyhow::Result<Option<crate::ContentPackage>> {
        Ok(None)
    }

    async fn upsert_package(&self, _package: &crate::ContentPackage) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_package(&self, _package_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_installed_packages(&self) -> anyhow::Result<Vec<crate::InstalledPackage>> {
        Ok(vec![])
    }

    async fn is_package_installed(&self, _package_id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn mark_package_installed(&self, _package_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn mark_package_uninstalled(&self, _package_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn enable_package(&self, _package_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn disable_package(&self, _package_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_enabled_packages(&self) -> anyhow::Result<Vec<crate::InstalledPackage>> {
        Ok(vec![])
    }

    async fn get_morphology_for_word(
        &self,
        _word_id: i32,
    ) -> anyhow::Result<Vec<MorphologySegment>> {
        Ok(vec![])
    }

    async fn get_root_by_id(&self, _root_id: &str) -> anyhow::Result<Option<crate::Root>> {
        Ok(None)
    }

    async fn get_lemma_by_id(&self, _lemma_id: &str) -> anyhow::Result<Option<crate::Lemma>> {
        Ok(None)
    }

    async fn get_scheduler_candidates(
        &self,
        _goal_id: &str,
    ) -> anyhow::Result<Vec<crate::scheduler_v2::CandidateNode>> {
        Ok(vec![])
    }

    async fn get_prerequisite_parents(
        &self,
        _node_ids: &[i64],
    ) -> anyhow::Result<std::collections::HashMap<i64, Vec<i64>>> {
        Ok(std::collections::HashMap::new())
    }

    async fn get_goal(
        &self,
        _goal_id: &str,
    ) -> anyhow::Result<Option<crate::ports::content_repository::SchedulerGoal>> {
        Ok(None)
    }

    async fn get_nodes_for_goal(&self, _goal_id: &str) -> anyhow::Result<Vec<i64>> {
        Ok(vec![])
    }

    async fn get_verses_batch(
        &self,
        verse_keys: &[String],
    ) -> anyhow::Result<std::collections::HashMap<String, crate::Verse>> {
        let mut result = std::collections::HashMap::new();
        for key in verse_keys {
            if let Some(verse) = self.get_verse(key).await? {
                result.insert(key.clone(), verse);
            }
        }
        Ok(result)
    }

    async fn get_words_batch(
        &self,
        word_ids: &[i32],
    ) -> anyhow::Result<std::collections::HashMap<i32, crate::Word>> {
        let mut result = std::collections::HashMap::new();
        for &id in word_ids {
            if let Some(word) = self.get_word(id).await? {
                result.insert(id, word);
            }
        }
        Ok(result)
    }
}

// ==========================================================================
// Tests
// ==========================================================================

#[tokio::test]
async fn test_ayah_sequence_basic() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Verify exercise was created successfully
    assert_eq!(exercise.get_node_id(), 11);
    assert_eq!(exercise.get_type_name(), "ayah_sequence");

    // Verify question is generated
    let question = exercise.generate_question();
    assert!(question.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    assert!(question.contains("Which verse comes next"));
}

#[tokio::test]
async fn test_ayah_sequence_finds_correct_next_verse() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Verify correct next verse is verse 1:2
    assert_eq!(exercise.get_correct_verse_key(), "1:2");

    // Verify exercise knows the next verse text
    assert!(exercise.correct_next_verse_text.contains("ٱلْحَمْدُ لِلَّهِ"));
}

#[tokio::test]
async fn test_ayah_sequence_has_four_options() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Verify we have 4 MCQ options
    assert_eq!(exercise.get_options().len(), 4);
}

#[tokio::test]
async fn test_ayah_sequence_options_contain_correct() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Verify correct answer is in the options
    let correct_key = exercise.get_correct_verse_key();
    let options = exercise.get_options();

    let correct_in_options = options.iter().any(|(key, _)| key == correct_key);
    assert!(correct_in_options, "Correct answer must be in options");
}

#[tokio::test]
async fn test_ayah_sequence_check_answer_by_key() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Check answer by verse key
    assert!(exercise.check_answer("1:2"));
    assert!(!exercise.check_answer("1:3"));
    assert!(!exercise.check_answer("1:1"));
}

#[tokio::test]
async fn test_ayah_sequence_check_answer_by_text() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Check answer by verse text
    assert!(exercise.check_answer("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"));
    assert!(!exercise.check_answer("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
}

#[tokio::test]
async fn test_ayah_sequence_hint() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    // Verify hint shows verse number
    let hint = exercise.get_hint();
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("2"));
}

#[tokio::test]
async fn test_ayah_sequence_middle_verse() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(13, &repo).await.unwrap();

    // Verify correct next verse is 1:4
    assert_eq!(exercise.get_correct_verse_key(), "1:4");
    assert!(exercise.check_answer("1:4"));
}

#[tokio::test]
async fn test_ayah_sequence_last_verse_fails() {
    let repo = MockContentRepo::new();
    let result = AyahSequenceExercise::new(17, &repo).await;

    // Last verse should fail because there's no next verse
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No next verse found"));
}

#[tokio::test]
async fn test_ayah_sequence_distractors_are_different() {
    let repo = MockContentRepo::new();
    let exercise = AyahSequenceExercise::new(11, &repo).await.unwrap();

    let options = exercise.get_options();
    let current_key = &exercise.current_verse_key;
    let correct_key = exercise.get_correct_verse_key();

    // Verify all options are different from each other
    let mut keys: Vec<&String> = options.iter().map(|(k, _)| k).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), 4, "All 4 options should be unique");

    // Verify distractors don't include the current verse
    assert!(!options.iter().any(|(k, _)| k == current_key));

    // Verify exactly one option is the correct answer
    let correct_count = options.iter().filter(|(k, _)| k == correct_key).count();
    assert_eq!(correct_count, 1, "Exactly one option should be correct");
}
