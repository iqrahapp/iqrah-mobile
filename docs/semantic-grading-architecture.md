# Semantic Answer Grading Architecture

## Overview

The Iqrah app now supports **semantic similarity grading** for `TranslationExercise` using `model2vec-rs` with BGE-M3 multilingual embeddings. This upgrade replaces simple fuzzy matching with true semantic understanding, enabling more accurate grading of translation exercises.

## Architecture

### Components

#### 1. Semantic Module (`rust/crates/iqrah-core/src/semantic/`)

**Location:** `iqrah-core/src/semantic/`

**Files:**
- `mod.rs` - Module exports and public API
- `embedding.rs` - Model wrapper for text embeddings
- `grader.rs` - Grading logic with configurable thresholds
- `tests.rs` - Unit tests

**Key Types:**

```rust
// Wrapper for model2vec StaticModel
pub struct SemanticEmbedder {
    model: Arc<StaticModel>,
}

// Grading with configurable thresholds
pub struct SemanticGrader<'a> {
    embedder: &'a SemanticEmbedder,
    excellent_min: f32,  // Default: 0.85
    partial_min: f32,    // Default: 0.70
}

// Grade labels
pub enum SemanticGradeLabel {
    Excellent,  // ≥ 0.85 similarity
    Partial,    // ≥ 0.70 similarity
    Incorrect,  // < 0.70 similarity
}

// Grade result
pub struct SemanticGrade {
    pub label: SemanticGradeLabel,
    pub similarity: f32,
}
```

**Global Singleton:**

```rust
pub static SEMANTIC_EMBEDDER: OnceCell<SemanticEmbedder> = OnceCell::new();
```

The embedder is loaded once at app startup and shared across all exercises.

#### 2. TranslationExercise Integration

**Location:** `rust/crates/iqrah-core/src/exercises/translation.rs`

**Changes:**
- `check_answer()` now uses **semantic grading only**
- Returns `false` if semantic embedder not initialized or grading fails
- Logs error information when grading fails

**Flow:**

```rust
fn check_answer(&self, answer: &str) -> bool {
    // Get embedder, return false if not initialized
    let embedder = match SEMANTIC_EMBEDDER.get() {
        Some(e) => e,
        None => {
            tracing::error!("Semantic embedder not initialized!");
            return false;
        }
    };

    let grader = SemanticGrader::new(embedder);

    // Grade the answer, return false on error
    let grade = match grader.grade_answer(answer, &self.translation) {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Semantic grading failed: {}", e);
            return false;
        }
    };

    // Accept Excellent and Partial grades as correct
    grade.label != SemanticGradeLabel::Incorrect
}
```

#### 3. ExerciseService Updates

**Location:** `rust/crates/iqrah-core/src/exercises/service.rs`

**New Methods:**

```rust
// Initialize semantic model at app startup
pub fn init_semantic_model(model_path: &str, cache_dir: Option<&str>) -> Result<()>
```

The `cache_dir` parameter allows mobile apps to specify where model files should be cached (important for Android/iOS).

**Updated Methods:**

```rust
// check_answer() now returns semantic metadata for TranslationExercise
pub fn check_answer(&self, exercise: &dyn Exercise, answer: &str) -> ExerciseResponse
```

The response now includes:
- `semantic_grade: Option<String>` - "Excellent" | "Partial" | "Incorrect"
- `similarity_score: Option<f32>` - Cosine similarity (0.0 to 1.0)

#### 4. ExerciseResponse Schema

**Location:** `rust/crates/iqrah-core/src/exercises/types.rs`

**Updated Fields:**

```rust
pub struct ExerciseResponse {
    pub is_correct: bool,
    pub correct_answer: Option<String>,
    pub hint: Option<String>,
    pub options: Option<Vec<String>>,

    // NEW: Semantic grading metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_grade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_score: Option<f32>,
}
```

#### 5. WebSocket API Integration

**Protocol Updates (`rust/crates/iqrah-server/src/protocol.rs`):**

