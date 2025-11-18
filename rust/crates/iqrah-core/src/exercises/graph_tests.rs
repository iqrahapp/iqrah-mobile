// tests for graph-based exercises

use crate::exercises::graph::CrossVerseConnectionExercise;
use crate::exercises::types::Exercise;
use crate::{ContentRepository, DistributionType, Edge, EdgeType, MorphologySegment, Verse, Word};
use async_trait::async_trait;
use std::collections::HashMap;

// ==========================================================================
// Mock ContentRepository for Testing
// ==========================================================================

struct MockContentRepo {
    verses_text: HashMap<String, String>, // node_id -> text
    verses: HashMap<i32, Vec<Verse>>,     // chapter_num -> verses
    edges: HashMap<String, Vec<Edge>>,    // source_id -> edges
    words: HashMap<String, Vec<Word>>,    // verse_key -> words
}

impl MockContentRepo {
    fn new() -> Self {
        let mut verses_text = HashMap::new();
        let mut verses = HashMap::new();
        let mut edges = HashMap::new();
        let mut words = HashMap::new();

        // Verse 1:1 and 1:2 (Al-Fatihah)
        verses_text.insert(
            "VERSE:1:1".to_string(),
            "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
        );
        verses_text.insert("VERSE:1:2".to_string(), "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string());
        verses_text.insert("VERSE:1:3".to_string(), "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string());

        // Verses from chapter 112 (Al-Ikhlas)
        verses_text.insert("VERSE:112:1".to_string(), "قُلْ هُوَ ٱللَّهُ أَحَدٌ".to_string());
        verses_text.insert("VERSE:112:2".to_string(), "ٱللَّهُ ٱلصَّمَدُ".to_string());

        // Verses from chapter 113 (Al-Falaq)
        verses_text.insert("VERSE:113:1".to_string(), "قُلْ أَعُوذُ بِرَبِّ ٱلْفَلَقِ".to_string());

        // Define verses for chapter 1
        let chapter_1 = vec![
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
            Verse {
                key: "1:3".to_string(),
                chapter_number: 1,
                verse_number: 3,
                text_uthmani: "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                text_simple: Some("الرحمن الرحيم".to_string()),
                juz: 1,
                page: 1,
            },
        ];

        // Define verses for chapter 112
        let chapter_112 = vec![
            Verse {
                key: "112:1".to_string(),
                chapter_number: 112,
                verse_number: 1,
                text_uthmani: "قُلْ هُوَ ٱللَّهُ أَحَدٌ".to_string(),
                text_simple: Some("قل هو الله احد".to_string()),
                juz: 30,
                page: 604,
            },
            Verse {
                key: "112:2".to_string(),
                chapter_number: 112,
                verse_number: 2,
                text_uthmani: "ٱللَّهُ ٱلصَّمَدُ".to_string(),
                text_simple: Some("الله الصمد".to_string()),
                juz: 30,
                page: 604,
            },
        ];

        // Define verses for chapter 113
        let chapter_113 = vec![Verse {
            key: "113:1".to_string(),
            chapter_number: 113,
            verse_number: 1,
            text_uthmani: "قُلْ أَعُوذُ بِرَبِّ ٱلْفَلَقِ".to_string(),
            text_simple: Some("قل اعوذ برب الفلق".to_string()),
            juz: 30,
            page: 604,
        }];

        verses.insert(1, chapter_1);
        verses.insert(112, chapter_112);
        verses.insert(113, chapter_113);

        // Create graph edges (verse 1:1 connected to 112:1 via shared "Allah" concept)
        edges.insert(
            "VERSE:1:1".to_string(),
            vec![Edge {
                source_id: "VERSE:1:1".to_string(),
                target_id: "VERSE:112:1".to_string(),
                edge_type: EdgeType::Knowledge,
                distribution_type: DistributionType::Const,
                param1: 0.8,
                param2: 0.0,
            }],
        );

        // Verse 1:2 connected to 1:3 (sequential)
        edges.insert(
            "VERSE:1:2".to_string(),
            vec![Edge {
                source_id: "VERSE:1:2".to_string(),
                target_id: "VERSE:1:3".to_string(),
                edge_type: EdgeType::Dependency,
                distribution_type: DistributionType::Const,
                param1: 1.0,
                param2: 0.0,
            }],
        );

        // Add some words for morphology queries (minimal setup)
        words.insert(
            "1:1".to_string(),
            vec![Word {
                id: 1,
                verse_key: "1:1".to_string(),
                position: 1,
                text_uthmani: "بِسْمِ".to_string(),
                text_simple: Some("بسم".to_string()),
                transliteration: Some("bismi".to_string()),
            }],
        );

        Self {
            verses_text,
            verses,
            edges,
            words,
        }
    }
}

#[async_trait]
impl ContentRepository for MockContentRepo {
    async fn get_node(&self, _node_id: &str) -> anyhow::Result<Option<crate::Node>> {
        Ok(None)
    }

    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>> {
        Ok(self.edges.get(source_id).cloned().unwrap_or_default())
    }

    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
        Ok(self.verses_text.get(node_id).cloned())
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
// Exercise 19: Cross-Verse Connection Tests
// ==========================================================================

#[tokio::test]
async fn test_cross_verse_basic() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Verify question includes source verse
    let question = exercise.generate_question();
    assert!(question.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    assert!(question.contains("thematically connected"));
}

#[tokio::test]
async fn test_cross_verse_finds_connected_verse() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Verse 1:1 should be connected to 112:1 via graph edge
    assert_eq!(exercise.get_correct_verse_key(), "112:1");
}

#[tokio::test]
async fn test_cross_verse_has_four_options() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    let options = exercise.get_options();
    assert_eq!(options.len(), 4); // 1 correct + 3 distractors
}

#[tokio::test]
async fn test_cross_verse_options_contain_correct() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    let options = exercise.get_options();
    let correct_key = exercise.get_correct_verse_key();

    // Check that options contain the correct answer
    let has_correct = options.iter().any(|(key, _)| key == correct_key);
    assert!(has_correct);
}

#[tokio::test]
async fn test_cross_verse_check_answer_by_key() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Check answer by verse key
    assert!(exercise.check_answer("112:1"));
    assert!(!exercise.check_answer("1:2"));
}

#[tokio::test]
async fn test_cross_verse_check_answer_by_text() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    // Check answer by verse text
    assert!(exercise.check_answer("قُلْ هُوَ ٱللَّهُ أَحَدٌ"));
    assert!(!exercise.check_answer("ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"));
}

#[tokio::test]
async fn test_cross_verse_type_name() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    assert_eq!(exercise.get_type_name(), "cross_verse_connection");
}

#[tokio::test]
async fn test_cross_verse_hint() {
    let repo = MockContentRepo::new();
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:1".to_string(), &repo)
        .await
        .unwrap();

    let hint = exercise.get_hint();
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("Surah 112")); // Hint shows chapter number
}

#[tokio::test]
async fn test_cross_verse_fallback_to_same_chapter() {
    let repo = MockContentRepo::new();
    // Verse 1:2 has edges but should use fallback for missing connections
    let exercise = CrossVerseConnectionExercise::new("VERSE:1:2".to_string(), &repo)
        .await
        .unwrap();

    // Should find verse 1:3 as connected (via edge)
    assert_eq!(exercise.get_correct_verse_key(), "1:3");
}
