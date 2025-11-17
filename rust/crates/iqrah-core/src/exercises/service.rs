// exercises/service.rs
// Exercise service for generating axis-specific exercises

use super::mcq::McqExercise;
use super::memorization::MemorizationExercise;
use super::translation::TranslationExercise;
use super::types::{Exercise, ExerciseResponse, ExerciseType};
use crate::semantic::grader::{SemanticGrader, SEMANTIC_EMBEDDER};
use crate::semantic::SemanticEmbedder;
use crate::{ContentRepository, KnowledgeAxis, KnowledgeNode};
use anyhow::Result;
use std::sync::Arc;

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
    /// // From Flutter, after getting app documents directory:
    /// let cache_dir = "/data/user/0/com.example.app/files/huggingface";
    /// ExerciseService::init_semantic_model(
    ///     "minishlab/potion-multilingual-128M",
    ///     Some(cache_dir)
    /// )?;
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

        // Try to downcast to McqExercise to get options
        let options = (exercise as &dyn std::any::Any)
            .downcast_ref::<McqExercise>()
            .map(|mcq| mcq.get_options().to_vec());

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

            Self {
                quran_text,
                translations,
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

        async fn get_quran_text(&self, node_id: &str) -> Result<Option<String>> {
            Ok(self.quran_text.get(node_id).cloned())
        }

        async fn get_translation(&self, node_id: &str, _lang: &str) -> Result<Option<String>> {
            Ok(self.translations.get(node_id).cloned())
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

        let exercise = service
            .generate_exercise("WORD:1:1:1:translation")
            .await
            .unwrap();

        let ex = exercise.as_exercise();
        let response = service.check_answer(ex, "in the name");

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
}
