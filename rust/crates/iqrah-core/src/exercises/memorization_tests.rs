// memorization_tests.rs
// Comprehensive tests for all memorization exercises

#[cfg(test)]
mod tests {
    use crate::exercises::{
        ClozeDeletionExercise, FirstLetterHintExercise, MemorizationExercise,
        MissingWordMcqExercise, NextWordDifficulty, NextWordMcqExercise,
    };
    use crate::{Chapter, ContentRepository, Node, NodeType, Verse, Word};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // ==========================================================================
    // Mock ContentRepository for Testing
    // ==========================================================================

    struct MockContentRepo {
        chapters: HashMap<i32, Chapter>,
        verses: HashMap<String, Verse>,
        words: HashMap<String, Vec<Word>>, // verse_key -> words
    }

    impl MockContentRepo {
        fn new() -> Self {
            let mut chapters = HashMap::new();
            let mut verses = HashMap::new();
            let mut words = HashMap::new();

            // Chapter 1: Al-Fatihah
            chapters.insert(
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

            // Verse 1:1 - "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
            verses.insert(
                "1:1".to_string(),
                Verse {
                    key: "1:1".to_string(),
                    chapter_number: 1,
                    verse_number: 1,
                    text_uthmani: "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                    text_simple: Some("بسم الله الرحمن الرحيم".to_string()),
                    juz: 1,
                    page: 1,
                },
            );

            // Words for verse 1:1
            words.insert(
                "1:1".to_string(),
                vec![
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
                        transliteration: Some("Allāhi".to_string()),
                    },
                    Word {
                        id: 3,
                        verse_key: "1:1".to_string(),
                        position: 3,
                        text_uthmani: "ٱلرَّحْمَٰنِ".to_string(),
                        text_simple: Some("الرحمن".to_string()),
                        transliteration: Some("al-Raḥmāni".to_string()),
                    },
                    Word {
                        id: 4,
                        verse_key: "1:1".to_string(),
                        position: 4,
                        text_uthmani: "ٱلرَّحِيمِ".to_string(),
                        text_simple: Some("الرحيم".to_string()),
                        transliteration: Some("al-Raḥīmi".to_string()),
                    },
                ],
            );

            // Verse 1:2 - "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ"
            verses.insert(
                "1:2".to_string(),
                Verse {
                    key: "1:2".to_string(),
                    chapter_number: 1,
                    verse_number: 2,
                    text_uthmani: "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string(),
                    text_simple: Some("الحمد لله رب العالمين".to_string()),
                    juz: 1,
                    page: 1,
                },
            );

            // Words for verse 1:2
            words.insert(
                "1:2".to_string(),
                vec![
                    Word {
                        id: 5,
                        verse_key: "1:2".to_string(),
                        position: 1,
                        text_uthmani: "ٱلْحَمْدُ".to_string(),
                        text_simple: Some("الحمد".to_string()),
                        transliteration: Some("al-ḥamdu".to_string()),
                    },
                    Word {
                        id: 6,
                        verse_key: "1:2".to_string(),
                        position: 2,
                        text_uthmani: "لِلَّهِ".to_string(),
                        text_simple: Some("لله".to_string()),
                        transliteration: Some("lillāhi".to_string()),
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
                        transliteration: Some("al-ʿālamīna".to_string()),
                    },
                ],
            );

            Self {
                chapters,
                verses,
                words,
            }
        }
    }

    #[async_trait]
    impl ContentRepository for MockContentRepo {
        async fn get_node(&self, _node_id: &str) -> anyhow::Result<Option<Node>> {
            Ok(Some(Node {
                id: "test".to_string(),
                node_type: NodeType::Word,
                knowledge_node: None,
            }))
        }

