// tests for translation exercises

use crate::exercises::translation::ContextualTranslationExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    words_text: HashMap<i64, String>,        // node_id -> text
    words: HashMap<String, Vec<Word>>,       // verse_key -> words
    word_translations: HashMap<i64, String>, // word_id -> translation
    verses: HashMap<i32, Vec<Verse>>,        // chapter_num -> verses
}

impl MockContentRepo {
    fn new() -> Self {
        let mut words_text = HashMap::new();
        let mut words = HashMap::new();
        let mut word_translations = HashMap::new();
        let mut verses = HashMap::new();

        // Verse 1:1 words
        words_text.insert(111, "بِسْمِ".to_string());
        words_text.insert(112, "ٱللَّهِ".to_string());
        words_text.insert(113, "ٱلرَّحْمَٰنِ".to_string());
        words_text.insert(114, "ٱلرَّحِيمِ".to_string());

        // Verse 1:2 words
        words_text.insert(121, "ٱلْحَمْدُ".to_string());
        words_text.insert(122, "لِلَّهِ".to_string());
        words_text.insert(123, "رَبِّ".to_string());
        words_text.insert(124, "ٱلْعَٰلَمِينَ".to_string());

        // Verse texts
        words_text.insert(11, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string());
        words_text.insert(12, "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string());

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
                transliteration: Some("al-lahi".to_string()),
            },
            Word {
                id: 3,
                verse_key: "1:1".to_string(),
                position: 3,
                text_uthmani: "ٱلرَّحْمَٰنِ".to_string(),
                text_simple: Some("الرحمن".to_string()),
                transliteration: Some("ar-rahmani".to_string()),
            },
            Word {
                id: 4,
                verse_key: "1:1".to_string(),
                position: 4,
                text_uthmani: "ٱلرَّحِيمِ".to_string(),
                text_simple: Some("الرحيم".to_string()),
                transliteration: Some("ar-rahimi".to_string()),
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
                transliteration: Some("al-hamdu".to_string()),
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
                transliteration: Some("al-'alamina".to_string()),
            },
        ];

        words.insert("1:1".to_string(), verse_1_1);
        words.insert("1:2".to_string(), verse_1_2);

        // Word translations (translator_id = 1)
        word_translations.insert(1, "In the name".to_string());
        word_translations.insert(2, "of Allah".to_string());
        word_translations.insert(3, "the Most Gracious".to_string());
        word_translations.insert(4, "the Most Merciful".to_string());
        word_translations.insert(5, "All praise".to_string());
        word_translations.insert(6, "is due to Allah".to_string());
        word_translations.insert(7, "Lord".to_string());
        word_translations.insert(8, "of the worlds".to_string());

        // Verses for chapter 1
        let chapter_1_verses = vec![
            Verse {
                key: "1:1".to_string(),
                chapter_number: 1,
                verse_number: 1,
                text_uthmani: "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                text_simple: Some("بسم الله الرحمن الرحيم".to_string()),
                juz: 1,
                page: 1,
            },
            Verse {
                key: "1:2".to_string(),
                chapter_number: 1,
                verse_number: 2,
                text_uthmani: "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string(),
                text_simple: Some("الحمد لله رب العالمين".to_string()),
                juz: 1,
                page: 1,
            },
        ];

        verses.insert(1, chapter_1_verses);

        Self {
            words_text,
            words,
            word_translations,
            verses,
        }
    }
}

#[async_trait]
impl ContentRepository for MockContentRepo {
    async fn get_node(&self, node_id: i64) -> anyhow::Result<Option<crate::Node>> {
        let (ukey, node_type) = match node_id {
            // Verse nodes
            11 => ("VERSE:1:1".to_string(), crate::NodeType::Verse),
            12 => ("VERSE:1:2".to_string(), crate::NodeType::Verse),
            // Word instance nodes from verse 1:1
            111 => (
                "WORD_INSTANCE:1:1:1".to_string(),
                crate::NodeType::WordInstance,
            ),
            112 => (
                "WORD_INSTANCE:1:1:2".to_string(),
                crate::NodeType::WordInstance,
            ),
            113 => (
                "WORD_INSTANCE:1:1:3".to_string(),
                crate::NodeType::WordInstance,
            ),
            114 => (
                "WORD_INSTANCE:1:1:4".to_string(),
                crate::NodeType::WordInstance,
            ),
            // Word instance nodes from verse 1:2
            121 => (
                "WORD_INSTANCE:1:2:1".to_string(),
                crate::NodeType::WordInstance,
            ),
            122 => (
                "WORD_INSTANCE:1:2:2".to_string(),
                crate::NodeType::WordInstance,
            ),
            123 => (
                "WORD_INSTANCE:1:2:3".to_string(),
                crate::NodeType::WordInstance,
            ),
            124 => (
                "WORD_INSTANCE:1:2:4".to_string(),
                crate::NodeType::WordInstance,
            ),
            _ => return Ok(None),
        };
        Ok(Some(crate::Node {
            id: node_id,
            ukey,
            node_type,
        }))
    }
    async fn get_node_by_ukey(&self, ukey: &str) -> anyhow::Result<Option<crate::Node>> {
        let (id, node_type) = match ukey {
            // Verse nodes
            "VERSE:1:1" => (11, crate::NodeType::Verse),
            "VERSE:1:2" => (12, crate::NodeType::Verse),
            // Word instance nodes from verse 1:1
            "WORD_INSTANCE:1:1:1" => (111, crate::NodeType::WordInstance),
            "WORD_INSTANCE:1:1:2" => (112, crate::NodeType::WordInstance),
            "WORD_INSTANCE:1:1:3" => (113, crate::NodeType::WordInstance),
            "WORD_INSTANCE:1:1:4" => (114, crate::NodeType::WordInstance),
            // Word instance nodes from verse 1:2
            "WORD_INSTANCE:1:2:1" => (121, crate::NodeType::WordInstance),
            "WORD_INSTANCE:1:2:2" => (122, crate::NodeType::WordInstance),
            "WORD_INSTANCE:1:2:3" => (123, crate::NodeType::WordInstance),
            "WORD_INSTANCE:1:2:4" => (124, crate::NodeType::WordInstance),
            _ => return Ok(None),
        };
        Ok(Some(crate::Node {
            id,
            ukey: ukey.to_string(),
            node_type,
        }))
    }

