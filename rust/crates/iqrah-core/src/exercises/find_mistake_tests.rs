// exercises/find_mistake_tests.rs
// Comprehensive tests for Exercise 11: Find the Mistake

use super::*;
use crate::domain::{Chapter, Verse, Word};
use crate::ContentRepository;
use async_trait::async_trait;
use std::collections::HashMap;

// ============================================================================
// Mock Repository for Testing
// ============================================================================

struct MockContentRepo {
    verses: HashMap<String, Verse>,
    words: HashMap<String, Vec<Word>>,
    chapters: HashMap<i32, Chapter>,
}

impl MockContentRepo {
    fn new() -> Self {
        let mut repo = Self {
            verses: HashMap::new(),
            words: HashMap::new(),
            chapters: HashMap::new(),
        };

        // Add Al-Fatihah (Chapter 1)
        repo.chapters.insert(
            1,
            Chapter {
                number: 1,
                name_arabic: "الفاتحة".to_string(),
                name_transliteration: "Al-Fatihah".to_string(),
                name_translation: "The Opening".to_string(),
                revelation_place: Some("makkah".to_string()),
                verse_count: 7,
            },
        );

        // Add verses
        let verses_data = [
            ("1:1", "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
            ("1:2", "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"),
            ("1:3", "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
            ("1:4", "مَٰلِكِ يَوْمِ ٱلدِّينِ"),
        ];

        for (verse_key, text) in &verses_data {
            repo.verses.insert(
                verse_key.to_string(),
                Verse {
                    key: verse_key.to_string(),
                    chapter_number: 1,
                    verse_number: verse_key.split(':').nth(1).unwrap().parse().unwrap(),
                    text_uthmani: text.to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
            );

            // Add words for each verse
            let words: Vec<&str> = text.split_whitespace().collect();
            let mut word_list = Vec::new();
            for (i, word_text) in words.iter().enumerate() {
                word_list.push(Word {
                    id: (i + 1) as i32,
                    verse_key: verse_key.to_string(),
                    position: (i + 1) as i32,
                    text_uthmani: word_text.to_string(),
                    text_simple: None,
                    transliteration: None,
                });
            }
            repo.words.insert(verse_key.to_string(), word_list);
        }

        repo
    }
}

#[async_trait]
impl ContentRepository for MockContentRepo {
    async fn get_node(&self, _node_id: i64) -> anyhow::Result<Option<crate::Node>> {
        Ok(None)
    }
    async fn get_node_by_ukey(&self, _ukey: &str) -> anyhow::Result<Option<crate::Node>> {
        unimplemented!()
    }

    async fn get_edges_from(&self, _source_id: i64) -> anyhow::Result<Vec<crate::Edge>> {
        Ok(Vec::new())
    }

    async fn get_quran_text(&self, _node_id: i64) -> anyhow::Result<Option<String>> {
        // This mock is verse-key based, not i64. This is a hack.
        // In a real scenario, we'd look up the ukey from the i64.
        let verse_key = format!("1:{}", _node_id);
        Ok(self.verses.get(&verse_key).map(|v| v.text_uthmani.clone()))
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
        Ok(Vec::new())
    }

    async fn get_nodes_by_type(
        &self,
        _node_type: crate::NodeType,
    ) -> anyhow::Result<Vec<crate::Node>> {
        Ok(Vec::new())
    }

    async fn get_words_in_ayahs(
        &self,
        _ayah_node_ids: &[i64],
    ) -> anyhow::Result<Vec<crate::Node>> {
        Ok(Vec::new())
    }

    async fn get_adjacent_words(
        &self,
        _word_node_id: i64,
    ) -> anyhow::Result<(Option<crate::Node>, Option<crate::Node>)> {
        Ok((None, None))
    }

    async fn get_chapter(&self, chapter_number: i32) -> anyhow::Result<Option<Chapter>> {
        Ok(self.chapters.get(&chapter_number).cloned())
    }

    async fn get_chapters(&self) -> anyhow::Result<Vec<Chapter>> {
        Ok(self.chapters.values().cloned().collect())
    }

    async fn get_verse(&self, verse_key: &str) -> anyhow::Result<Option<Verse>> {
        Ok(self.verses.get(verse_key).cloned())
    }

    async fn get_verses_for_chapter(&self, chapter_number: i32) -> anyhow::Result<Vec<Verse>> {
        let mut verses: Vec<Verse> = self
            .verses
            .values()
            .filter(|v| v.chapter_number == chapter_number)
            .cloned()
            .collect();
        verses.sort_by_key(|v| v.verse_number);
        Ok(verses)
    }

    async fn get_words_for_verse(&self, verse_key: &str) -> anyhow::Result<Vec<crate::Word>> {
        Ok(self.words.get(verse_key).cloned().unwrap_or_default())
    }

    async fn get_word(&self, _word_id: i32) -> anyhow::Result<Option<crate::Word>> {
        Ok(None)
    }

    async fn get_languages(&self) -> anyhow::Result<Vec<crate::Language>> {
        Ok(Vec::new())
    }

    async fn get_language(&self, _code: &str) -> anyhow::Result<Option<crate::Language>> {
        Ok(None)
    }

    async fn get_translators_for_language(
        &self,
        _language_code: &str,
    ) -> anyhow::Result<Vec<crate::Translator>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
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
        Ok(Vec::new())
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
        Ok(Vec::new())
    }

    async fn get_morphology_for_word(
        &self,
        _word_id: i32,
    ) -> anyhow::Result<Vec<crate::MorphologySegment>> {
        Ok(Vec::new())
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

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_find_mistake_creation() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    // Should have a mistake position
    let position = exercise.get_mistake_position();
    assert!(position >= 1);
    assert!(position <= 4); // Verse 1:1 has 4 words

    // Modified verse should be different from correct verse
    assert_ne!(exercise.get_modified_verse(), "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
}

#[tokio::test]
async fn test_find_mistake_has_different_word() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    // Correct and incorrect words should be different
    assert_ne!(exercise.get_correct_word(), exercise.get_incorrect_word());
}

#[tokio::test]
async fn test_find_mistake_modified_verse_structure() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    // Modified verse should have same number of words as original
    let original_word_count = "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".split_whitespace().count();
    let modified_word_count = exercise.get_modified_verse().split_whitespace().count();
    assert_eq!(original_word_count, modified_word_count);
}

#[tokio::test]
async fn test_find_mistake_check_position() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(2, &repo).await.unwrap();

    let correct_position = exercise.get_mistake_position();

    // Correct position should return true
    assert!(exercise.check_position(correct_position));

    // Wrong positions should return false
    for pos in 1..=5 {
        if pos != correct_position {
            assert!(!exercise.check_position(pos));
        }
    }
}

#[tokio::test]
async fn test_find_mistake_check_answer() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(2, &repo).await.unwrap();