        async fn get_edges_from(&self, _source_id: &str) -> anyhow::Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
            // Handle WORD_INSTANCE:chapter:verse:position format
            if node_id.starts_with("WORD_INSTANCE:") || node_id.starts_with("WORD:") {
                let parts: Vec<&str> = node_id.split(':').collect();
                if parts.len() >= 4 {
                    let verse_key = format!("{}:{}", parts[1], parts[2]);
                    let position: i32 = parts[3].parse().unwrap_or(1);

                    if let Some(words) = self.words.get(&verse_key) {
                        if let Some(word) = words.iter().find(|w| w.position == position) {
                            return Ok(Some(word.text_uthmani.clone()));
                        }
                    }
                }
            }

            // Handle VERSE:chapter:verse format
            if node_id.starts_with("VERSE:") {
                let verse_key = node_id.strip_prefix("VERSE:").unwrap_or("");
                if let Some(verse) = self.verses.get(verse_key) {
                    return Ok(Some(verse.text_uthmani.clone()));
                }
            }

            Ok(None)
        }

        async fn get_translation(
            &self,
            _node_id: &str,
            _lang: &str,
        ) -> anyhow::Result<Option<String>> {
            Ok(Some("test translation".to_string()))
        }

        async fn get_metadata(&self, _node_id: &str, _key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn get_all_metadata(
            &self,
            _node_id: &str,
        ) -> anyhow::Result<HashMap<String, String>> {
            Ok(HashMap::new())
        }

        async fn node_exists(&self, _node_id: &str) -> anyhow::Result<bool> {
            Ok(true)
        }

        async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_nodes_by_type(&self, _node_type: NodeType) -> anyhow::Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn insert_nodes_batch(&self, _nodes: &[crate::ImportedNode]) -> anyhow::Result<()> {
            Ok(())
        }

        async fn insert_edges_batch(&self, _edges: &[crate::ImportedEdge]) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_words_in_ayahs(&self, _ayah_node_ids: &[String]) -> anyhow::Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_adjacent_words(
            &self,
            _word_node_id: &str,
        ) -> anyhow::Result<(Option<Node>, Option<Node>)> {
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
            Ok(self
                .verses
                .values()
                .filter(|v| v.chapter_number == chapter_number)
                .cloned()
                .collect())
        }

        async fn get_words_for_verse(&self, verse_key: &str) -> anyhow::Result<Vec<Word>> {
            Ok(self.words.get(verse_key).cloned().unwrap_or_default())
        }

        async fn get_word(&self, word_id: i32) -> anyhow::Result<Option<Word>> {
            for words in self.words.values() {
                if let Some(word) = words.iter().find(|w| w.id == word_id) {
                    return Ok(Some(word.clone()));
                }
            }
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
            Ok(1)
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
        ) -> anyhow::Result<Vec<crate::MorphologySegment>> {
            Ok(vec![])
        }

        async fn get_root_by_id(&self, _root_id: &str) -> anyhow::Result<Option<crate::Root>> {
            Ok(None)
        }

        async fn get_lemma_by_id(&self, _lemma_id: &str) -> anyhow::Result<Option<crate::Lemma>> {
            Ok(None)
        }
    }

    // ==========================================================================
    // Exercise 2: Next Word MCQ Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_next_word_mcq_easy_generates_correct_question() {
        let repo = MockContentRepo::new();
        let exercise =
            NextWordMcqExercise::new("VERSE:1:1".to_string(), NextWordDifficulty::Easy, &repo)
                .await
                .unwrap();

        use crate::exercises::Exercise;
        let question = exercise.generate_question();

