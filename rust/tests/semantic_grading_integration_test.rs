// Integration tests for semantic grading with real model
//
// Run these tests with:
// ```
// cargo test --test semantic_grading_integration_test
// ```
//
// These tests will download the model on first run (~30-60 seconds).
// Subsequent runs will use the cached model and be much faster (<3 seconds).

use iqrah_core::{ExerciseService, SemanticEmbedder, SemanticGrader, SemanticGradeLabel};

/// Test model initialization and basic semantic grading
#[test]
fn test_semantic_model_initialization() {
    // Use a small, fast model for testing
    let model_path = "minishlab/potion-base-8M";

    println!("Loading model: {}", model_path);
    println!("Note: First run may take 30-60 seconds to download model");

    let result = ExerciseService::init_semantic_model(model_path);

    match result {
        Ok(_) => {
            println!("✅ Model initialized successfully");
        }
        Err(e) => {
            panic!(
                "❌ Failed to initialize model: {}\n\
                 This test requires internet connection for first run.\n\
                 Subsequent runs will use cached model.",
                e
            );
        }
    }
}

/// Test semantic grading with English text (Translation exercises)
#[test]
fn test_semantic_grading_english() {
    let model_path = "minishlab/potion-base-8M";

    // Initialize model (will use cache if already downloaded)
    ExerciseService::init_semantic_model(model_path)
        .expect("Failed to initialize model - run test_semantic_model_initialization first");

    let embedder = iqrah_core::SEMANTIC_EMBEDDER
        .get()
        .expect("Embedder should be initialized");

    let grader = SemanticGrader::new(embedder);

    // Test 1: High similarity - synonyms
    println!("\nTest 1: High similarity (synonyms)");
    let grade = grader
        .grade_answer("In the name of God", "By the name of Allah")
        .expect("Grading failed");
    println!(
        "  Answer: 'In the name of God' vs 'By the name of Allah'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    assert!(
        grade.similarity > 0.60,
        "Similar phrases should have >0.60 similarity, got {:.3}",
        grade.similarity
    );

    // Test 2: Medium similarity - related but different
    println!("\nTest 2: Medium similarity (related)");
    let grade = grader
        .grade_answer("The merciful", "The compassionate")
        .expect("Grading failed");
    println!("  Answer: 'The merciful' vs 'The compassionate'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    assert!(
        grade.similarity > 0.50,
        "Related words should have >0.50 similarity, got {:.3}",
        grade.similarity
    );

    // Test 3: Low similarity - completely different
    println!("\nTest 3: Low similarity (different meanings)");
    let grade = grader
        .grade_answer("The sky", "In the name")
        .expect("Grading failed");
    println!("  Answer: 'The sky' vs 'In the name'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    assert!(
        grade.similarity < 0.60,
        "Different meanings should have <0.60 similarity, got {:.3}",
        grade.similarity
    );

    // Test 4: Exact match
    println!("\nTest 4: Exact match");
    let grade = grader
        .grade_answer("In the name", "In the name")
        .expect("Grading failed");
    println!("  Answer: 'In the name' vs 'In the name'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    assert!(
        grade.similarity > 0.90,
        "Exact match should have >0.90 similarity, got {:.3}",
        grade.similarity
    );
    assert_eq!(
        grade.label,
        SemanticGradeLabel::Excellent,
        "Exact match should get Excellent grade"
    );

    println!("\n✅ All English semantic grading tests passed");
}

