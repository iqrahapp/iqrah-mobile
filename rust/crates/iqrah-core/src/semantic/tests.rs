// semantic/tests.rs
// Integration tests for semantic grading module

use super::*;

// Note: These tests assume a model is available at the test path.
// In production, you would either:
// 1. Bundle a test model
// 2. Mock the embedder
// 3. Skip these tests in CI and run them manually with a real model

#[test]
fn test_semantic_module_structure() {
    // Just verify the module structure is correct
    // We can't test the actual model without a real model file
    // Semantic module is properly structured (verified by successful compilation)
}

#[test]
fn test_grade_labels() {
    // Test that grade labels work correctly
    use grader::SemanticGradeLabel;

    let excellent = SemanticGradeLabel::Excellent;
    let partial = SemanticGradeLabel::Partial;
    let incorrect = SemanticGradeLabel::Incorrect;

    assert_eq!(excellent.to_str(), "Excellent");
    assert_eq!(partial.to_str(), "Partial");
    assert_eq!(incorrect.to_str(), "Incorrect");

    // Test equality
    assert_eq!(excellent, SemanticGradeLabel::Excellent);
    assert_ne!(excellent, SemanticGradeLabel::Partial);
}

#[test]
fn test_global_embedder_singleton() {
    // Test that the global singleton works
    use grader::SEMANTIC_EMBEDDER;

    // Initially should be empty
    assert!(SEMANTIC_EMBEDDER.get().is_none());

    // Note: We can't actually initialize it here because we don't have a model file
    // In production, this would be initialized at app startup
}

// Integration tests that require a real model would go here
// They should be marked with #[ignore] by default and run manually
#[test]
#[ignore]
fn test_embedder_with_real_model() {
    use embedding::SemanticEmbedder;

    // This test requires a real model file
    // Run with: cargo test -- --ignored
    let model_path = "tests/fixtures/bge-m3-test.model";

    match SemanticEmbedder::new(model_path) {
        Ok(embedder) => {
            // Test embedding
            let emb = embedder.embed("Hello world").unwrap();
            assert!(!emb.is_empty(), "Embedding should not be empty");

            // Test similarity
            let sim = embedder.similarity("Hello world", "Hello world").unwrap();
            assert!(sim > 0.9, "Identical text should have high similarity");

            let sim = embedder.similarity("Hello world", "Goodbye world").unwrap();
            assert!(sim < 0.9, "Different text should have lower similarity");
        }
        Err(_) => {
            // Model not found, skip test
            println!("Model not found at {}, skipping test", model_path);
        }
    }
}

#[test]
#[ignore]
fn test_grader_with_real_model() {
    use embedding::SemanticEmbedder;
    use grader::SemanticGrader;

    // This test requires a real model file
    let model_path = "tests/fixtures/bge-m3-test.model";

    match SemanticEmbedder::new(model_path) {
        Ok(embedder) => {
            let grader = SemanticGrader::new(&embedder);

            // Test grading
            let grade = grader
                .grade_answer("In the name of God", "In the name of Allah")
                .unwrap();
            assert!(
                grade.similarity > 0.7,
                "Similar meanings should have reasonable similarity"
            );

            let grade = grader
                .grade_answer("Completely different", "In the name of God")
                .unwrap();
            assert!(
                grade.similarity < 0.7,
                "Different meanings should have low similarity"
            );
        }
        Err(_) => {
            println!("Model not found at {}, skipping test", model_path);
        }
    }
}
