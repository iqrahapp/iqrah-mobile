// exercises/ayah_chain_tests.rs
// Comprehensive tests for Exercise 10: Ayah Chain

use super::*;
use crate::domain::{Chapter, Verse};
use crate::ContentRepository;
use async_trait::async_trait;
use std::collections::HashMap;

// ============================================================================
// Mock Repository for Testing
// ============================================================================

struct MockContentRepo {
    verses: HashMap<String, Verse>,
    chapters: HashMap<i32, Chapter>,
}

impl MockContentRepo {
    fn new() -> Self {
        let mut repo = Self {
            verses: HashMap::new(),
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

        // Add all 7 verses of Al-Fatihah
        let verses_data = [
            ("1:1", "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
            ("1:2", "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"),
            ("1:3", "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"),
            ("1:4", "مَٰلِكِ يَوْمِ ٱلدِّينِ"),
            ("1:5", "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ"),
            ("1:6", "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ"),
            ("1:7", "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ"),
        ];

        for (i, (verse_key, text)) in verses_data.iter().enumerate() {
            let verse_num = i + 1;
            repo.verses.insert(
                verse_key.to_string(),
                Verse {
                    key: verse_key.to_string(),
                    chapter_number: 1,
                    verse_number: verse_num as i32,
                    text_uthmani: text.to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
            );
        }

        repo
    }
}

#[async_trait]
impl ContentRepository for MockContentRepo {
    async fn get_node(&self, _node_id: &str) -> anyhow::Result<Option<crate::Node>> {
        Ok(None)
    }

    async fn get_edges_from(&self, _source_id: &str) -> anyhow::Result<Vec<crate::Edge>> {
        Ok(Vec::new())
    }

    async fn get_edges_to(&self, _target_id: &str) -> anyhow::Result<Vec<crate::Edge>> {
        Ok(Vec::new())
    }

    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
        if let Some(verse_key) = node_id.strip_prefix("VERSE:") {
            Ok(self.verses.get(verse_key).map(|v| v.text_uthmani.clone()))
        } else {
            Ok(None)
        }
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
        Ok(Vec::new())
    }

    async fn get_nodes_by_type(
        &self,
        _node_type: crate::NodeType,
    ) -> anyhow::Result<Vec<crate::Node>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
    }

    async fn get_adjacent_words(
        &self,
        _word_node_id: &str,
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

    async fn get_words_for_verse(&self, _verse_key: &str) -> anyhow::Result<Vec<crate::Word>> {
        Ok(Vec::new())
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
        _user_id: &str,
        _now_ts: i64,
        _user_repo: &dyn crate::ports::user_repository::UserRepository,
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

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_ayah_chain_creation() {
    let repo = MockContentRepo::new();
    let exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

    let stats = exercise.get_stats();
    assert_eq!(stats.total_verses, 7); // Al-Fatihah has 7 verses
    assert_eq!(stats.completed_count, 0);
    assert!(!stats.is_complete);
    assert!(!stats.mistake_made);
}

#[tokio::test]
async fn test_ayah_chain_first_verse() {
    let repo = MockContentRepo::new();
    let exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

    let current = exercise.current_verse().unwrap();
    assert_eq!(current.key, "1:1");
    assert_eq!(current.text_uthmani, "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
}

#[tokio::test]
async fn test_ayah_chain_correct_answer_advances() {
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

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
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

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
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

    // Submit answer without tashkeel - should still be correct
    // Note: ٰ (alif khanjariyyah U+0670) normalizes to regular alif,
    // so "ٱلرَّحْمَٰنِ" → "الرحمان" (with middle alif)
    let result = exercise.submit_answer("بسم الله الرحمان الرحيم").unwrap();
    assert!(result); // Should be correct due to normalization
}

#[tokio::test]
async fn test_ayah_chain_complete_all_verses() {
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

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
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

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
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

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
    let repo = MockContentRepo::new();
    let exercise = AyahChainExercise::new_range(1, 1, 3, &repo).await.unwrap();

    let stats = exercise.get_stats();
    assert_eq!(stats.total_verses, 3); // Only verses 1-3

    let current = exercise.current_verse().unwrap();
    assert_eq!(current.key, "1:1");
}

#[tokio::test]
async fn test_ayah_chain_question_format() {
    let repo = MockContentRepo::new();
    let exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

    let question = exercise.generate_question();
    assert!(question.contains("1/7")); // Progress indicator
    assert!(question.contains("1:1")); // Verse reference
}

#[tokio::test]
async fn test_ayah_chain_hint() {
    let repo = MockContentRepo::new();
    let exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

    let hint = exercise.get_hint().unwrap();
    // Updated: Now shows first word + word count for cleaner UX
    assert!(hint.contains("بِسْمِ")); // First word
    assert!(hint.contains("4 words total")); // Word count (verse 1:1 has 4 words)
}

#[tokio::test]
async fn test_ayah_chain_check_answer_method() {
    let repo = MockContentRepo::new();
    let exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

    // Test Exercise trait check_answer method
    assert!(exercise.check_answer("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"));
    assert!(!exercise.check_answer("wrong answer"));
}

#[tokio::test]
async fn test_ayah_chain_cannot_submit_after_complete() {
    let repo = MockContentRepo::new();
    let mut exercise = AyahChainExercise::new("CHAPTER:1".to_string(), &repo)
        .await
        .unwrap();

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