        // Should contain all words except the last one
        assert!(question.contains("بِسْمِ"));
        assert!(question.contains("ٱللَّهِ"));
        assert!(question.contains("ٱلرَّحْمَٰنِ"));
        assert!(question.contains("_____")); // Placeholder for last word
    }

    #[tokio::test]
    async fn test_next_word_mcq_correct_answer() {
        let repo = MockContentRepo::new();
        let exercise =
            NextWordMcqExercise::new("VERSE:1:1".to_string(), NextWordDifficulty::Easy, &repo)
                .await
                .unwrap();

        use crate::exercises::Exercise;

        // Last word of 1:1 is "ٱلرَّحِيمِ"
        assert!(exercise.check_answer("ٱلرَّحِيمِ"));
        assert!(exercise.check_answer("الرحيم")); // Without tashkeel

        // Wrong answer
        assert!(!exercise.check_answer("ٱللَّهِ"));
    }

    #[tokio::test]
    async fn test_next_word_mcq_has_four_options() {
        let repo = MockContentRepo::new();
        let exercise =
            NextWordMcqExercise::new("VERSE:1:1".to_string(), NextWordDifficulty::Easy, &repo)
                .await
                .unwrap();

        let options = exercise.get_options();
        assert_eq!(options.len(), 4); // 1 correct + 3 distractors
    }

    #[tokio::test]
    async fn test_next_word_mcq_easy_distractors_from_same_verse() {
        let repo = MockContentRepo::new();
        let exercise =
            NextWordMcqExercise::new("VERSE:1:1".to_string(), NextWordDifficulty::Easy, &repo)
                .await
                .unwrap();

        let options = exercise.get_options();

        // Should contain the correct answer
        assert!(options.contains(&"ٱلرَّحِيمِ".to_string()));

        // Should contain distractors from the same verse
        let verse_words = ["بِسْمِ", "ٱللَّهِ", "ٱلرَّحْمَٰنِ", "ٱلرَّحِيمِ"];
        for option in options {
            assert!(verse_words.contains(&option.as_str()));
        }
    }

    #[tokio::test]
    async fn test_next_word_mcq_medium_distractors_from_same_chapter() {
        let repo = MockContentRepo::new();
        let exercise =
            NextWordMcqExercise::new("VERSE:1:1".to_string(), NextWordDifficulty::Medium, &repo)
                .await
                .unwrap();

        let options = exercise.get_options();
        assert_eq!(options.len(), 4);
        assert!(options.contains(&"ٱلرَّحِيمِ".to_string()));
    }

    // ==========================================================================
    // Exercise 3: Missing Word MCQ Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_missing_word_mcq_generates_blank() {
        let repo = MockContentRepo::new();
        let exercise = MissingWordMcqExercise::new(
            "WORD_INSTANCE:1:1:2".to_string(), // Second word "ٱللَّهِ"
            &repo,
        )
        .await
        .unwrap();

        use crate::exercises::Exercise;
        let question = exercise.generate_question();

        // Should have blank where "ٱللَّهِ" was
        assert!(question.contains("بِسْمِ"));
        assert!(question.contains("_____"));
        assert!(question.contains("ٱلرَّحْمَٰنِ"));
        assert!(question.contains("ٱلرَّحِيمِ"));
    }

    #[tokio::test]
    async fn test_missing_word_mcq_correct_answer() {
        let repo = MockContentRepo::new();
        let exercise = MissingWordMcqExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;

        assert!(exercise.check_answer("ٱللَّهِ"));
        assert!(exercise.check_answer("الله")); // Without tashkeel

        assert!(!exercise.check_answer("بِسْمِ"));
    }

    #[tokio::test]
    async fn test_missing_word_mcq_distractors_from_same_verse() {
        let repo = MockContentRepo::new();
        let exercise = MissingWordMcqExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
            .await
            .unwrap();

        let options = exercise.get_options();
        assert_eq!(options.len(), 4);

        // Should contain the correct answer
        assert!(options.contains(&"ٱللَّهِ".to_string()));
    }

    // ==========================================================================
    // Exercise 6: Cloze Deletion Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_cloze_deletion_basic() {
        let repo = MockContentRepo::new();
        let exercise = ClozeDeletionExercise::new(
            "WORD_INSTANCE:1:1:2".to_string(),
            false, // No letter hints
            &repo,
        )
        .await
        .unwrap();

        use crate::exercises::Exercise;
        let question = exercise.generate_question();

        assert!(question.contains("بِسْمِ"));
        assert!(question.contains("_____"));
        assert!(question.contains("ٱلرَّحْمَٰنِ"));
    }

    #[tokio::test]
    async fn test_cloze_deletion_normalization() {
        let repo = MockContentRepo::new();
        let exercise = ClozeDeletionExercise::new("WORD_INSTANCE:1:1:2".to_string(), false, &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;

        // With tashkeel
        assert!(exercise.check_answer("ٱللَّهِ"));

        // Without tashkeel
        assert!(exercise.check_answer("الله"));

        // With extra whitespace
        assert!(exercise.check_answer("  الله  "));
    }

    #[tokio::test]
    async fn test_cloze_deletion_with_hint_letters() {
        let repo = MockContentRepo::new();
        let exercise = ClozeDeletionExercise::new(
            "WORD_INSTANCE:1:1:2".to_string(),
            true, // Show letter hints
            &repo,
        )
        .await
        .unwrap();

        use crate::exercises::Exercise;
        let question = exercise.generate_question();

        // Should contain "Available letters:"
        assert!(question.contains("Available letters:"));

        // Should have jumbled letters
        let hint_letters = exercise.get_hint_letters();
        assert!(hint_letters.is_some());

        let letters = hint_letters.unwrap();
        assert!(!letters.is_empty());
    }

    #[tokio::test]
    async fn test_cloze_deletion_wrong_answer() {
        let repo = MockContentRepo::new();
        let exercise = ClozeDeletionExercise::new("WORD_INSTANCE:1:1:2".to_string(), false, &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;

        assert!(!exercise.check_answer("بِسْمِ"));
        assert!(!exercise.check_answer("wrong"));
    }

    #[tokio::test]
    async fn test_cloze_deletion_partial_match_rejected() {
        let repo = MockContentRepo::new();
        let exercise = ClozeDeletionExercise::new("WORD_INSTANCE:1:1:2".to_string(), false, &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;

        // Partial match should not be accepted
        assert!(!exercise.check_answer("الل")); // Incomplete word
    }

    // ==========================================================================
    // Exercise 7: First Letter Hint Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_first_letter_hint_display() {
        let repo = MockContentRepo::new();
        let exercise = FirstLetterHintExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;
        let question = exercise.generate_question();

        // Should contain first letter of "ٱللَّهِ"
        let first_letter = exercise.get_first_letter();
        assert_eq!(first_letter, 'ٱ');

        // Question should show first letter + blank
        assert!(question.contains(&format!("{}ـ_____", first_letter)));
    }

    #[tokio::test]
    async fn test_first_letter_hint_correct() {
        let repo = MockContentRepo::new();
        let exercise = FirstLetterHintExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;

        assert!(exercise.check_answer("ٱللَّهِ"));
        assert!(exercise.check_answer("الله"));
    }

    #[tokio::test]
    async fn test_first_letter_hint_normalization() {
        let repo = MockContentRepo::new();
        let exercise = FirstLetterHintExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
            .await
            .unwrap();

        use crate::exercises::Exercise;

        // Various forms of the same word
        assert!(exercise.check_answer("ٱللَّهِ")); // With tashkeel
        assert!(exercise.check_answer("الله")); // Without tashkeel
        assert!(exercise.check_answer(" الله ")); // With whitespace
    }

    // ==========================================================================
    // Helper Function Tests
    // ==========================================================================

    #[test]
    fn test_normalize_arabic() {
        assert_eq!(
            MemorizationExercise::normalize_arabic("بِسْمِ"),
            MemorizationExercise::normalize_arabic("بسم")
        );

        assert_eq!(
            MemorizationExercise::normalize_arabic("ٱللَّهِ"),
            MemorizationExercise::normalize_arabic("الله")
        );

        assert_eq!(MemorizationExercise::normalize_arabic("  test  "), "test");
    }
}
