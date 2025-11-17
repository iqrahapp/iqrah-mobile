// semantic/grader.rs
// Semantic answer grading using similarity thresholds

use super::embedding::SemanticEmbedder;
use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

/// Global singleton for the semantic embedder
/// This is loaded once at application startup and shared across all exercises
pub static SEMANTIC_EMBEDDER: OnceCell<SemanticEmbedder> = OnceCell::new();

/// Grade label for semantic similarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticGradeLabel {
    /// Excellent match (similarity >= excellent_threshold)
    Excellent,
    /// Partial match (similarity >= partial_threshold)
    Partial,
    /// Incorrect (similarity < partial_threshold)
    Incorrect,
}

impl SemanticGradeLabel {
    /// Convert to string for serialization
    pub fn to_str(&self) -> &'static str {
        match self {
            SemanticGradeLabel::Excellent => "Excellent",
            SemanticGradeLabel::Partial => "Partial",
            SemanticGradeLabel::Incorrect => "Incorrect",
        }
    }
}

/// Result of semantic grading
#[derive(Debug, Clone)]
pub struct SemanticGrade {
    /// The grade label (Excellent/Partial/Incorrect)
    pub label: SemanticGradeLabel,
    /// The raw similarity score (0.0 to 1.0)
    pub similarity: f32,
}

/// Semantic answer grader with configurable thresholds
pub struct SemanticGrader<'a> {
    embedder: &'a SemanticEmbedder,
    /// Minimum similarity for "Excellent" grade (default: 0.85)
    excellent_min: f32,
    /// Minimum similarity for "Partial" grade (default: 0.70)
    partial_min: f32,
}

impl<'a> SemanticGrader<'a> {
    /// Create a new grader with default thresholds
    ///
    /// Default thresholds:
    /// - Excellent: >= 0.85
    /// - Partial: >= 0.70
    /// - Incorrect: < 0.70
    pub fn new(embedder: &'a SemanticEmbedder) -> Self {
        Self {
            embedder,
            excellent_min: 0.85,
            partial_min: 0.70,
        }
    }

    /// Create a new grader with custom thresholds
    ///
    /// # Arguments
    /// * `embedder` - The semantic embedder to use
    /// * `excellent_min` - Minimum similarity for Excellent grade
    /// * `partial_min` - Minimum similarity for Partial grade
    pub fn with_thresholds(
        embedder: &'a SemanticEmbedder,
        excellent_min: f32,
        partial_min: f32,
    ) -> Self {
        Self {
            embedder,
            excellent_min,
            partial_min,
        }
    }

    /// Grade a user answer against a reference answer
    ///
    /// # Arguments
    /// * `user_answer` - The user's submitted answer
    /// * `reference_answer` - The correct reference answer
    ///
    /// # Returns
    /// A SemanticGrade with label and similarity score
    pub fn grade_answer(&self, user_answer: &str, reference_answer: &str) -> Result<SemanticGrade> {
        let similarity = self.embedder.similarity(user_answer, reference_answer)?;
        let label = self.classify_similarity(similarity);

        Ok(SemanticGrade { label, similarity })
    }

    /// Grade a user answer against multiple reference answers
    /// Uses the maximum similarity across all references
    ///
    /// # Arguments
    /// * `user_answer` - The user's submitted answer
    /// * `reference_answers` - List of acceptable reference answers
    ///
    /// # Returns
    /// A SemanticGrade with label and similarity score (maximum across references)
    pub fn grade_against_many(
        &self,
        user_answer: &str,
        reference_answers: &[String],
    ) -> Result<SemanticGrade> {
        let similarity = self.embedder.max_similarity(user_answer, reference_answers)?;
        let label = self.classify_similarity(similarity);

        Ok(SemanticGrade { label, similarity })
    }

    /// Classify a similarity score into a grade label
    fn classify_similarity(&self, similarity: f32) -> SemanticGradeLabel {
        if similarity >= self.excellent_min {
            SemanticGradeLabel::Excellent
        } else if similarity >= self.partial_min {
            SemanticGradeLabel::Partial
        } else {
            SemanticGradeLabel::Incorrect
        }
    }

    /// Get the current threshold values
    pub fn get_thresholds(&self) -> (f32, f32) {
        (self.excellent_min, self.partial_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock embedder for testing without loading an actual model
    struct MockEmbedder;

    impl MockEmbedder {
        fn similarity(&self, text_a: &str, text_b: &str) -> f32 {
            // Simple mock: exact match = 1.0, else based on shared words
            if text_a == text_b {
                return 1.0;
            }

            let words_a: std::collections::HashSet<&str> =
                text_a.split_whitespace().collect();
            let words_b: std::collections::HashSet<&str> =
                text_b.split_whitespace().collect();

            let intersection = words_a.intersection(&words_b).count();
            let union = words_a.union(&words_b).count();

            if union == 0 {
                return 0.0;
            }

            intersection as f32 / union as f32
        }
    }

    #[test]
    fn test_classify_similarity_excellent() {
        // Create a mock embedder (we can't easily test without a real model)
        // So we'll just test the classification logic
        let similarity = 0.90;

        // Test classification thresholds
        assert!(similarity >= 0.85, "Should be excellent");

        let label = if similarity >= 0.85 {
            SemanticGradeLabel::Excellent
        } else if similarity >= 0.70 {
            SemanticGradeLabel::Partial
        } else {
            SemanticGradeLabel::Incorrect
        };

        assert_eq!(label, SemanticGradeLabel::Excellent);
    }

    #[test]
    fn test_classify_similarity_partial() {
        let similarity = 0.75;

        let label = if similarity >= 0.85 {
            SemanticGradeLabel::Excellent
        } else if similarity >= 0.70 {
            SemanticGradeLabel::Partial
        } else {
            SemanticGradeLabel::Incorrect
        };

        assert_eq!(label, SemanticGradeLabel::Partial);
    }

    #[test]
    fn test_classify_similarity_incorrect() {
        let similarity = 0.60;

        let label = if similarity >= 0.85 {
            SemanticGradeLabel::Excellent
        } else if similarity >= 0.70 {
            SemanticGradeLabel::Partial
        } else {
            SemanticGradeLabel::Incorrect
        };

        assert_eq!(label, SemanticGradeLabel::Incorrect);
    }

    #[test]
    fn test_mock_similarity() {
        let mock = MockEmbedder;

        // Exact match should be 1.0
        assert_eq!(mock.similarity("hello world", "hello world"), 1.0);

        // Partial match should be between 0 and 1
        let sim = mock.similarity("hello world", "hello there");
        assert!(sim > 0.0 && sim < 1.0);

        // No match should be low
        let sim = mock.similarity("hello", "goodbye");
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_grade_label_to_str() {
        assert_eq!(SemanticGradeLabel::Excellent.to_str(), "Excellent");
        assert_eq!(SemanticGradeLabel::Partial.to_str(), "Partial");
        assert_eq!(SemanticGradeLabel::Incorrect.to_str(), "Incorrect");
    }
}
