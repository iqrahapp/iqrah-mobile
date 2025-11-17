# Task: Integrate Semantic Answer Grading with model2vec-rs + BGE-M3

## Status: 🔴 Not Started

**Priority:** High
**Complexity:** Medium-High
**Estimated Time:** 4-8 hours
**Dependencies:** Phase 4 Exercise System (✅ Complete)

---

## 📋 Context

The Iqrah app currently has **5 exercise types** implemented in Phase 4:
1. MemorizationExercise (free-form Arabic recall)
2. TranslationExercise (free-form translation)
3. McqArToEn (multiple choice AR→EN)
4. McqEnToAr (multiple choice EN→AR)
5. *(Future: TajweedExercise)*

**Current Grading Approaches:**

| Exercise Type | Current Grading Method | Location |
|--------------|------------------------|----------|
| MemorizationExercise | Exact match (removes diacritics) | `rust/crates/iqrah-core/src/exercises/memorization.rs:76-83` |
| TranslationExercise | Fuzzy matching (50% word overlap) | `rust/crates/iqrah-core/src/exercises/translation.rs:43-76` |
| MCQ | Exact match (normalized) | `rust/crates/iqrah-core/src/exercises/mcq.rs:171-175` |

**Problem:** The current fuzzy matching for `TranslationExercise` is too simplistic:
- Only checks word overlap (50% threshold)
- Doesn't understand semantics
- False negatives: "In the name of God" vs "By God's name" (low overlap, same meaning)
- False positives: "The name is God" vs "In the name" (high overlap, different meaning)

**Solution:** Integrate **semantic similarity** using `model2vec-rs` with **BGE-M3** (multilingual embeddings).

---

## 🎯 Goals

1. **Replace fuzzy matching** in `TranslationExercise` with semantic similarity
2. **Keep it fast:** < 50-100ms per answer on mobile devices
3. **On-device:** No network dependency
4. **Multilingual:** Support Arabic, English, French, Spanish, etc. (BGE-M3 handles 100+ languages)
5. **Maintain backward compatibility:** Don't break existing exercises

---

## 🏗️ Project Structure (Relevant Files)

### Rust Crates
```
rust/
├── crates/
│   ├── iqrah-core/              # Core learning logic
│   │   ├── src/
│   │   │   ├── exercises/       # ⚠️ PRIMARY WORK AREA
│   │   │   │   ├── mod.rs
│   │   │   │   ├── types.rs     # Exercise trait, ExerciseResponse
│   │   │   │   ├── service.rs   # ExerciseService orchestration
│   │   │   │   ├── memorization.rs
│   │   │   │   ├── translation.rs  # ⚠️ UPGRADE THIS
│   │   │   │   └── mcq.rs
│   │   │   ├── domain/
│   │   │   └── services/
│   │   └── Cargo.toml           # ⚠️ Add model2vec-rs here
│   │
│   ├── iqrah-server/            # WebSocket API server
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── protocol.rs      # Command/Event definitions
│   │   │   └── websocket.rs     # Exercise handlers
│   │   └── Cargo.toml
│   │
│   └── iqrah-storage/           # Database layer (SQLite)
│       └── src/
│
└── tests/                       # Integration tests
    └── knowledge_axis_test.rs
```

### Flutter App
```
lib/
├── bridge/                      # ⚠️ FFI bindings to Rust
├── screens/
│   └── exercises/               # ⚠️ Exercise UI (likely needs updates)
└── models/
```

---

## 📦 Implementation Plan

### Phase 1: Add Semantic Embedding Module (Rust)

**Location:** `rust/crates/iqrah-core/src/semantic/`

**Files to Create:**
- `mod.rs` - Module exports
- `embedding.rs` - BGE-M3 model wrapper
- `grader.rs` - Semantic grading logic
- `tests.rs` - Unit tests

**Dependencies (add to `iqrah-core/Cargo.toml`):**
```toml
model2vec = "0.1"  # Adjust to latest version
```

