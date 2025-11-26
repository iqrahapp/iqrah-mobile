// exercises/service.rs
// Exercise service for generating axis-specific exercises

use super::ayah_sequence::AyahSequenceExercise;
use super::grammar::IdentifyRootExercise;
use super::graph::CrossVerseConnectionExercise;
use super::mcq::McqExercise;
use super::memorization::MemorizationExercise;
use super::translation::{ContextualTranslationExercise, TranslationExercise};
use super::types::{Exercise, ExerciseResponse, ExerciseType};
use crate::semantic::grader::{SemanticGrader, SEMANTIC_EMBEDDER};
use crate::semantic::SemanticEmbedder;
use crate::{ContentRepository, KnowledgeAxis, KnowledgeNode};
use anyhow::Result;
use std::sync::Arc;

// Modern enum-based architecture
use super::exercise_data::ExerciseData;
use super::generators;

/// Service for generating and managing exercises
pub struct ExerciseService {
    content_repo: Arc<dyn ContentRepository>,
}

impl ExerciseService {
    pub fn new(content_repo: Arc<dyn ContentRepository>) -> Self {
        Self { content_repo }
    }

    /// Initialize the semantic grading model
    ///
    /// This should be called once at application startup to load the semantic model.
    /// After initialization, all TranslationExercise instances will use semantic grading.
    ///
    /// # Arguments
    /// * `model_path` - Path to the model2vec model (local path or HuggingFace model ID)
    /// * `cache_dir` - Optional cache directory for model files (important for mobile!)
    ///   - If provided, sets HF_HOME to this directory before loading model
    ///   - On mobile, this should be the app's documents directory
    ///   - If None, uses system default (~/.cache/huggingface on Linux/macOS)
    ///
    /// # Returns
    /// Ok(()) if the model was loaded successfully, Err if loading failed
    ///
    /// # Example (Flutter FFI)
    /// ```rust,no_run
    /// # fn main() -> anyhow::Result<()> {
    /// use iqrah_core::ExerciseService;
    ///
    /// // From Flutter, after getting app documents directory:
    /// let cache_dir = "/data/user/0/com.example.app/files/huggingface";
    /// ExerciseService::init_semantic_model(
    ///     "minishlab/potion-multilingual-128M",
    ///     Some(cache_dir)
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn init_semantic_model(model_path: &str, cache_dir: Option<&str>) -> Result<()> {
        tracing::info!("Initializing semantic grading model: {}", model_path);

        // Set HF_HOME if cache directory is provided (important for mobile)
        if let Some(dir) = cache_dir {
            tracing::info!("Setting model cache directory (HF_HOME): {}", dir);
            std::env::set_var("HF_HOME", dir);
        } else {
            tracing::warn!(
                "No cache directory provided. Using system default. \
                 For mobile apps, provide cache_dir parameter!"
            );
        }

        let embedder = SemanticEmbedder::new(model_path)?;

        SEMANTIC_EMBEDDER
            .set(embedder)
            .map_err(|_| anyhow::anyhow!("Semantic embedder already initialized"))?;

        tracing::info!("✅ Semantic grading model initialized successfully");
        Ok(())
    }

