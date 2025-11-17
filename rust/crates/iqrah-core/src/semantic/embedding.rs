// semantic/embedding.rs
// Wrapper for model2vec BGE-M3 embeddings
//
// ## Model Caching Behavior
//
// model2vec-rs automatically caches downloaded models:
// - **Download Once**: Models are downloaded to HuggingFace cache (~/.cache/huggingface/)
// - **Reuse Cached**: Subsequent calls use cached files (no re-download)
// - **Load Once**: This embedder is wrapped in OnceCell (see grader.rs)
//   - Loaded only ONCE at app startup
//   - Shared across all exercises
//   - Stays in RAM for the application lifetime
// - **No Re-loading**: The model is never reloaded unless the app restarts
//
// Performance:
// - First download: ~30-60 seconds (downloads model files)
// - Subsequent startups: ~1-3 seconds (loads from cache)
// - Per-exercise cost: ~0ms (singleton already in RAM)

use anyhow::{Context, Result};
use model2vec_rs::model::StaticModel;
use std::sync::Arc;

/// Semantic embedder using model2vec with BGE-M3
/// Provides text embedding and similarity computation
///
/// This embedder is designed to be loaded once and shared across all exercises.
/// See SEMANTIC_EMBEDDER in grader.rs for the singleton instance.
pub struct SemanticEmbedder {
    model: Arc<StaticModel>,
}

impl SemanticEmbedder {
    /// Create a new embedder by loading a model from the specified path or Hugging Face repo
    ///
    /// # Arguments
    /// * `model_path` - Path to local model or Hugging Face model ID (e.g., "minishlab/potion-base-8M")
    ///
    /// # Caching
    /// - HuggingFace models are cached in `~/.cache/huggingface/hub/`
    /// - Only downloaded once (first call)
    /// - Subsequent calls load from cache (much faster)
    ///
    /// # Returns
    /// A new SemanticEmbedder instance, or an error if the model fails to load
    pub fn new(model_path: &str) -> Result<Self> {
        tracing::info!("Loading semantic model from: {}", model_path);
        tracing::info!("Note: First download may take 30-60 seconds. Subsequent loads are fast (<3s).");

        // Load model from path (local or HuggingFace)
        // model2vec-rs handles caching automatically
        let model = StaticModel::from_pretrained(model_path, None, None, None)
            .context("Failed to load semantic model")?;

        tracing::info!("✅ Semantic model loaded successfully and ready for inference");

        Ok(Self {
            model: Arc::new(model),
        })
    }

    /// Embed a single text string
    ///
    /// # Arguments
    /// * `text` - The text to embed
    ///
    /// # Returns
    /// A vector of floats representing the embedding
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let sentences = vec![text.to_string()];
        let embeddings = self.model.encode(&sentences);

        // Get the first (and only) embedding
        embeddings
            .into_iter()
            .next()
            .context("Failed to generate embedding")
    }

    /// Compute cosine similarity between two text strings
    ///
    /// # Arguments
    /// * `text_a` - First text
    /// * `text_b` - Second text
    ///
    /// # Returns
    /// A similarity score between -1.0 and 1.0 (typically 0.0 to 1.0 for meaningful comparisons)
    pub fn similarity(&self, text_a: &str, text_b: &str) -> Result<f32> {
        // Batch encode both texts for efficiency
        let sentences = vec![text_a.to_string(), text_b.to_string()];
        let embeddings = self.model.encode(&sentences);

        if embeddings.len() != 2 {
            anyhow::bail!("Expected 2 embeddings, got {}", embeddings.len());
        }

        let similarity = cosine_similarity(&embeddings[0], &embeddings[1]);
        Ok(similarity)
    }

    /// Compute similarities between a user answer and multiple reference answers
    /// Returns the maximum similarity score
    ///
    /// # Arguments
    /// * `user_answer` - The user's answer
    /// * `reference_answers` - List of acceptable reference answers
    ///
    /// # Returns
    /// The maximum similarity score across all reference answers
    pub fn max_similarity(&self, user_answer: &str, reference_answers: &[String]) -> Result<f32> {
        if reference_answers.is_empty() {
            return Ok(0.0);
        }

        // Batch encode user answer + all reference answers for efficiency
        let mut sentences = vec![user_answer.to_string()];
        sentences.extend(reference_answers.iter().cloned());

        let embeddings = self.model.encode(&sentences);

        if embeddings.is_empty() {
            anyhow::bail!("No embeddings generated");
        }

        // First embedding is the user answer
        let user_emb = &embeddings[0];

        // Compute similarity with each reference and find max
        let mut max_sim = f32::MIN;
        for ref_emb in &embeddings[1..] {
            let sim = cosine_similarity(user_emb, ref_emb);
            max_sim = max_sim.max(sim);
        }

        Ok(max_sim)
    }
}

/// Compute cosine similarity between two embedding vectors
///
/// Cosine similarity = (A · B) / (||A|| × ||B||)
/// Returns a value between -1.0 and 1.0
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        tracing::error!("Embedding dimension mismatch: {} vs {}", a.len(), b.len());
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim - 1.0).abs() < 0.0001, "Identical vectors should have similarity ~1.0");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!(sim.abs() < 0.0001, "Orthogonal vectors should have similarity ~0.0");
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim + 1.0).abs() < 0.0001, "Opposite vectors should have similarity ~-1.0");
    }

    #[test]
    fn test_cosine_similarity_dimension_mismatch() {
        let vec1 = vec![1.0, 2.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert_eq!(sim, 0.0, "Mismatched dimensions should return 0.0");
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let vec1 = vec![0.0, 0.0, 0.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert_eq!(sim, 0.0, "Zero vector should return 0.0");
    }
}
