// translate_phrase_tests.rs
// Comprehensive tests for Exercise 20: Translate Phrase (Text Input)

use crate::exercises::translate_phrase::TranslatePhraseExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, MorphologySegment, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    texts: HashMap<String, String>,                     // node_id -> text
    verse_translations: HashMap<(String, i32), String>, // (verse_key, translator_id) -> translation
    word_translations: HashMap<(i32, i32), String>,     // (word_id, translator_id) -> translation
    words: HashMap<String, Vec<Word>>,                  // verse_key -> words
}

impl MockContentRepo {
    fn new() -> Self {
        let mut texts = HashMap::new();
        let mut verse_translations = HashMap::new();
        let mut word_translations = HashMap::new();
        let mut words = HashMap::new();

        // Verse 1:1: بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ
        texts.insert(
            "VERSE:1:1".to_string(),
            "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
        );
        verse_translations.insert(
            ("1:1".to_string(), 1),
            "In the name of Allah, the Entirely Merciful, the Especially Merciful.".to_string(),
        );

        // Verse 1:2: ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ
        texts.insert("VERSE:1:2".to_string(), "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string());
        verse_translations.insert(
            ("1:2".to_string(), 1),
            "All praise is due to Allah, Lord of the worlds.".to_string(),
        );

        // Verse 112:1: قُلْ هُوَ ٱللَّهُ أَحَدٌ
        texts.insert("VERSE:112:1".to_string(), "قُلْ هُوَ ٱللَّهُ أَحَدٌ".to_string());
        verse_translations.insert(
            ("112:1".to_string(), 1),
            "Say, He is Allah, the One.".to_string(),
        );

        // Word instances for verse 1:1
        texts.insert("WORD_INSTANCE:1:1:1".to_string(), "بِسْمِ".to_string());
        texts.insert("WORD_INSTANCE:1:1:2".to_string(), "ٱللَّهِ".to_string());

        // Word translations
        word_translations.insert((1, 1), "In the name".to_string());
        word_translations.insert((2, 1), "of Allah".to_string());

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
        ];

        words.insert("1:1".to_string(), verse_1_1);

        Self {
            texts,
            verse_translations,
            word_translations,
            words,
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

    async fn get_edges_to(&self, _target_id: &str) -> anyhow::Result<Vec<crate::Edge>> {
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
        verse_key: &str,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        Ok(self
            .verse_translations
            .get(&(verse_key.to_string(), translator_id))
            .cloned())
    }

    async fn get_word_translation(
        &self,
        word_id: i32,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        Ok(self
            .word_translations
            .get(&(word_id, translator_id))
            .cloned())
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
async fn test_translate_phrase_verse_basic() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    // Verify exercise was created successfully
    assert_eq!(exercise.get_node_id(), "VERSE:1:1");
    assert_eq!(exercise.get_type_name(), "translate_phrase");
    assert_eq!(exercise.get_arabic_text(), "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
}

#[tokio::test]
async fn test_translate_phrase_question_format() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    let question = exercise.generate_question();
    assert!(question.contains("Translate to English"));
    assert!(question.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    assert!(question.contains("Type the English translation"));
}

#[tokio::test]
async fn test_translate_phrase_check_answer_exact() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    // Exact match
    assert!(exercise
        .check_answer("In the name of Allah, the Entirely Merciful, the Especially Merciful."));
}

#[tokio::test]
async fn test_translate_phrase_check_answer_normalized() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    // Case insensitive
    assert!(exercise
        .check_answer("in the name of allah, the entirely merciful, the especially merciful."));

    // Without punctuation
    assert!(
        exercise.check_answer("in the name of allah the entirely merciful the especially merciful")
    );
}

#[tokio::test]
async fn test_translate_phrase_wrong_answer() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    // Wrong translation
    assert!(!exercise.check_answer("All praise is due to Allah"));

    // Partial translation
    assert!(!exercise.check_answer("In the name of Allah"));

    // Empty answer
    assert!(!exercise.check_answer(""));
}

#[tokio::test]
async fn test_translate_phrase_hint() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    let hint = exercise.get_hint();
    assert!(hint.is_some());
    let hint_text = hint.unwrap();
    assert!(hint_text.contains("Starts with"));
    assert!(hint_text.contains("In the name"));
}

#[tokio::test]
async fn test_translate_phrase_different_verse() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:2".to_string(), 1, &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_verse_key(), Some("1:2"));
    assert!(exercise.check_answer("All praise is due to Allah, Lord of the worlds."));
    assert!(exercise.check_answer("all praise is due to allah lord of the worlds"));
}

#[tokio::test]
async fn test_translate_phrase_word_level() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("WORD_INSTANCE:1:1:1".to_string(), 1, &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_arabic_text(), "بِسْمِ");
    assert_eq!(exercise.get_correct_translation(), "In the name");
    assert!(exercise.check_answer("In the name"));
    assert!(exercise.check_answer("in the name"));
}

#[tokio::test]
async fn test_translate_phrase_extra_whitespace() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:1:2".to_string(), 1, &repo)
        .await
        .unwrap();

    // Extra whitespace should be normalized
    assert!(exercise.check_answer("  all  praise  is  due  to  allah  lord  of  the  worlds  "));
}

#[tokio::test]
async fn test_translate_phrase_different_punctuation() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("VERSE:112:1".to_string(), 1, &repo)
        .await
        .unwrap();

    // Different punctuation variations
    assert!(exercise.check_answer("Say, He is Allah, the One."));
    assert!(exercise.check_answer("Say He is Allah the One"));
    assert!(exercise.check_answer("say! he is allah: the one?"));
}

#[tokio::test]
async fn test_translate_phrase_normalization() {
    let repo = MockContentRepo::new();
    let exercise = TranslatePhraseExercise::new("WORD_INSTANCE:1:1:2".to_string(), 1, &repo)
        .await
        .unwrap();

    // Test English normalization
    assert!(exercise.check_answer("of Allah"));
    assert!(exercise.check_answer("OF ALLAH"));
    assert!(exercise.check_answer("  of  Allah  "));
    assert!(exercise.check_answer("of (Allah)"));
}