```rust
AnswerChecked {
    is_correct: bool,
    hint: Option<String>,
    correct_answer: Option<String>,
    options: Option<Vec<String>>,
    // NEW
    semantic_grade: Option<String>,
    similarity_score: Option<f32>,
}
```

**Handler Updates (`rust/crates/iqrah-server/src/websocket.rs`):**

The `handle_check_answer()` function now passes through semantic metadata from `ExerciseResponse`.

## Grading Thresholds

### Default Thresholds

| Grade Label | Similarity Range | Meaning |
|------------|------------------|---------|
| Excellent | ≥ 0.85 | High confidence semantic match |
| Partial | 0.70 - 0.84 | Moderate confidence match |
| Incorrect | < 0.70 | Low confidence, different meaning |

### Threshold Rationale

- **0.85 (Excellent):** High similarity - answer captures the meaning accurately
- **0.70 (Partial):** Moderate similarity - answer has some semantic overlap
- **Below 0.70:** Low similarity - likely different meaning

### Future Tuning

Thresholds should be adjusted based on:
1. Real user data analysis
2. A/B testing results
3. Per-language calibration
4. Per-exercise-type optimization

## Model Loading Strategy

### Initialization

```rust
// At app startup (in main.rs or during initialization)
// For server/desktop (uses default system cache):
ExerciseService::init_semantic_model("minishlab/potion-multilingual-128M", None)?;

// For mobile (Flutter provides cache directory):
let cache_dir = "/data/user/0/com.app/files/huggingface";
ExerciseService::init_semantic_model("minishlab/potion-multilingual-128M", Some(cache_dir))?;
```

The model path can be:
- **HuggingFace model ID:** e.g., `"minishlab/potion-multilingual-128M"` (downloads automatically)
- **Local path:** e.g., `"/path/to/model"` (uses local files)

The cache directory should be:
- **Desktop/Server:** `None` (uses `~/.cache/huggingface`)
- **Mobile:** App documents directory from Flutter (e.g., via `getApplicationDocumentsDirectory()`)

### Singleton Pattern

The semantic embedder uses `OnceCell` for lazy initialization:
- Loaded once at app startup
- Shared across all exercises
- Thread-safe access
- Zero-cost when not initialized

## Performance Considerations

### Target Performance

| Device Class | Target Inference Time |
|-------------|----------------------|
| High-end | < 30ms per answer |
| Mid-range | < 50ms per answer |
| Low-end | < 100ms per answer |

### Optimizations

1. **Batch Processing:** The embedder can encode multiple texts at once
2. **Model Quantization:** Use FP16 or INT8 models for faster inference
3. **Caching:** Future optimization - cache embeddings for common reference answers

### Resource Usage

- **Model Size:** ~200MB for quantized BGE-M3
- **Memory:** Minimal additional memory per request
- **CPU Usage:** Brief spike during inference

## Error Handling

### Graceful Failure

**The semantic model is required for Translation and Memorization exercises.** If the model is not initialized or fails:

