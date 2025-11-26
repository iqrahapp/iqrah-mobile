// reverse_cloze_tests.rs
// Comprehensive tests for Exercise 8: Reverse Cloze

use crate::exercises::reverse_cloze::ReverseClozeExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, MorphologySegment, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    words_text: HashMap<i64, String>,     // node_id -> text
    words: HashMap<String, Vec<Word>>,    // verse_key -> words
    verses_text: HashMap<i64, String>,    // node_id -> text
}

impl MockContentRepo {
    fn new() -> Self {
        let mut words_text = HashMap::new();
        let mut words = HashMap::new();
        let mut verses_text = HashMap::new();

        // Verse 1:1: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
        words_text.insert(111, "بِسْمِ".to_string());
        words_text.insert(112, "ٱللَّهِ".to_string());
        words_text.insert(113, "ٱلرَّحْمَٰنِ".to_string());
        words_text.insert(114, "ٱلرَّحِيمِ".to_string());

        // Verse 1:2: ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ
        words_text.insert(121, "ٱلْحَمْدُ".to_string());
        words_text.insert(122, "لِلَّهِ".to_string());
        words_text.insert(123, "رَبِّ".to_string());
        words_text.insert(124, "ٱلْعَٰلَمِينَ".to_string());

        // Verse texts
        verses_text.insert(11, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string());
        verses_text.insert(12, "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string());

        // Create Word objects for verse 1:1
        let verse_1_1 = vec![
            Word {
                id: 1,
                verse_key: "1:1".to_string(),
                position: 1,
                text_uthmani: "بِسْمِ".to_string(),
                text_simple: Some("بسم".to_string()),
                transliteration: Some("bismi".to_string()),
            },
            Word {
                id: 2,
                verse_key: "1:1".to_string(),
                position: 2,
                text_uthmani: "ٱللَّهِ".to_string(),
                text_simple: Some("الله".to_string()),
                transliteration: Some("l-lahi".to_string()),
            },
            Word {
                id: 3,
                verse_key: "1:1".to_string(),
                position: 3,
                text_uthmani: "ٱلرَّحْمَٰنِ".to_string(),
                text_simple: Some("الرحمن".to_string()),
                transliteration: Some("l-raḥmāni".to_string()),
            },
            Word {
                id: 4,
                verse_key: "1:1".to_string(),
                position: 4,
                text_uthmani: "ٱلرَّحِيمِ".to_string(),
                text_simple: Some("الرحيم".to_string()),
                transliteration: Some("l-raḥīmi".to_string()),
            },
        ];

        // Create Word objects for verse 1:2
        let verse_1_2 = vec![
            Word {
                id: 5,
                verse_key: "1:2".to_string(),
                position: 1,
                text_uthmani: "ٱلْحَمْدُ".to_string(),
                text_simple: Some("الحمد".to_string()),
                transliteration: Some("l-ḥamdu".to_string()),
            },
            Word {
                id: 6,
                verse_key: "1:2".to_string(),
                position: 2,
                text_uthmani: "لِلَّهِ".to_string(),
                text_simple: Some("لله".to_string()),
                transliteration: Some("lillahi".to_string()),
            },
            Word {
                id: 7,
                verse_key: "1:2".to_string(),
                position: 3,
                text_uthmani: "رَبِّ".to_string(),
                text_simple: Some("رب".to_string()),
                transliteration: Some("rabbi".to_string()),
            },
            Word {
                id: 8,
                verse_key: "1:2".to_string(),
                position: 4,
                text_uthmani: "ٱلْعَٰلَمِينَ".to_string(),
                text_simple: Some("العالمين".to_string()),
                transliteration: Some("l-ʿālamīna".to_string()),
            },
        ];

        words.insert("1:1".to_string(), verse_1_1);
        words.insert("1:2".to_string(), verse_1_2);

        Self {
            words_text,
            words,
            verses_text,
        }
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
        Ok(vec![])
    }

    async fn get_quran_text(&self, node_id: i64) -> anyhow::Result<Option<String>> {
        Ok(self
            .words_text
            .get(&node_id)
            .or_else(|| self.verses_text.get(&node_id))
            .cloned())
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

    async fn get_words_in_ayahs(
        &self,
        _ayah_node_ids: &[i64],
    ) -> anyhow::Result<Vec<crate::Node>> {
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

    async fn get_verses_for_chapter(&self, _chapter_number: i32) -> anyhow::Result<Vec<Verse>> {
        Ok(vec![])
    }

    async fn get_words_for_verse(&self, verse_key: &str) -> anyhow::Result<Vec<Word>> {
        Ok(self.words.get(verse_key).cloned().unwrap_or_default())
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
async fn test_reverse_cloze_basic() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Verify exercise was created successfully
    assert_eq!(exercise.get_node_id(), "WORD_INSTANCE:1:1:1");
    assert_eq!(exercise.get_type_name(), "reverse_cloze");

    // Verify question is generated
    let question = exercise.generate_question();
    assert!(question.contains("بِسْمِ"));
    assert!(question.contains("What comes next"));
}

#[tokio::test]
async fn test_reverse_cloze_finds_next_word() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(111, &repo).await.unwrap();

    // After "بِسْمِ" (position 1), next word should be "ٱللَّهِ" (position 2)
    assert_eq!(exercise.get_next_word(), "ٱللَّهِ");
}

#[tokio::test]
async fn test_reverse_cloze_check_answer_exact() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(111, &repo).await.unwrap();

    // Exact match with tashkeel
    assert!(exercise.check_answer("ٱللَّهِ"));
}

#[tokio::test]
async fn test_reverse_cloze_check_answer_normalized() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(111, &repo).await.unwrap();

    // Without tashkeel (normalized)
    assert!(exercise.check_answer("الله"));
}

#[tokio::test]
async fn test_reverse_cloze_wrong_answer() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(111, &repo).await.unwrap();