**Key Design:**
```rust
// embedding.rs
pub struct SemanticEmbedder {
    model: Model,
}

impl SemanticEmbedder {
    pub fn new(model_path: &str) -> Result<Self>;
    pub fn embed(&self, text: &str) -> Result<Embedding>;
    pub fn similarity(&self, a: &str, b: &str) -> Result<f32>;
}

// grader.rs
pub enum SemanticGradeLabel {
    Excellent,   // ≥ 0.85 similarity
    Partial,     // ≥ 0.70 similarity
    Incorrect,   // < 0.70 similarity
}

pub struct SemanticGrader<'a> {
    embedder: &'a SemanticEmbedder,
    excellent_min: f32,  // Default: 0.85
    partial_min: f32,    // Default: 0.70
}

impl SemanticGrader {
    pub fn grade_answer(&self, user: &str, reference: &str) -> Result<SemanticGrade>;
    pub fn grade_against_many(&self, user: &str, refs: &[String]) -> Result<SemanticGrade>;
}
```

**Model Loading Strategy:**
- Bundle BGE-M3 model file in app assets
- Load once at app startup (use `lazy_static` or `OnceCell`)
- Share single instance across all exercises

---

### Phase 2: Upgrade TranslationExercise

**File:** `rust/crates/iqrah-core/src/exercises/translation.rs`

**Current Implementation:**
```rust
// Line 43-76: fuzzy_match() with word overlap
fn check_answer(&self, answer: &str) -> bool {
    Self::fuzzy_match(answer, &self.translation)
}
```

**New Implementation:**
```rust
use crate::semantic::grader::{SemanticGrader, SemanticGradeLabel};

impl Exercise for TranslationExercise {
    fn check_answer(&self, answer: &str) -> bool {
        // Option 1: Keep backward compatible (fuzzy as fallback)
        if let Some(grader) = SEMANTIC_GRADER.get() {
            match grader.grade_answer(answer, &self.translation) {
                Ok(grade) => grade.label != SemanticGradeLabel::Incorrect,
                Err(_) => Self::fuzzy_match(answer, &self.translation), // Fallback
            }
        } else {
            Self::fuzzy_match(answer, &self.translation)
        }
    }
}
```

**Update ExerciseResponse:**
```rust
// types.rs - Add semantic metadata
pub struct ExerciseResponse {
    pub is_correct: bool,
    pub correct_answer: Option<String>,
    pub hint: Option<String>,
    pub options: Option<Vec<String>>,

    // NEW: Semantic grading metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_grade: Option<SemanticGradeLabel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_score: Option<f32>,
}
```

---

### Phase 3: Update ExerciseService

**File:** `rust/crates/iqrah-core/src/exercises/service.rs`

**Add Initialization:**
```rust
use crate::semantic::grader::SemanticGrader;
use once_cell::sync::OnceCell;

static SEMANTIC_EMBEDDER: OnceCell<SemanticEmbedder> = OnceCell::new();

impl ExerciseService {
    pub fn init_semantic_model(model_path: &str) -> Result<()> {
        let embedder = SemanticEmbedder::new(model_path)?;
        SEMANTIC_EMBEDDER.set(embedder).ok();
        Ok(())
    }
}
```

**Update `check_answer()` (Line 94-120):**
```rust
pub fn check_answer(&self, exercise: &dyn Exercise, answer: &str) -> ExerciseResponse {
    let is_correct = exercise.check_answer(answer);

    // Try semantic grading if available
    let (semantic_grade, similarity_score) = if let Some(embedder) = SEMANTIC_EMBEDDER.get() {
        // Extract reference answer from exercise somehow
        // This might require adding a new method to Exercise trait
        (Some(grade.label), Some(grade.similarity))
    } else {
        (None, None)
    };

    // ... rest of logic

    ExerciseResponse {
        is_correct,
        semantic_grade,
        similarity_score,
        // ... rest
    }
}
```

---

### Phase 4: WebSocket API Integration

**File:** `rust/crates/iqrah-server/src/main.rs`

**Add to `main()`:**
```rust
// After database init, before server start
ExerciseService::init_semantic_model("models/bge-m3.onnx")?;
tracing::info!("Semantic grading model loaded");
```

**File:** `rust/crates/iqrah-server/src/protocol.rs`

**Update Events:**
```rust
// Add new command
InitSemanticModel {
    model_path: String,
},

// Update AnswerChecked event (already has options field)
AnswerChecked {
    is_correct: bool,
    hint: Option<String>,
    correct_answer: Option<String>,
    options: Option<Vec<String>>,

    // NEW fields
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_grade: Option<String>,  // "Excellent" | "Partial" | "Incorrect"
    #[serde(skip_serializing_if = "Option::is_none")]
    similarity_score: Option<f32>,
}
```