/// Test semantic grading with Arabic text (Memorization exercises)
#[test]
fn test_semantic_grading_arabic() {
    let model_path = "minishlab/potion-base-8M";

    // Initialize model
    ExerciseService::init_semantic_model(model_path)
        .expect("Failed to initialize model");

    let embedder = iqrah_core::SEMANTIC_EMBEDDER
        .get()
        .expect("Embedder should be initialized");

    let grader = SemanticGrader::new(embedder);

    // Test 1: Exact Arabic match
    println!("\nTest 1: Exact Arabic match");
    let grade = grader.grade_answer("بسم", "بسم").expect("Grading failed");
    println!("  Answer: 'بسم' vs 'بسم'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    assert!(
        grade.similarity > 0.85,
        "Exact Arabic match should have >0.85 similarity, got {:.3}",
        grade.similarity
    );

    // Test 2: Similar Arabic words
    println!("\nTest 2: Similar Arabic words");
    let grade = grader.grade_answer("الرحمن", "الرحيم").expect("Grading failed");
    println!("  Answer: 'الرحمن' vs 'الرحيم'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    // These are related (both mean merciful/compassionate) so should have decent similarity
    assert!(
        grade.similarity > 0.40,
        "Related Arabic words should have >0.40 similarity, got {:.3}",
        grade.similarity
    );

    // Test 3: Different Arabic words
    println!("\nTest 3: Different Arabic words");
    let grade = grader.grade_answer("الله", "كتاب").expect("Grading failed");
    println!("  Answer: 'الله' vs 'كتاب'");
    println!("  Grade: {:?}, Score: {:.3}", grade.label, grade.similarity);
    // These are completely different so should have low similarity
    assert!(
        grade.similarity < 0.70,
        "Different Arabic words should have <0.70 similarity, got {:.3}",
        grade.similarity
    );

    println!("\n✅ All Arabic semantic grading tests passed");
}

/// Test threshold boundaries
#[test]
fn test_grading_thresholds() {
    let model_path = "minishlab/potion-base-8M";

    ExerciseService::init_semantic_model(model_path)
        .expect("Failed to initialize model");

    let embedder = iqrah_core::SEMANTIC_EMBEDDER
        .get()
        .expect("Embedder should be initialized");

    let grader = SemanticGrader::new(embedder);

    println!("\nTesting grade label thresholds:");

    // We can't predict exact similarity scores, but we can verify:
    // 1. Exact matches get Excellent
    // 2. Very different text gets Incorrect
    // 3. Labels are assigned correctly based on thresholds

    // Exact match should get Excellent
    let grade = grader
        .grade_answer("test phrase", "test phrase")
        .expect("Grading failed");
    println!("  Exact match: Score={:.3}, Label={:?}", grade.similarity, grade.label);
    assert_eq!(
        grade.label,
        SemanticGradeLabel::Excellent,
        "Exact match should be Excellent"
    );

    // Completely different should get Incorrect
    let grade = grader
        .grade_answer("apple", "elephant")
        .expect("Grading failed");
    println!(
        "  Different words: Score={:.3}, Label={:?}",
        grade.similarity, grade.label
    );
    assert_eq!(
        grade.label,
        SemanticGradeLabel::Incorrect,
        "Completely different words should be Incorrect"
    );

    println!("\n✅ Threshold tests passed");
}

/// Test batch processing
#[test]
fn test_batch_similarity() {
    let model_path = "minishlab/potion-base-8M";

    ExerciseService::init_semantic_model(model_path)
        .expect("Failed to initialize model");

    let embedder = iqrah_core::SEMANTIC_EMBEDDER
        .get()
        .expect("Embedder should be initialized");

    // Test multiple reference answers
    let references = vec![
        "In the name".to_string(),
        "By the name".to_string(),
        "With the name".to_string(),
    ];

    let similarity = embedder
        .max_similarity("In the name", &references)
        .expect("Max similarity failed");

    println!("\nTest max_similarity:");
    println!("  User answer: 'In the name'");
    println!("  References: {:?}", references);
    println!("  Max similarity: {:.3}", similarity);

    assert!(
        similarity > 0.85,
        "Should find high similarity with at least one reference, got {:.3}",
        similarity
    );

    println!("\n✅ Batch similarity test passed");
}

/// Performance benchmark
#[test]
fn test_inference_performance() {
    use std::time::Instant;

    let model_path = "minishlab/potion-base-8M";

    ExerciseService::init_semantic_model(model_path)
        .expect("Failed to initialize model");

    let embedder = iqrah_core::SEMANTIC_EMBEDDER
        .get()
        .expect("Embedder should be initialized");

    let grader = SemanticGrader::new(embedder);

    println!("\nPerformance benchmark:");

    // Warm up
    for _ in 0..3 {
        let _ = grader.grade_answer("test", "test");
    }

    // Benchmark 10 gradings
    let iterations = 10;
    let start = Instant::now();

    for i in 0..iterations {
        let answer = format!("test phrase {}", i);
        let reference = "test phrase";
        let _ = grader.grade_answer(&answer, reference).expect("Grading failed");
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    println!("  Total time for {} iterations: {:?}", iterations, elapsed);
    println!("  Average time per grading: {:.2}ms", avg_ms);

    // Target: < 100ms per grading
    assert!(
        avg_ms < 100.0,
        "Average grading time should be <100ms, got {:.2}ms",
        avg_ms
    );

    println!("\n✅ Performance test passed (avg: {:.2}ms per grading)", avg_ms);
}