    let correct_position = exercise.get_mistake_position();

    // String version of position should work
    assert!(exercise.check_answer(&correct_position.to_string()));

    // Wrong position should fail
    let wrong_position = if correct_position == 1 { 2 } else { 1 };
    assert!(!exercise.check_answer(&wrong_position.to_string()));

    // Non-numeric answer should fail
    assert!(!exercise.check_answer("not a number"));
}

#[tokio::test]
async fn test_find_mistake_question_format() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    let question = exercise.generate_question();
    assert!(question.contains("Find the mistake"));
    assert!(question.contains(exercise.get_modified_verse()));
}

#[tokio::test]
async fn test_find_mistake_hint() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    let hint = exercise.get_hint().unwrap();
    assert!(hint.contains("1:1")); // Should mention verse reference
}

#[tokio::test]
async fn test_find_mistake_avoids_first_and_last() {
    let repo = MockContentRepo::new();

    // Test multiple times due to randomness
    for _ in 0..10 {
        let exercise = FindMistakeExercise::new(2, &repo).await.unwrap();

        let position = exercise.get_mistake_position();
        let word_count = "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".split_whitespace().count();

        // For verses with more than 3 words, should avoid first and last
        if word_count > 3 {
            assert!(position > 1);
            assert!(position < word_count as i32);
        }
    }
}

#[tokio::test]
async fn test_find_mistake_replacement_from_same_chapter() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    let incorrect_word = exercise.get_incorrect_word();

    // Verify the incorrect word exists in other verses of the same chapter
    let mut found_in_other_verses = false;
    for verse_key in ["1:2", "1:3", "1:4"] {
        let words = repo.get_words_for_verse(verse_key).await.unwrap();
        if words.iter().any(|w| w.text_uthmani == incorrect_word) {
            found_in_other_verses = true;
            break;
        }
    }

    assert!(
        found_in_other_verses,
        "Replacement word should come from another verse in the same chapter"
    );
}

#[tokio::test]
async fn test_find_mistake_type_name() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    assert_eq!(exercise.get_type_name(), "find_mistake");
}

#[tokio::test]
async fn test_find_mistake_node_id() {
    let repo = MockContentRepo::new();
    let exercise = FindMistakeExercise::new(1, &repo).await.unwrap();

    assert_eq!(exercise.get_node_id(), 1);
}