    async fn get_edges_from(&self, _source_id: i64) -> anyhow::Result<Vec<crate::Edge>> {
        Ok(vec![])
    }

    async fn get_quran_text(&self, node_id: i64) -> anyhow::Result<Option<String>> {
        Ok(self.words_text.get(&node_id).cloned())
    }

    async fn get_translation(&self, node_id: i64, _lang: &str) -> anyhow::Result<Option<String>> {
        // Map node_id to word_id
        // 111-114 map to word_ids 1-4 (verse 1:1)
        // 121-124 map to word_ids 5-8 (verse 1:2)
        let word_id = match node_id {
            111 => 1,
            112 => 2,
            113 => 3,
            114 => 4, // Verse 1:1
            121 => 5,
            122 => 6,
            123 => 7,
            124 => 8, // Verse 1:2
            _ => return Ok(None),
        };
        Ok(self.word_translations.get(&word_id).cloned())
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

    async fn get_words_for_verse(&self, verse_key: &str) -> anyhow::Result<Vec<Word>> {
        Ok(self.words.get(verse_key).cloned().unwrap_or_default())
    }

    async fn get_word(&self, _word_id: i64) -> anyhow::Result<Option<Word>> {
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
        word_id: i64,
        _translator_id: i32,
    ) -> anyhow::Result<Option<String>> {
        Ok(self.word_translations.get(&word_id).cloned())
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
        _word_id: i64,
    ) -> anyhow::Result<Vec<crate::MorphologySegment>> {
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
        word_ids: &[i64],
    ) -> anyhow::Result<std::collections::HashMap<i64, crate::Word>> {
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
// Exercise 15: Contextual Translation MCQ Tests
// ==========================================================================

#[tokio::test]
async fn test_contextual_translation_basic() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(111, "WORD_INSTANCE:1:1:1", &repo)
        .await
        .unwrap();

    // Verify question includes verse context
    let question = exercise.generate_question();
    assert!(question.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    assert!(question.contains("بِسْمِ")); // Highlighted word
    assert!(question.contains("in this context"));
}

#[tokio::test]
async fn test_contextual_translation_correct_answer() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(111, "WORD_INSTANCE:1:1:1", &repo)
        .await
        .unwrap();

    // Check correct answer
    assert!(exercise.check_answer("In the name"));
    assert!(exercise.check_answer("in the name")); // Case insensitive
    assert!(exercise.check_answer("  in the name  ")); // Whitespace handling

    // Check wrong answer
    assert!(!exercise.check_answer("of Allah"));
}

#[tokio::test]
async fn test_contextual_translation_has_four_options() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(112, "WORD_INSTANCE:1:1:2", &repo)
        .await
        .unwrap();

    let options = exercise.get_options();
    assert_eq!(options.len(), 4); // 1 correct + 3 distractors
}

#[tokio::test]
async fn test_contextual_translation_options_contain_correct() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(112, "WORD_INSTANCE:1:1:2", &repo)
        .await
        .unwrap();

    let options = exercise.get_options();
    assert!(options.contains(&"of Allah".to_string()));
}

#[tokio::test]
async fn test_contextual_translation_distractors_are_different() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(113, "WORD_INSTANCE:1:1:3", &repo)
        .await
        .unwrap();

    let options = exercise.get_options();
    let correct = exercise.get_correct_answer();

    // Filter out the correct answer to get distractors
    let distractors: Vec<_> = options.iter().filter(|&opt| opt != correct).collect();

    // All distractors should be different from correct
    assert_eq!(distractors.len(), 3);
    for distractor in distractors {
        assert_ne!(distractor, correct);
    }
}

#[tokio::test]
async fn test_contextual_translation_type_name() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(111, "WORD_INSTANCE:1:1:1", &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_type_name(), "contextual_translation");
}

#[tokio::test]
async fn test_contextual_translation_hint() {
    let repo = MockContentRepo::new();
    let exercise = ContextualTranslationExercise::new(111, "WORD_INSTANCE:1:1:1", &repo)
        .await
        .unwrap();

    let hint = exercise.get_hint();
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("I")); // First letter of "In the name"
}