    // Wrong word
    assert!(!exercise.check_answer("ٱلرَّحْمَٰنِ"));
    assert!(!exercise.check_answer("بِسْمِ"));
}

#[tokio::test]
async fn test_reverse_cloze_hint() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(111, &repo).await.unwrap();

    // Hint should show first letter
    let hint = exercise.get_hint();
    assert!(hint.is_some());
    let hint_text = hint.unwrap();
    assert!(hint_text.contains("Starts with"));
    // Normalized "ٱللَّهِ" starts with 'ا'
    assert!(hint_text.contains("ا"));
}

#[tokio::test]
async fn test_reverse_cloze_middle_word() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(112, &repo).await.unwrap();

    // After "ٱللَّهِ" (position 2), next word should be "ٱلرَّحْمَٰنِ" (position 3)
    assert_eq!(exercise.get_next_word(), "ٱلرَّحْمَٰنِ");
    assert!(exercise.check_answer("ٱلرَّحْمَٰنِ"));
    // Note: ٰ (alif khanjariyyah U+0670) normalizes to regular alif
    // So "ٱلرَّحْمَٰنِ" → "الرحمان" (with middle alif from khanjariyyah)
    assert!(exercise.check_answer("الرحمان")); // normalized
}

#[tokio::test]
async fn test_reverse_cloze_last_word_fails() {
    let repo = MockContentRepo::new();
    let result = ReverseClozeExercise::new(114, &repo).await;

    // Last word (position 4) should fail because there's no next word
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No next word found"));
}

#[tokio::test]
async fn test_reverse_cloze_different_verse() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(121, &repo).await.unwrap();

    // Verse 1:2, after "ٱلْحَمْدُ" (position 1), next word should be "لِلَّهِ" (position 2)
    assert_eq!(exercise.get_next_word(), "لِلَّهِ");
    assert!(exercise.check_answer("لِلَّهِ"));
    assert!(exercise.check_answer("لله")); // normalized
}

#[tokio::test]
async fn test_reverse_cloze_case_insensitivity() {
    let repo = MockContentRepo::new();
    let exercise = ReverseClozeExercise::new(122, &repo).await.unwrap();

    // After "لِلَّهِ", next should be "رَبِّ"
    assert!(exercise.check_answer("رَبِّ"));
    assert!(exercise.check_answer("ربّ")); // normalized
    assert!(exercise.check_answer("رب")); // without shadda
}
