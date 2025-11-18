// grammar_tests.rs
// Comprehensive tests for grammar exercises

#[cfg(test)]
mod tests {
    use crate::exercises::{Exercise, IdentifyRootExercise};
    use crate::{Chapter, ContentRepository, Node, NodeType, Verse, Word};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // ==========================================================================
    // Mock ContentRepository for Testing
    // ==========================================================================

    struct MockContentRepo {
        words_text: HashMap<String, String>,      // node_id -> text
        words: HashMap<String, Vec<crate::Word>>, // verse_key -> words
        morphology: HashMap<i32, Vec<crate::MorphologySegment>>, // word_id -> segments
        roots: HashMap<String, crate::Root>,      // root_id -> root
    }

    impl MockContentRepo {
        fn new() -> Self {
            let mut words_text = HashMap::new();
            let mut words = HashMap::new();
            let mut morphology = HashMap::new();
            let mut roots = HashMap::new();

            // Words from Al-Fatihah with text
            words_text.insert("WORD_INSTANCE:1:1:1".to_string(), "بِسْمِ".to_string());
            words_text.insert("WORD_INSTANCE:1:1:2".to_string(), "ٱللَّهِ".to_string());
            words_text.insert("WORD_INSTANCE:1:1:3".to_string(), "ٱلرَّحْمَٰنِ".to_string());
            words_text.insert("WORD_INSTANCE:1:1:4".to_string(), "ٱلرَّحِيمِ".to_string());
            words_text.insert("WORD_INSTANCE:1:2:1".to_string(), "ٱلْحَمْدُ".to_string());
            words_text.insert("WORD_INSTANCE:1:2:2".to_string(), "لِلَّهِ".to_string());
            words_text.insert("WORD_INSTANCE:1:2:3".to_string(), "رَبِّ".to_string());
            words_text.insert("WORD_INSTANCE:1:2:4".to_string(), "ٱلْعَٰلَمِينَ".to_string());

            // Create Word objects for verse 1:1
            let verse_1_1 = vec![
                crate::Word {
                    id: 1,
                    verse_key: "1:1".to_string(),
                    position: 1,
                    text_uthmani: "بِسْمِ".to_string(),
                    text_simple: Some("بسم".to_string()),
                    transliteration: Some("bismi".to_string()),
                },
                crate::Word {
                    id: 2,
                    verse_key: "1:1".to_string(),
                    position: 2,
                    text_uthmani: "ٱللَّهِ".to_string(),
                    text_simple: Some("الله".to_string()),
                    transliteration: Some("al-lahi".to_string()),
                },
                crate::Word {
                    id: 3,
                    verse_key: "1:1".to_string(),
                    position: 3,
                    text_uthmani: "ٱلرَّحْمَٰنِ".to_string(),
                    text_simple: Some("الرحمن".to_string()),
                    transliteration: Some("ar-rahmani".to_string()),
                },
                crate::Word {
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
                crate::Word {
                    id: 5,
                    verse_key: "1:2".to_string(),
                    position: 1,
                    text_uthmani: "ٱلْحَمْدُ".to_string(),
                    text_simple: Some("الحمد".to_string()),
                    transliteration: Some("al-hamdu".to_string()),
                },
                crate::Word {
                    id: 6,
                    verse_key: "1:2".to_string(),
                    position: 2,
                    text_uthmani: "لِلَّهِ".to_string(),
                    text_simple: Some("لله".to_string()),
                    transliteration: Some("lillahi".to_string()),
                },
                crate::Word {
                    id: 7,
                    verse_key: "1:2".to_string(),
                    position: 3,
                    text_uthmani: "رَبِّ".to_string(),
                    text_simple: Some("رب".to_string()),
                    transliteration: Some("rabbi".to_string()),
                },
                crate::Word {
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

            // Define roots
            roots.insert(
                "س-م-و".to_string(),
                crate::Root {
                    root_id: "س-م-و".to_string(),
                    arabic: "سمو".to_string(),
                    transliteration: Some("s-m-w".to_string()),
                    root_type: "trilateral".to_string(),
                },
            );
            roots.insert(
                "ا-ل-ه".to_string(),
                crate::Root {
                    root_id: "ا-ل-ه".to_string(),
                    arabic: "اله".to_string(),
                    transliteration: Some("'-l-h".to_string()),
                    root_type: "trilateral".to_string(),
                },
            );
            roots.insert(
                "ر-ح-م".to_string(),
                crate::Root {
                    root_id: "ر-ح-م".to_string(),
                    arabic: "رحم".to_string(),
                    transliteration: Some("r-h-m".to_string()),
                    root_type: "trilateral".to_string(),
                },
            );
            roots.insert(
                "ح-م-د".to_string(),
                crate::Root {
                    root_id: "ح-م-د".to_string(),
                    arabic: "حمد".to_string(),
                    transliteration: Some("h-m-d".to_string()),
                    root_type: "trilateral".to_string(),
                },
            );
            roots.insert(
                "ر-ب-ب".to_string(),
                crate::Root {
                    root_id: "ر-ب-ب".to_string(),
                    arabic: "ربب".to_string(),
                    transliteration: Some("r-b-b".to_string()),
                    root_type: "trilateral".to_string(),
                },
            );
            roots.insert(
                "ع-ل-م".to_string(),
                crate::Root {
                    root_id: "ع-ل-م".to_string(),
                    arabic: "علم".to_string(),
                    transliteration: Some("'-l-m".to_string()),
                    root_type: "trilateral".to_string(),
                },
            );

            // Create morphology segments for each word
            morphology.insert(
                1,
                vec![crate::MorphologySegment {
                    segment_id: 1,
                    word_id: 1,
                    position: 1,
                    lemma_id: Some("اسم".to_string()),
                    root_id: Some("س-م-و".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                2,
                vec![crate::MorphologySegment {
                    segment_id: 2,
                    word_id: 2,
                    position: 1,
                    lemma_id: Some("الله".to_string()),
                    root_id: Some("ا-ل-ه".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                3,
                vec![crate::MorphologySegment {
                    segment_id: 3,
                    word_id: 3,
                    position: 1,
                    lemma_id: Some("رحمن".to_string()),
                    root_id: Some("ر-ح-م".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                4,
                vec![crate::MorphologySegment {
                    segment_id: 4,
                    word_id: 4,
                    position: 1,
                    lemma_id: Some("رحيم".to_string()),
                    root_id: Some("ر-ح-م".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                5,
                vec![crate::MorphologySegment {
                    segment_id: 5,
                    word_id: 5,
                    position: 1,
                    lemma_id: Some("حمد".to_string()),
                    root_id: Some("ح-م-د".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                6,
                vec![crate::MorphologySegment {
                    segment_id: 6,
                    word_id: 6,
                    position: 1,
                    lemma_id: Some("الله".to_string()),
                    root_id: Some("ا-ل-ه".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                7,
                vec![crate::MorphologySegment {
                    segment_id: 7,
                    word_id: 7,
                    position: 1,
                    lemma_id: Some("رب".to_string()),
                    root_id: Some("ر-ب-ب".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );
            morphology.insert(
                8,
                vec![crate::MorphologySegment {
                    segment_id: 8,
                    word_id: 8,
                    position: 1,
                    lemma_id: Some("عالم".to_string()),
                    root_id: Some("ع-ل-م".to_string()),
                    pos_tag: Some("noun".to_string()),
                }],
            );

            Self {
                words_text,
                words,
                morphology,
                roots,
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
            Ok(self.words_text.get(node_id).cloned())
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

        async fn get_chapter(&self, _chapter_number: i32) -> anyhow::Result<Option<Chapter>> {
            Ok(None)
        }

        async fn get_chapters(&self) -> anyhow::Result<Vec<Chapter>> {
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

        async fn get_word(&self, word_id: i32) -> anyhow::Result<Option<Word>> {
            // Search through all verses to find the word
            for word_list in self.words.values() {
                if let Some(word) = word_list.iter().find(|w| w.id == word_id) {
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
            word_id: i32,
        ) -> anyhow::Result<Vec<crate::MorphologySegment>> {
            Ok(self.morphology.get(&word_id).cloned().unwrap_or_default())
        }

        async fn get_root_by_id(&self, root_id: &str) -> anyhow::Result<Option<crate::Root>> {
            Ok(self.roots.get(root_id).cloned())
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
    // Exercise 21: Identify the Root Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_identify_root_generates_question() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        let question = exercise.generate_question();

        // Should ask about the root of the word
        assert!(question.contains("root"));
        assert!(question.contains("بِسْمِ"));
    }

    #[tokio::test]
    async fn test_identify_root_correct_answer_bism() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        // Root of "بِسْمِ" is "س-م-و"
        assert!(exercise.check_answer("س-م-و"));
        assert!(exercise.check_answer("سمو")); // Without dashes
        assert!(exercise.check_answer("س م و")); // With spaces

        // Wrong roots
        assert!(!exercise.check_answer("ك-ت-ب"));
        assert!(!exercise.check_answer("ع-ل-م"));
    }

    #[tokio::test]
    async fn test_identify_root_correct_answer_allah() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:2".to_string(), &repo)
            .await
            .unwrap();

        // Root of "ٱللَّهِ" is "ا-ل-ه"
        assert!(exercise.check_answer("ا-ل-ه"));
        assert!(exercise.check_answer("اله")); // Without dashes
    }

    #[tokio::test]
    async fn test_identify_root_correct_answer_rahman() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:3".to_string(), &repo)
            .await
            .unwrap();

        // Root of "ٱلرَّحْمَٰنِ" is "ر-ح-م"
        assert!(exercise.check_answer("ر-ح-م"));
        assert!(exercise.check_answer("رحم")); // Without dashes

        let correct_root = exercise.get_correct_root();
        assert_eq!(correct_root, "ر-ح-م");
    }

    #[tokio::test]
    async fn test_identify_root_has_four_options() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        let options = exercise.get_options();
        assert_eq!(options.len(), 4); // 1 correct + 3 distractors
    }

    #[tokio::test]
    async fn test_identify_root_options_contain_correct() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        let options = exercise.get_options();
        let correct_root = exercise.get_correct_root();

        // Options should contain the correct root
        assert!(options.contains(&correct_root.to_string()));
    }

    #[tokio::test]
    async fn test_identify_root_distractors_are_different() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        let options = exercise.get_options();
        let correct_root = exercise.get_correct_root();

        // Should have exactly one instance of the correct root
        let correct_count = options.iter().filter(|&opt| opt == correct_root).count();
        assert_eq!(correct_count, 1);

        // All options should be unique
        let unique_options: std::collections::HashSet<_> = options.iter().collect();
        assert_eq!(unique_options.len(), options.len());
    }

    #[tokio::test]
    async fn test_identify_root_hint_shows_letter_count() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        let hint = exercise.get_hint();
        assert!(hint.is_some());

        let hint_text = hint.unwrap();
        // Should mention letter count (trilateral roots are 3 letters)
        assert!(hint_text.contains("letter"));
    }

    #[tokio::test]
    async fn test_identify_root_multiple_words() {
        let repo = MockContentRepo::new();

        // Test different words
        let words = vec![
            ("WORD_INSTANCE:1:1:1", "س-م-و"), // بِسْمِ
            ("WORD_INSTANCE:1:1:2", "ا-ل-ه"), // ٱللَّهِ
            ("WORD_INSTANCE:1:1:3", "ر-ح-م"), // ٱلرَّحْمَٰنِ
            ("WORD_INSTANCE:1:2:1", "ح-م-د"), // ٱلْحَمْدُ
            ("WORD_INSTANCE:1:2:3", "ر-ب-ب"), // رَبِّ
            ("WORD_INSTANCE:1:2:4", "ع-ل-م"), // ٱلْعَٰلَمِينَ
        ];

        for (word_id, expected_root) in words {
            let exercise = IdentifyRootExercise::new(word_id.to_string(), &repo)
                .await
                .unwrap();

            let actual_root = exercise.get_correct_root();
            assert_eq!(actual_root, expected_root, "Incorrect root for {}", word_id);

            assert!(exercise.check_answer(expected_root));
        }
    }

    #[tokio::test]
    async fn test_identify_root_type_name() {
        let repo = MockContentRepo::new();
        let exercise = IdentifyRootExercise::new("WORD_INSTANCE:1:1:1".to_string(), &repo)
            .await
            .unwrap();

        assert_eq!(exercise.get_type_name(), "identify_root");
    }
}