1. **Without Model:** Returns `false` (answer marked incorrect)
2. **Model Load Failure:** Server fails to start (returns error during initialization)
3. **Inference Error:** Logs error, returns `false` (answer marked incorrect)
4. **Other Exercise Types:** MCQ exercises are unaffected (don't use semantic grading)

### Critical Setup

**IMPORTANT:** You must call `ExerciseService::init_semantic_model()` at server startup:

```rust
// In main.rs - this will fail server startup if model can't load
ExerciseService::init_semantic_model("minishlab/potion-multilingual-128M", cache_dir.as_deref())?;
```

Failure to initialize will result in all Translation and Memorization exercises returning `false`.

## Adding Semantic Grading to Other Exercise Types

### Steps to Add Semantic Grading

1. **Import Dependencies:**
   ```rust
   use crate::semantic::grader::{SemanticGrader, SEMANTIC_EMBEDDER};
   ```

2. **Update `check_answer()` Method:**
   ```rust
   fn check_answer(&self, answer: &str) -> bool {
       // Get embedder, return false if not initialized
       let embedder = match SEMANTIC_EMBEDDER.get() {
           Some(e) => e,
           None => {
               tracing::error!("Semantic embedder not initialized!");
               return false;
           }
       };

       let grader = SemanticGrader::new(embedder);

       // Grade the answer, return false on error
       let grade = match grader.grade_answer(answer, &self.reference) {
           Ok(g) => g,
           Err(e) => {
               tracing::error!("Semantic grading failed: {}", e);
               return false;
           }
       };

       grade.label != SemanticGradeLabel::Incorrect
   }
   ```

3. **Update ExerciseService:**
   - Add downcast logic to `check_answer()`
   - Populate semantic fields in `ExerciseResponse`

### Example: MemorizationExercise

For Arabic memorization, you would need:
1. A model that supports Arabic well (BGE-M3 does)
2. Normalized comparison (remove diacritics before embedding)
3. Higher threshold (Arabic requires more precision)

## Testing

### Unit Tests

**Location:** `rust/crates/iqrah-core/src/semantic/`

- `embedding.rs`: Cosine similarity tests (5 tests)
- `grader.rs`: Classification logic tests (4 tests)
- `tests.rs`: Module integration tests (3 tests)

### Integration Tests

**Existing tests:** All 18 exercise tests pass
- `test_exact_match`
- `test_case_insensitive`
- `test_partial_match`
- `test_word_overlap`
- `test_no_match`
- ... and 13 more

### Manual Testing with Real Model

Tests marked with `#[ignore]` require a real model:

```bash
# Run ignored tests with a real model
cargo test --package iqrah-core semantic --ignored
```

## Deployment

### Production Checklist

- [ ] Choose model: HuggingFace ID or bundled model
- [ ] Test inference performance on target devices
- [ ] Monitor semantic grading usage and accuracy
- [ ] Collect user feedback
- [ ] A/B test threshold values
- [ ] Set up metrics/logging

### Model Distribution

**Option A: Bundle in App Assets**
- Pros: Works offline immediately
- Cons: Increases app size (~200MB)

**Option B: Download on First Run**
- Pros: Smaller initial app size
- Cons: Requires network, one-time download

**Recommendation:** Start with bundled model for simplicity.

## Future Improvements

### Short Term
1. Add semantic grading to MemorizationExercise (Arabic)
2. Implement embedding caching for common answers
3. A/B test threshold values
4. Add telemetry for semantic grading accuracy

### Medium Term
1. Multi-reference answer support
2. Per-language threshold tuning
3. Progressive model loading (download smaller model first)
4. Semantic search for similar exercises

### Long Term
1. Fine-tune BGE-M3 on Quranic text
2. Custom model training for Islamic terminology
3. Semantic clustering for exercise difficulty
4. Intelligent hint generation based on semantic similarity

## References

- **model2vec-rs:** https://github.com/MinishLab/model2vec-rs
- **BGE-M3:** https://huggingface.co/BAAI/bge-m3
- **Cosine Similarity:** https://en.wikipedia.org/wiki/Cosine_similarity
- **Phase 4 Implementation:** See `docs/database-architecture/v2-implementation-specs/05-knowledge-axis-and-session-integration.md`

## Troubleshooting

### Model Won't Load

**Error:** `Failed to load semantic model`

**Solutions:**
1. Check model path is correct
2. Verify network connection (for HuggingFace models)
3. Check disk space for model cache
4. Try a different model or local path

### Poor Grading Accuracy

**Symptoms:** Incorrect answers marked correct, or vice versa

**Solutions:**
1. Adjust thresholds (try `with_thresholds()`)
2. Check if reference answers are clear and concise
3. Verify model supports the language
4. Collect examples and analyze similarity scores

### Performance Issues

**Symptoms:** Slow inference (> 100ms)

**Solutions:**
1. Use quantized model (FP16 or INT8)
2. Implement embedding caching
3. Profile with specific device
4. Consider smaller model

## Contact

For questions or issues:
- File an issue in the repository
- Check existing documentation
- Review the code in `rust/crates/iqrah-core/src/semantic/`