---

### Phase 5: Testing

**Unit Tests (`rust/crates/iqrah-core/src/semantic/tests.rs`):**
```rust
#[test]
fn test_semantic_similarity_high() {
    // "In the name of God" vs "By God's name"
    let sim = embedder.similarity(text1, text2)?;
    assert!(sim > 0.85);
}

#[test]
fn test_semantic_similarity_low() {
    // "In the name" vs "The creator"
    let sim = embedder.similarity(text1, text2)?;
    assert!(sim < 0.70);
}

#[test]
fn test_grader_thresholds() {
    let grader = SemanticGrader::new(&embedder);

    let result = grader.grade_answer("In the name", "By God's name")?;
    assert_eq!(result.label, SemanticGradeLabel::Excellent);
}
```

**Integration Tests (`rust/tests/semantic_grading_test.rs`):**
```rust
#[tokio::test]
async fn test_translation_exercise_semantic_grading() {
    ExerciseService::init_semantic_model("test_models/bge-m3.onnx")?;

    let service = ExerciseService::new(content_repo);
    let exercise = service.generate_exercise("WORD:1:1:1:translation").await?;

    let response = service.check_answer(exercise.as_exercise(), "By God's name");

    assert!(response.is_correct);
    assert_eq!(response.semantic_grade, Some(SemanticGradeLabel::Excellent));
    assert!(response.similarity_score.unwrap() > 0.85);
}
```

---

## 🔧 Technical Considerations

### Model File Management

**Option A: Bundle in App Assets**
```
assets/
└── models/
    └── bge-m3.onnx  (~200MB for quantized version)
```

**Option B: Download on First Run**
- Store in app's document directory
- Show one-time download progress
- Cache locally

**Recommendation:** Start with Option A (bundled) for simplicity.

### Performance Targets

| Device Class | Target Inference Time |
|-------------|----------------------|
| High-end (Snapdragon 8 Gen 2) | < 30ms |
| Mid-range (Snapdragon 7 Gen 1) | < 50ms |
| Low-end (Snapdragon 6 Gen 1) | < 100ms |

**Optimization Strategies:**
- Use quantized model (INT8 or FP16)
- Batch embeddings if checking multiple references
- Cache embeddings for common reference answers

### Threshold Tuning

Initial thresholds are conservative:
- **Excellent:** ≥ 0.85 (high confidence)
- **Partial:** ≥ 0.70 (moderate confidence)
- **Incorrect:** < 0.70 (low confidence)

**TODO for future iteration:**
- Collect real user data
- Run A/B tests
- Adjust thresholds per language/exercise type

---

## 📊 Success Criteria

- [ ] `model2vec-rs` + BGE-M3 integrated and loading successfully
- [ ] `SemanticEmbedder` module with tests (>80% coverage)
- [ ] `SemanticGrader` module with configurable thresholds
- [ ] `TranslationExercise.check_answer()` uses semantic grading
- [ ] `ExerciseResponse` includes semantic metadata
- [ ] WebSocket API returns semantic grades
- [ ] All existing tests still pass
- [ ] New semantic grading tests added (≥5 tests)
- [ ] Inference time < 100ms on test device
- [ ] Documentation updated

---

## 🚫 Out of Scope (For This Task)

- Flutter UI updates (separate task)
- Semantic grading for MemorizationExercise (Arabic requires different model)
- Retraining or fine-tuning BGE-M3
- Multi-reference answer support (can be added later)
- Caching embeddings (optimization for later)

---

## 📚 References

- **model2vec-rs:** https://github.com/MinishLab/model2vec
- **BGE-M3:** https://huggingface.co/BAAI/bge-m3
- **Iqrah Exercise System:** `docs/database-architecture/v2-implementation-specs/05-knowledge-axis-and-session-integration.md`
- **Current Phase 4 Implementation:** Commits `5142cd3d6`, `bc74212cd`, `0099b5852`

---

## 🎯 Getting Started

See: `docs/tasks/semantic-grading/AGENT_PROMPT.md`