    /// Generate an exercise for a given node ID
    /// Automatically detects the knowledge axis from the node ID
    pub async fn generate_exercise(&self, node_id: &str) -> Result<ExerciseType> {
        // Parse node to determine axis
        let axis = if let Some(kn) = KnowledgeNode::parse(node_id) {
            kn.axis
        } else {
            // Default to memorization for non-knowledge nodes
            KnowledgeAxis::Memorization
        };

        // Generate exercise based on axis
        match axis {
            KnowledgeAxis::Memorization | KnowledgeAxis::ContextualMemorization => {
                let exercise =
                    MemorizationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Memorization(Box::new(exercise)))
            }
            KnowledgeAxis::Translation | KnowledgeAxis::Meaning => {
                let exercise =
                    TranslationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Translation(Box::new(exercise)))
            }
            KnowledgeAxis::Tafsir => {
                // For now, treat tafsir like translation
                let exercise =
                    TranslationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Translation(Box::new(exercise)))
            }
            KnowledgeAxis::Tajweed => {
                // Tajweed not implemented yet, fall back to memorization
                let exercise =
                    MemorizationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Memorization(Box::new(exercise)))
            }
        }
    }

    /// Generate an exercise for a specific axis (override node's axis)
    pub async fn generate_exercise_for_axis(
        &self,
        node_id: &str,
        axis: KnowledgeAxis,
    ) -> Result<ExerciseType> {
        match axis {
            KnowledgeAxis::Memorization | KnowledgeAxis::ContextualMemorization => {
                let exercise =
                    MemorizationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Memorization(Box::new(exercise)))
            }
            KnowledgeAxis::Translation | KnowledgeAxis::Meaning | KnowledgeAxis::Tafsir => {
                let exercise =
                    TranslationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Translation(Box::new(exercise)))
            }
            KnowledgeAxis::Tajweed => {
                // Tajweed not implemented yet, fall back to memorization
                let exercise =
                    MemorizationExercise::new(node_id.to_string(), &*self.content_repo).await?;
                Ok(ExerciseType::Memorization(Box::new(exercise)))
            }
        }
    }

    /// Generate an exercise using the modern enum-based architecture (V2)
    ///
    /// This is the next-generation exercise generator that returns lightweight
    /// ExerciseData enums containing only keys/IDs instead of full text.
    /// Flutter can then fetch content based on user preferences (Tajweed, Indopak, etc.)
    ///
    /// # Routing Logic
    /// - WORD nodes: Memorization exercises
    /// - VERSE nodes: Full verse input exercises
    /// - CHAPTER nodes: Ayah Chain exercises
    ///
    /// TODO: Add more sophisticated routing based on:
    /// - Learning state (FSRS energy levels)
    /// - User preferences
    /// - Exercise variety (randomization)
    pub async fn generate_exercise_v2(&self, node_id: &str) -> Result<ExerciseData> {
        // Strip knowledge axis suffix if present
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(node_id) {
            kn.base_node_id
        } else {
            node_id.to_string()
        };

        // Route based on node type prefix
        if base_node_id.starts_with("WORD:") || base_node_id.starts_with("WORD_INSTANCE:") {
            // Word-level exercises
            generators::generate_memorization(base_node_id, &*self.content_repo).await
        } else if base_node_id.starts_with("VERSE:") {
            // Verse-level exercises
            generators::generate_full_verse_input(base_node_id, &*self.content_repo).await
        } else if base_node_id.starts_with("CHAPTER:") {
            // Chapter-level exercises
            generators::generate_ayah_chain(base_node_id, &*self.content_repo).await
        } else {
            Err(anyhow::anyhow!(
                "Cannot determine exercise type for node: {}",
                node_id
            ))
        }
    }

    /// Generate an MCQ exercise (Arabic to English)
    /// Tests translation understanding with multiple choice
    pub async fn generate_mcq_ar_to_en(&self, node_id: &str) -> Result<ExerciseType> {
        let exercise = McqExercise::new_ar_to_en(node_id.to_string(), &*self.content_repo).await?;
        Ok(ExerciseType::McqArToEn(Box::new(exercise)))
    }

    /// Generate an MCQ exercise (English to Arabic)
    /// Tests memorization with multiple choice
    pub async fn generate_mcq_en_to_ar(&self, node_id: &str) -> Result<ExerciseType> {
        let exercise = McqExercise::new_en_to_ar(node_id.to_string(), &*self.content_repo).await?;
        Ok(ExerciseType::McqEnToAr(Box::new(exercise)))
    }

    /// Check an answer for an exercise
    /// Returns ExerciseResponse with semantic grading metadata for Translation/Memorization exercises
    pub fn check_answer(&self, exercise: &dyn Exercise, answer: &str) -> ExerciseResponse {
        let is_correct = exercise.check_answer(answer);

        // Try to downcast to get MCQ options
        let options = (exercise as &dyn std::any::Any)
            .downcast_ref::<McqExercise>()
            .map(|mcq| mcq.get_options().to_vec())
            .or_else(|| {
                (exercise as &dyn std::any::Any)
                    .downcast_ref::<IdentifyRootExercise>()
                    .map(|root_ex| root_ex.get_options().to_vec())
            })
            .or_else(|| {
                (exercise as &dyn std::any::Any)
                    .downcast_ref::<ContextualTranslationExercise>()
                    .map(|ctx_trans| ctx_trans.get_options().to_vec())
            })
            .or_else(|| {
                (exercise as &dyn std::any::Any)
                    .downcast_ref::<CrossVerseConnectionExercise>()
                    .map(|graph_ex| {
                        // Convert (verse_key, text) pairs to just verse_key strings
                        graph_ex
                            .get_options()
                            .iter()
                            .map(|(key, _)| key.clone())
                            .collect()
                    })
            })
            .or_else(|| {
                (exercise as &dyn std::any::Any)
                    .downcast_ref::<AyahSequenceExercise>()
                    .map(|ayah_ex| {
                        // Convert (verse_key, text) pairs to just verse_key strings
                        ayah_ex
                            .get_options()
                            .iter()
                            .map(|(key, _)| key.clone())
                            .collect()
                    })
            });

        // Get semantic grading metadata for TranslationExercise or MemorizationExercise
        // Only if embedder is initialized (fail gracefully if not)
        let (semantic_grade, similarity_score) = if let Some(embedder) = SEMANTIC_EMBEDDER.get() {
            let grader = SemanticGrader::new(embedder);

            if let Some(translation_ex) =
                (exercise as &dyn std::any::Any).downcast_ref::<TranslationExercise>()
            {
                match grader.grade_answer(answer, translation_ex.get_translation()) {
                    Ok(grade) => (
                        Some(grade.label.to_str().to_string()),
                        Some(grade.similarity),
                    ),
                    Err(e) => {
                        tracing::error!("Semantic grading failed for TranslationExercise: {}", e);
                        (None, None)
                    }
                }
            } else if let Some(memorization_ex) =
                (exercise as &dyn std::any::Any).downcast_ref::<MemorizationExercise>()
            {
                // For memorization, grade the normalized Arabic text
                let normalized_answer = MemorizationExercise::normalize_arabic(answer);
                let normalized_correct =
                    MemorizationExercise::normalize_arabic(memorization_ex.get_word_text());

                match grader.grade_answer(&normalized_answer, &normalized_correct) {
                    Ok(grade) => (
                        Some(grade.label.to_str().to_string()),
                        Some(grade.similarity),
                    ),
                    Err(e) => {
                        tracing::error!("Semantic grading failed for MemorizationExercise: {}", e);
                        (None, None)
                    }
                }
            } else {
                (None, None)
            }
        } else {
            tracing::warn!("Semantic embedder not initialized, skipping semantic grading metadata");
            (None, None)
        };

        ExerciseResponse {
            is_correct,
            correct_answer: if !is_correct {
                // Don't reveal correct answer to encourage learning
                None
            } else {
                None
            },
            hint: if !is_correct {
                exercise.get_hint()
            } else {
                None
            },
            options,
            semantic_grade,
            similarity_score,
        }
    }

    /// Get a hint for an exercise
    pub fn get_hint(&self, exercise: &dyn Exercise) -> Option<String> {
        exercise.get_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, NodeType};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock ContentRepository for testing
    struct MockContentRepo {
        quran_text: HashMap<String, String>,
        translations: HashMap<String, String>,
    }

    impl MockContentRepo {
        fn new() -> Self {
            let mut quran_text = HashMap::new();
            let mut translations = HashMap::new();

            // Add test data
            quran_text.insert("WORD:1:1:1".to_string(), "بِسْمِ".to_string());
            translations.insert("WORD:1:1:1".to_string(), "In the name".to_string());
            quran_text.insert("WORD:1".to_string(), "بِسْمِ".to_string());
            translations.insert("WORD:1".to_string(), "In the name".to_string());

            Self {
                quran_text,
                translations,
            }
        }

        fn get_base_id(&self, node_id: &str) -> String {
            if let Some(kn) = KnowledgeNode::parse(node_id) {
                kn.base_node_id
            } else {
                node_id.to_string()
            }
        }
    }

    #[async_trait]
    impl ContentRepository for MockContentRepo {
        async fn get_node(&self, _node_id: &str) -> Result<Option<Node>> {
            Ok(Some(Node {
                id: "test".to_string(),
                node_type: NodeType::Word,
                knowledge_node: None,
            }))
        }

        async fn get_edges_from(&self, _source_id: &str) -> Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_edges_to(&self, _target_id: &str) -> Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_quran_text(&self, node_id: &str) -> Result<Option<String>> {
            let base_id = self.get_base_id(node_id);
            Ok(self.quran_text.get(&base_id).cloned())
        }

        async fn get_translation(&self, node_id: &str, _lang: &str) -> Result<Option<String>> {
            let base_id = self.get_base_id(node_id);
            Ok(self.translations.get(&base_id).cloned())
        }

        async fn get_metadata(&self, _node_id: &str, _key: &str) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_all_metadata(&self, _node_id: &str) -> Result<HashMap<String, String>> {
            Ok(HashMap::new())
        }

        async fn node_exists(&self, _node_id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn get_all_nodes(&self) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_nodes_by_type(&self, _node_type: NodeType) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn insert_nodes_batch(&self, _nodes: &[crate::ImportedNode]) -> Result<()> {
            Ok(())
        }

        async fn insert_edges_batch(&self, _edges: &[crate::ImportedEdge]) -> Result<()> {
            Ok(())
        }

        async fn get_words_in_ayahs(&self, _ayah_node_ids: &[String]) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_adjacent_words(
            &self,
            _word_node_id: &str,
        ) -> Result<(Option<Node>, Option<Node>)> {
            Ok((None, None))
        }

        async fn get_chapter(&self, _chapter_number: i32) -> Result<Option<crate::Chapter>> {
            Ok(None)
        }

        async fn get_chapters(&self) -> Result<Vec<crate::Chapter>> {
            Ok(vec![])
        }

        async fn get_verse(&self, _verse_key: &str) -> Result<Option<crate::Verse>> {
            Ok(None)
        }

        async fn get_verses_for_chapter(&self, _chapter_number: i32) -> Result<Vec<crate::Verse>> {
            Ok(vec![])
        }

        async fn get_words_for_verse(&self, _verse_key: &str) -> Result<Vec<crate::Word>> {
            Ok(vec![])
        }

        async fn get_word(&self, _word_id: i32) -> Result<Option<crate::Word>> {
            Ok(None)
        }

        async fn get_verses_batch(
            &self,
            verse_keys: &[String],
        ) -> Result<HashMap<String, crate::Verse>> {
            let mut result = HashMap::new();
            for key in verse_keys {
                if let Some(verse) = self.get_verse(key).await? {
                    result.insert(key.clone(), verse);
                }
            }
            Ok(result)
        }

        async fn get_words_batch(&self, word_ids: &[i32]) -> Result<HashMap<i32, crate::Word>> {
            let mut result = HashMap::new();
            for &id in word_ids {
                if let Some(word) = self.get_word(id).await? {
                    result.insert(id, word);
                }
            }
            Ok(result)
        }

        async fn get_languages(&self) -> Result<Vec<crate::Language>> {
            Ok(vec![])
        }

        async fn get_language(&self, _code: &str) -> Result<Option<crate::Language>> {
            Ok(None)
        }

        async fn get_translators_for_language(
            &self,
            _language_code: &str,
        ) -> Result<Vec<crate::Translator>> {
            Ok(vec![])
        }

        async fn get_translator(&self, _translator_id: i32) -> Result<Option<crate::Translator>> {
            Ok(None)
        }

        async fn get_translator_by_slug(&self, _slug: &str) -> Result<Option<crate::Translator>> {
            Ok(None)
        }

        async fn get_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
        ) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_word_translation(
            &self,
            _word_id: i32,
            _translator_id: i32,
        ) -> Result<Option<String>> {
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
        ) -> Result<i32> {
            Ok(1)
        }

        async fn insert_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
            _translation: &str,
            _footnotes: Option<&str>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_available_packages(
            &self,
            _package_type: Option<crate::PackageType>,
            _language_code: Option<&str>,
        ) -> Result<Vec<crate::ContentPackage>> {
            Ok(vec![])
        }

        async fn get_package(&self, _package_id: &str) -> Result<Option<crate::ContentPackage>> {
            Ok(None)
        }

        async fn upsert_package(&self, _package: &crate::ContentPackage) -> Result<()> {
            Ok(())
        }

        async fn delete_package(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn get_installed_packages(&self) -> Result<Vec<crate::InstalledPackage>> {
            Ok(vec![])
        }

        async fn is_package_installed(&self, _package_id: &str) -> Result<bool> {
            Ok(false)
        }

        async fn mark_package_installed(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn mark_package_uninstalled(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn enable_package(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn disable_package(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn get_enabled_packages(&self) -> Result<Vec<crate::InstalledPackage>> {
            Ok(vec![])
        }

        async fn get_morphology_for_word(
            &self,
            _word_id: i32,
        ) -> Result<Vec<crate::MorphologySegment>> {
            Ok(vec![])
        }

        async fn get_root_by_id(&self, _root_id: &str) -> Result<Option<crate::Root>> {
            Ok(None)
        }

        async fn get_lemma_by_id(&self, _lemma_id: &str) -> Result<Option<crate::Lemma>> {
            Ok(None)
        }

        async fn get_scheduler_candidates(
            &self,
            _goal_id: &str,
            _user_id: &str,
            _now_ts: i64,
            _user_repo: &dyn crate::ports::user_repository::UserRepository,
        ) -> Result<Vec<crate::scheduler_v2::CandidateNode>> {
            Ok(vec![])
        }

        async fn get_prerequisite_parents(
            &self,
            _node_ids: &[String],
        ) -> Result<HashMap<String, Vec<String>>> {
            Ok(HashMap::new())
        }

        async fn get_goal(
            &self,
            _goal_id: &str,
        ) -> Result<Option<crate::ports::content_repository::SchedulerGoal>> {
            Ok(None)
        }

        async fn get_nodes_for_goal(&self, _goal_id: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_generate_memorization_exercise() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service
            .generate_exercise("WORD:1:1:1:memorization")
            .await
            .unwrap();

        let ex = exercise.as_exercise();
        assert_eq!(ex.get_type_name(), "memorization");
        assert!(ex.generate_question().contains("Recall"));
    }

    #[tokio::test]
    async fn test_generate_translation_exercise() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service
            .generate_exercise("WORD:1:1:1:translation")
            .await
            .unwrap();

        let ex = exercise.as_exercise();
        assert_eq!(ex.get_type_name(), "translation");
        assert!(ex.generate_question().contains("What does"));
    }

    #[tokio::test]
    async fn test_check_answer() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        // Use MCQ exercise (doesn't require semantic model)
        let exercise = service.generate_mcq_ar_to_en("WORD:1:1:1").await.unwrap();

        let ex = exercise.as_exercise();
        // MCQ should accept the correct answer
        let response = service.check_answer(ex, "In the name");

        assert!(response.is_correct);
        assert!(response.hint.is_none()); // No hint for correct answer
    }

    #[tokio::test]
    async fn test_check_wrong_answer_provides_hint() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service
            .generate_exercise("WORD:1:1:1:translation")
            .await
            .unwrap();

        let ex = exercise.as_exercise();
        let response = service.check_answer(ex, "wrong answer");

        assert!(!response.is_correct);
        assert!(response.hint.is_some()); // Hint provided for wrong answer
    }

    #[tokio::test]
    async fn test_generate_mcq_ar_to_en() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_mcq_ar_to_en("WORD:1:1:1").await.unwrap();

        let ex = exercise.as_exercise();
        assert_eq!(ex.get_type_name(), "mcq_ar_to_en");
        assert!(ex.generate_question().contains("What does"));
    }

    #[tokio::test]
    async fn test_generate_mcq_en_to_ar() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_mcq_en_to_ar("WORD:1:1:1").await.unwrap();

        let ex = exercise.as_exercise();
        assert_eq!(ex.get_type_name(), "mcq_en_to_ar");
        assert!(ex.generate_question().contains("Which Arabic word"));
    }

    #[tokio::test]
    async fn test_mcq_check_answer_includes_options() {
        let content_repo = Arc::new(MockContentRepo::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_mcq_ar_to_en("WORD:1:1:1").await.unwrap();

        let ex = exercise.as_exercise();
        let response = service.check_answer(ex, "In the name");

        assert!(response.is_correct);
        assert!(response.options.is_some()); // MCQ should include options
        let options = response.options.unwrap();
        assert_eq!(options.len(), 4); // 1 correct + 3 distractors
        assert!(options.contains(&"In the name".to_string()));
    }

    // ========================================================================
    // V2 Enum-Based Architecture Tests
    // ========================================================================

    /// Enhanced mock repository with more test data for V2 tests
    struct MockContentRepoV2 {
        words: HashMap<i32, crate::Word>,
        verses: HashMap<String, crate::Verse>,
        chapters: HashMap<i32, crate::Chapter>,
    }

    impl MockContentRepoV2 {
        fn new() -> Self {
            let mut words = HashMap::new();
            let mut verses = HashMap::new();
            let mut chapters = HashMap::new();

            // Add test word
            words.insert(
                1,
                crate::Word {
                    id: 1,
                    verse_key: "1:1".to_string(),
                    position: 1,
                    text_uthmani: "بِسْمِ".to_string(),
                    text_simple: None,
                    transliteration: None,
                },
            );

            // Add test verse
            verses.insert(
                "1:1".to_string(),
                crate::Verse {
                    key: "1:1".to_string(),
                    chapter_number: 1,
                    verse_number: 1,
                    text_uthmani: "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
            );

            // Add test chapter
            chapters.insert(
                1,
                crate::Chapter {
                    number: 1,
                    name_arabic: "الفاتحة".to_string(),
                    name_transliteration: "Al-Fatihah".to_string(),
                    name_translation: "The Opening".to_string(),
                    revelation_place: Some("makkah".to_string()),
                    verse_count: 7,
                },
            );

            Self {
                words,
                verses,
                chapters,
            }
        }
    }

    #[async_trait]
    impl ContentRepository for MockContentRepoV2 {
        async fn get_node(&self, _node_id: &str) -> Result<Option<Node>> {
            Ok(None)
        }

        async fn get_edges_from(&self, _source_id: &str) -> Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_edges_to(&self, _target_id: &str) -> Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_quran_text(&self, _node_id: &str) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_translation(&self, _node_id: &str, _lang: &str) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_metadata(&self, _node_id: &str, _key: &str) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_all_metadata(&self, _node_id: &str) -> Result<HashMap<String, String>> {
            Ok(HashMap::new())
        }

        async fn node_exists(&self, _node_id: &str) -> Result<bool> {
            Ok(false)
        }

        async fn get_all_nodes(&self) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_nodes_by_type(&self, _node_type: NodeType) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn insert_nodes_batch(&self, _nodes: &[crate::ImportedNode]) -> Result<()> {
            Ok(())
        }

        async fn insert_edges_batch(&self, _edges: &[crate::ImportedEdge]) -> Result<()> {
            Ok(())
        }

        async fn get_words_in_ayahs(&self, _ayah_node_ids: &[String]) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_adjacent_words(
            &self,
            _word_node_id: &str,
        ) -> Result<(Option<Node>, Option<Node>)> {
            Ok((None, None))
        }

        async fn get_chapter(&self, chapter_number: i32) -> Result<Option<crate::Chapter>> {
            Ok(self.chapters.get(&chapter_number).cloned())
        }

        async fn get_chapters(&self) -> Result<Vec<crate::Chapter>> {
            Ok(self.chapters.values().cloned().collect())
        }

        async fn get_verse(&self, verse_key: &str) -> Result<Option<crate::Verse>> {
            Ok(self.verses.get(verse_key).cloned())
        }

        async fn get_verses_for_chapter(&self, chapter_number: i32) -> Result<Vec<crate::Verse>> {
            Ok(self
                .verses
                .values()
                .filter(|v| v.chapter_number == chapter_number)
                .cloned()
                .collect())
        }

        async fn get_words_for_verse(&self, _verse_key: &str) -> Result<Vec<crate::Word>> {
            Ok(vec![])
        }

        async fn get_word(&self, word_id: i32) -> Result<Option<crate::Word>> {
            Ok(self.words.get(&word_id).cloned())
        }

        async fn get_verses_batch(
            &self,
            verse_keys: &[String],
        ) -> Result<HashMap<String, crate::Verse>> {
            let mut result = HashMap::new();
            for key in verse_keys {
                if let Some(verse) = self.get_verse(key).await? {
                    result.insert(key.clone(), verse);
                }
            }
            Ok(result)
        }

        async fn get_words_batch(&self, word_ids: &[i32]) -> Result<HashMap<i32, crate::Word>> {
            let mut result = HashMap::new();
            for &id in word_ids {
                if let Some(word) = self.get_word(id).await? {
                    result.insert(id, word);
                }
            }
            Ok(result)
        }

        async fn get_languages(&self) -> Result<Vec<crate::Language>> {
            Ok(vec![])
        }

        async fn get_language(&self, _code: &str) -> Result<Option<crate::Language>> {
            Ok(None)
        }

        async fn get_translators_for_language(
            &self,
            _language_code: &str,
        ) -> Result<Vec<crate::Translator>> {
            Ok(vec![])
        }

        async fn get_translator(&self, _translator_id: i32) -> Result<Option<crate::Translator>> {
            Ok(None)
        }

        async fn get_translator_by_slug(&self, _slug: &str) -> Result<Option<crate::Translator>> {
            Ok(None)
        }

        async fn get_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
        ) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_word_translation(
            &self,
            _word_id: i32,
            _translator_id: i32,
        ) -> Result<Option<String>> {
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
        ) -> Result<i32> {
            Ok(1)
        }

        async fn insert_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
            _translation: &str,
            _footnotes: Option<&str>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_available_packages(
            &self,
            _package_type: Option<crate::PackageType>,
            _language_code: Option<&str>,
        ) -> Result<Vec<crate::ContentPackage>> {
            Ok(vec![])
        }

        async fn get_package(&self, _package_id: &str) -> Result<Option<crate::ContentPackage>> {
            Ok(None)
        }

        async fn upsert_package(&self, _package: &crate::ContentPackage) -> Result<()> {
            Ok(())
        }

        async fn delete_package(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn get_installed_packages(&self) -> Result<Vec<crate::InstalledPackage>> {
            Ok(vec![])
        }

        async fn is_package_installed(&self, _package_id: &str) -> Result<bool> {
            Ok(false)
        }

        async fn mark_package_installed(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn mark_package_uninstalled(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn enable_package(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn disable_package(&self, _package_id: &str) -> Result<()> {
            Ok(())
        }

        async fn get_enabled_packages(&self) -> Result<Vec<crate::InstalledPackage>> {
            Ok(vec![])
        }

        async fn get_morphology_for_word(
            &self,
            _word_id: i32,
        ) -> Result<Vec<crate::MorphologySegment>> {
            Ok(vec![])
        }

        async fn get_root_by_id(&self, _root_id: &str) -> Result<Option<crate::Root>> {
            Ok(None)
        }

        async fn get_lemma_by_id(&self, _lemma_id: &str) -> Result<Option<crate::Lemma>> {
            Ok(None)
        }

        async fn get_scheduler_candidates(
            &self,
            _goal_id: &str,
            _user_id: &str,
            _now_ts: i64,
            _user_repo: &dyn crate::ports::user_repository::UserRepository,
        ) -> Result<Vec<crate::scheduler_v2::CandidateNode>> {
            Ok(vec![])
        }

        async fn get_prerequisite_parents(
            &self,
            _node_ids: &[String],
        ) -> Result<HashMap<String, Vec<String>>> {
            Ok(HashMap::new())
        }

        async fn get_goal(
            &self,
            _goal_id: &str,
        ) -> Result<Option<crate::ports::content_repository::SchedulerGoal>> {
            Ok(None)
        }

        async fn get_nodes_for_goal(&self, _goal_id: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_generate_exercise_v2_word_node() {
        let content_repo = Arc::new(MockContentRepoV2::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_exercise_v2("WORD:1").await.unwrap();

        // Should generate Memorization exercise
        assert_eq!(exercise.type_name(), "memorization");
        assert_eq!(exercise.node_id(), "WORD:1");
    }

    #[tokio::test]
    async fn test_generate_exercise_v2_verse_node() {
        let content_repo = Arc::new(MockContentRepoV2::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_exercise_v2("VERSE:1:1").await.unwrap();

        // Should generate FullVerseInput exercise
        assert_eq!(exercise.type_name(), "full_verse_input");
        assert_eq!(exercise.node_id(), "VERSE:1:1");
    }

    #[tokio::test]
    async fn test_generate_exercise_v2_chapter_node() {
        let content_repo = Arc::new(MockContentRepoV2::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_exercise_v2("CHAPTER:1").await.unwrap();

        // Should generate AyahChain exercise
        assert_eq!(exercise.type_name(), "ayah_chain");
        assert_eq!(exercise.node_id(), "CHAPTER:1");
    }

    #[tokio::test]
    async fn test_generate_exercise_v2_with_knowledge_axis() {
        let content_repo = Arc::new(MockContentRepoV2::new());
        let service = ExerciseService::new(content_repo);

        // Test with knowledge axis suffix (should strip it)
        let exercise = service
            .generate_exercise_v2("WORD:1:memorization")
            .await
            .unwrap();

        assert_eq!(exercise.type_name(), "memorization");
        // node_id should be the base without axis
        assert_eq!(exercise.node_id(), "WORD:1");
    }

    #[tokio::test]
    async fn test_generate_exercise_v2_invalid_node() {
        let content_repo = Arc::new(MockContentRepoV2::new());
        let service = ExerciseService::new(content_repo);

        // Should return error for unknown node type
        let result = service.generate_exercise_v2("INVALID:123").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot determine exercise type"));
    }

    #[tokio::test]
    async fn test_generate_exercise_v2_serialization() {
        let content_repo = Arc::new(MockContentRepoV2::new());
        let service = ExerciseService::new(content_repo);

        let exercise = service.generate_exercise_v2("WORD:1").await.unwrap();

        // Should be serializable to JSON
        let json = serde_json::to_string(&exercise).unwrap();
        assert!(json.contains(r#""type":"memorization"#));

        // Should be deserializable
        let deserialized: ExerciseData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "memorization");
    }
}
