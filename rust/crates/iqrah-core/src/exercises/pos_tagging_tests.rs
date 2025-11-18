// pos_tagging_tests.rs
// Comprehensive tests for Exercise 22: Part of Speech Tagging

use crate::exercises::pos_tagging::PosTaggingExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, MorphologySegment, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    texts: HashMap<String, String>,                   // node_id -> text
    words: HashMap<String, Vec<Word>>,                // verse_key -> words
    morphology: HashMap<i32, Vec<MorphologySegment>>, // word_id -> segments
}

impl MockContentRepo {
    fn new() -> Self {
        let mut texts = HashMap::new();
        let mut words = HashMap::new();
        let mut morphology = HashMap::new();

        // Verse 1:1: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
        texts.insert(
            "VERSE:1:1".to_string(),
            "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
        );
        texts.insert("WORD_INSTANCE:1:1:1".to_string(), "بِسْمِ".to_string());
        texts.insert("WORD_INSTANCE:1:1:2".to_string(), "ٱللَّهِ".to_string());
        texts.insert("WORD_INSTANCE:1:1:3".to_string(), "ٱلرَّحْمَٰنِ".to_string());

        // Verse 112:1: قُلْ هُوَ ٱللَّهُ أَحَدٌ
        texts.insert("VERSE:112:1".to_string(), "قُلْ هُوَ ٱللَّهُ أَحَدٌ".to_string());
        texts.insert("WORD_INSTANCE:112:1:1".to_string(), "قُلْ".to_string());
        texts.insert("WORD_INSTANCE:112:1:2".to_string(), "هُوَ".to_string());

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
        ];

        // Create Word objects for verse 112:1
        let verse_112_1 = vec![
            Word {
                id: 10,
                verse_key: "112:1".to_string(),
                position: 1,
                text_uthmani: "قُلْ".to_string(),
                text_simple: Some("قل".to_string()),
                transliteration: Some("qul".to_string()),
            },
            Word {
                id: 11,
                verse_key: "112:1".to_string(),
                position: 2,
                text_uthmani: "هُوَ".to_string(),
                text_simple: Some("هو".to_string()),
                transliteration: Some("huwa".to_string()),
            },
        ];

        words.insert("1:1".to_string(), verse_1_1);
        words.insert("112:1".to_string(), verse_112_1);

        // Create morphology segments with POS tags
        // Word 1: بِسْمِ (noun)
        morphology.insert(
            1,
            vec![MorphologySegment {
                segment_id: 1,
                word_id: 1,
                position: 1,
                lemma_id: Some("اسم".to_string()),
                root_id: Some("س-م-و".to_string()),
                pos_tag: Some("noun".to_string()),
            }],
        );

        // Word 2: ٱللَّهِ (noun - proper noun)
        morphology.insert(
            2,
            vec![MorphologySegment {
                segment_id: 2,
                word_id: 2,
                position: 1,
                lemma_id: Some("الله".to_string()),
                root_id: Some("ا-ل-ه".to_string()),
                pos_tag: Some("noun".to_string()),
            }],
        );

        // Word 3: ٱلرَّحْمَٰنِ (noun - adjective)
        morphology.insert(
            3,
            vec![MorphologySegment {
                segment_id: 3,
                word_id: 3,
                position: 1,
                lemma_id: Some("رحمن".to_string()),
                root_id: Some("ر-ح-م".to_string()),
                pos_tag: Some("noun".to_string()),
            }],
        );

        // Word 10: قُلْ (verb - imperative)
        morphology.insert(
            10,
            vec![MorphologySegment {
                segment_id: 10,
                word_id: 10,
                position: 1,
                lemma_id: Some("قول".to_string()),
                root_id: Some("ق-و-ل".to_string()),
                pos_tag: Some("verb".to_string()),
            }],
        );

        // Word 11: هُوَ (pronoun)
        morphology.insert(
            11,
            vec![MorphologySegment {
                segment_id: 11,
                word_id: 11,
                position: 1,
                lemma_id: None,
                root_id: None,
                pos_tag: Some("pronoun".to_string()),
            }],
        );

        Self {
            texts,
            words,
            morphology,
        }
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
        word_id: i32,
    ) -> anyhow::Result<Vec<MorphologySegment>> {
        Ok(self.morphology.get(&word_id).cloned().unwrap_or_default())
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
async fn test_pos_tagging_basic() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Verify exercise was created successfully
    assert_eq!(exercise.get_node_id(), "WORD_INSTANCE:1:1:1");
    assert_eq!(exercise.get_type_name(), "pos_tagging");
    assert_eq!(exercise.get_correct_pos(), "noun");
}

#[tokio::test]
async fn test_pos_tagging_question_format() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    let question = exercise.generate_question();
    assert!(question.contains("What part of speech"));
    assert!(question.contains("بِسْمِ"));
    assert!(question.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
}

#[tokio::test]
async fn test_pos_tagging_options_generated() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    let options = exercise.get_options();
    assert_eq!(options.len(), 4);
    assert!(options.contains(&"noun".to_string()));
}

#[tokio::test]
async fn test_pos_tagging_check_answer_correct() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    assert!(exercise.check_answer("noun"));
    assert!(exercise.check_answer("Noun")); // Case insensitive
    assert!(exercise.check_answer("NOUN"));
}

#[tokio::test]
async fn test_pos_tagging_check_answer_wrong() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    assert!(!exercise.check_answer("verb"));
    assert!(!exercise.check_answer("particle"));
    assert!(!exercise.check_answer("pronoun"));
}

#[tokio::test]
async fn test_pos_tagging_hint() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    let hint = exercise.get_hint();
    assert!(hint.is_some());
    let hint_text = hint.unwrap();
    assert!(hint_text.contains("Hint"));
}

#[tokio::test]
async fn test_pos_tagging_verb() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:112:1:1".to_string(), &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_correct_pos(), "verb");
    assert!(exercise.check_answer("verb"));
    assert!(!exercise.check_answer("noun"));
}

#[tokio::test]
async fn test_pos_tagging_pronoun() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:112:1:2".to_string(), &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_correct_pos(), "pronoun");
    assert!(exercise.check_answer("pronoun"));
    assert!(!exercise.check_answer("verb"));
}

#[tokio::test]
async fn test_pos_tagging_different_words() {
    let repo = MockContentRepo::new();

    // Test word 2: ٱللَّهِ (noun)
    let exercise2 = PosTaggingExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
        .await
        .unwrap();
    assert_eq!(exercise2.get_correct_pos(), "noun");

    // Test word 3: ٱلرَّحْمَٰنِ (noun)
    let exercise3 = PosTaggingExercise::new("WORD_INSTANCE:1:1:3".to_string(), &repo)
        .await
        .unwrap();
    assert_eq!(exercise3.get_correct_pos(), "noun");
}

#[tokio::test]
async fn test_pos_tagging_whitespace_normalization() {
    let repo = MockContentRepo::new();
    let exercise = PosTaggingExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Extra whitespace should be normalized
    assert!(exercise.check_answer("  noun  "));
    assert!(exercise.check_answer("\tnoun\n"));
}
