// full_verse_input_tests.rs
// Comprehensive tests for Exercise 9: Full Verse Input

use crate::exercises::full_verse_input::FullVerseInputExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, MorphologySegment, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    texts: HashMap<String, String>,    // node_id -> text
    words: HashMap<String, Vec<Word>>, // verse_key -> words
}

impl MockContentRepo {
    fn new() -> Self {
        let mut texts = HashMap::new();
        let mut words = HashMap::new();

        // Chapter 1: Al-Fatihah
        texts.insert("CHAPTER:1".to_string(), "الفاتحة".to_string());

        // Verse 1:1: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
        texts.insert(
            "VERSE:1:1".to_string(),
            "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
        );

        // Verse 1:2: ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ
        texts.insert("VERSE:1:2".to_string(), "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string());

        // Verse 1:3: ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
        texts.insert("VERSE:1:3".to_string(), "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string());

        // Chapter 2: Al-Baqarah
        texts.insert("CHAPTER:2".to_string(), "البقرة".to_string());

        // Verse 2:1: الٓمٓ
        texts.insert("VERSE:2:1".to_string(), "الٓمٓ".to_string());

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

        words.insert("1:1".to_string(), verse_1_1);

        Self { texts, words }
    }
}

#[async_trait]
impl ContentRepository for MockContentRepo {
    async fn get_node(&self, _node_id: &str) -> anyhow::Result<Option<crate::Node>> {
        Ok(None)
    }

    async fn get_edges_from(&self, _source_id: &str) -> anyhow::Result<Vec<crate::Edge>> {
        Ok(vec![])
    }

    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
        Ok(self.texts.get(node_id).cloned())
    }

    async fn get_translation(&self, _node_id: &str, _lang: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn get_metadata(&self, _node_id: &str, _key: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn get_all_metadata(&self, _node_id: &str) -> anyhow::Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    async fn node_exists(&self, _node_id: &str) -> anyhow::Result<bool> {
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

    async fn insert_nodes_batch(&self, _nodes: &[crate::ImportedNode]) -> anyhow::Result<()> {
        Ok(())
    }

    async fn insert_edges_batch(&self, _edges: &[crate::ImportedEdge]) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_words_in_ayahs(
        &self,
        _ayah_node_ids: &[String],
    ) -> anyhow::Result<Vec<crate::Node>> {
        Ok(vec![])
    }

    async fn get_adjacent_words(
        &self,
        _word_node_id: &str,
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
        _user_id: &str,
        _now_ts: i64,
    ) -> anyhow::Result<Vec<crate::scheduler_v2::CandidateNode>> {
        Ok(vec![])
    }

    async fn get_prerequisite_parents(
        &self,
        _node_ids: &[String],
    ) -> anyhow::Result<std::collections::HashMap<String, Vec<String>>> {
        Ok(std::collections::HashMap::new())
    }

    async fn get_goal(
        &self,
        _goal_id: &str,
    ) -> anyhow::Result<Option<crate::ports::content_repository::SchedulerGoal>> {
        Ok(None)
    }

    async fn get_nodes_for_goal(&self, _goal_id: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }
}

// ==========================================================================
// Tests
// ==========================================================================

#[tokio::test]
async fn test_full_verse_input_basic() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Verify exercise was created successfully
    assert_eq!(exercise.get_node_id(), "VERSE:1:1");
    assert_eq!(exercise.get_type_name(), "full_verse_input");
    assert_eq!(exercise.get_verse_key(), "1:1");
}

#[tokio::test]
async fn test_full_verse_input_question_format() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Verify question format
    let question = exercise.generate_question();
    assert!(question.contains("Type the complete verse"));
    assert!(question.contains("الفاتحة")); // Chapter name
    assert!(question.contains("verse 1"));
}

#[tokio::test]
async fn test_full_verse_input_check_answer_exact() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Exact match with tashkeel
    assert!(exercise.check_answer("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
}

#[tokio::test]
async fn test_full_verse_input_check_answer_normalized() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Without tashkeel (normalized)
    assert!(exercise.check_answer("بسم الله الرحمان الرحيم"));
}

#[tokio::test]
async fn test_full_verse_input_wrong_answer() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Wrong verse
    assert!(!exercise.check_answer("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"));

    // Partial verse
    assert!(!exercise.check_answer("بِسْمِ ٱللَّهِ"));

    // Empty answer
    assert!(!exercise.check_answer(""));
}

#[tokio::test]
async fn test_full_verse_input_hint() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Hint should show first word
    let hint = exercise.get_hint();
    assert!(hint.is_some());
    let hint_text = hint.unwrap();
    assert!(hint_text.contains("Starts with"));
    assert!(hint_text.contains("بسم")); // normalized "بِسْمِ"
}

#[tokio::test]
async fn test_full_verse_input_different_verse() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:2".to_string(), &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_verse_key(), "1:2");
    assert!(exercise.check_answer("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"));
    assert!(exercise.check_answer("الحمد لله رب العالمين")); // normalized
}

#[tokio::test]
async fn test_full_verse_input_different_chapter() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:2:1".to_string(), &repo)
        .await
        .unwrap();

    let question = exercise.generate_question();
    assert!(question.contains("البقرة")); // Chapter 2 name
    assert!(question.contains("verse 1"));

    assert!(exercise.check_answer("الٓمٓ"));
    assert!(exercise.check_answer("الم")); // normalized (without tashkeel)
}

#[tokio::test]
async fn test_full_verse_input_extra_whitespace() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Extra whitespace should be normalized
    assert!(exercise.check_answer("  بسم   الله   الرحمان   الرحيم  "));
}

#[tokio::test]
async fn test_full_verse_input_case_variations() {
    let repo = MockContentRepo::new();
    let exercise = FullVerseInputExercise::new("VERSE:1:3".to_string(), &repo)
        .await
        .unwrap();

    // Both should work (normalization handles case)
    assert!(exercise.check_answer("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    assert!(exercise.check_answer("الرحمان الرحيم"));
}
