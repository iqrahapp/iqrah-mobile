# AI Agent Prompt: Implement Semantic Answer Grading

## 🎯 Your Mission

Integrate **semantic similarity grading** into the Iqrah Quran learning app using `model2vec-rs` with the BGE-M3 multilingual model. This will upgrade the `TranslationExercise` from simple fuzzy matching to true semantic understanding.

---

## 📖 Context You Need to Read First

**CRITICAL:** Before starting, read these files to understand the existing system:

1. **Phase 4 Exercise System:**
   - `docs/database-architecture/v2-implementation-specs/05-knowledge-axis-and-session-integration.md`
   - This explains the knowledge axis system and exercise architecture

2. **Current Exercise Implementation:**
   - `rust/crates/iqrah-core/src/exercises/` - All exercise types
   - `rust/crates/iqrah-core/src/exercises/translation.rs` - **Primary file to upgrade**
   - `rust/crates/iqrah-core/src/exercises/service.rs` - Exercise orchestration

3. **Task Specification:**
   - `docs/tasks/semantic-grading/README.md` - **Full technical spec**

4. **WebSocket API:**
   - `rust/crates/iqrah-server/src/protocol.rs` - Command/Event definitions
   - `rust/crates/iqrah-server/src/websocket.rs` - Exercise handlers

---

## ✅ What to Do (Step-by-Step)

### Step 1: Repository Recon (15 min)

1. Read the files listed above
2. Run the existing tests to understand current behavior:
   ```bash
   cd rust
   cargo test --package iqrah-core exercises
   ```
3. Look at the current `TranslationExercise.fuzzy_match()` implementation
4. Understand how `ExerciseService.check_answer()` works

**Deliverable:** Write a brief summary of your findings (don't tell the user, just for your own understanding)

---

### Step 2: Add `model2vec-rs` Dependency (10 min)

1. Add to `rust/crates/iqrah-core/Cargo.toml`:
   ```toml
   [dependencies]
   model2vec = "0.1"  # Check for latest version
   ```

2. Verify it compiles:
   ```bash
   cargo check --package iqrah-core
   ```

---

### Step 3: Create Semantic Module (60-90 min)

Create these files:

**`rust/crates/iqrah-core/src/semantic/mod.rs`:**
```rust
pub mod embedding;
pub mod grader;

#[cfg(test)]
mod tests;
```

**`rust/crates/iqrah-core/src/semantic/embedding.rs`:**
- Implement `SemanticEmbedder` struct
- Load BGE-M3 model
- Provide `embed()` and `similarity()` methods
- Use singleton pattern (`OnceCell` or `lazy_static`)

**`rust/crates/iqrah-core/src/semantic/grader.rs`:**
- Implement `SemanticGrader` struct
- Define `SemanticGradeLabel` enum (Excellent/Partial/Incorrect)
- Implement threshold-based grading logic
- Support multiple reference answers

**`rust/crates/iqrah-core/src/semantic/tests.rs`:**
- Test model loading
- Test similarity calculations
- Test grading thresholds
- Test edge cases (empty strings, special characters, etc.)

**Export from `rust/crates/iqrah-core/src/lib.rs`:**
```rust
pub mod semantic;
```

---

### Step 4: Upgrade TranslationExercise (30-45 min)

**File:** `rust/crates/iqrah-core/src/exercises/translation.rs`

1. Import semantic grading:
   ```rust
   use crate::semantic::grader::{SemanticGrader, SEMANTIC_EMBEDDER};
   ```

2. Update `check_answer()` method to use semantic grading when available, with fuzzy matching as fallback:
   ```rust
   fn check_answer(&self, answer: &str) -> bool {
       if let Some(embedder) = SEMANTIC_EMBEDDER.get() {
           // Use semantic grading
           let grader = SemanticGrader::new(embedder);
           match grader.grade_answer(answer, &self.translation) {
               Ok(grade) => grade.label != SemanticGradeLabel::Incorrect,
               Err(_) => Self::fuzzy_match(answer, &self.translation),
           }
       } else {
           // Fallback to fuzzy match
           Self::fuzzy_match(answer, &self.translation)
       }
   }
   ```

3. Keep the existing `fuzzy_match()` as a fallback
4. Add tests for semantic grading behavior

---

### Step 5: Update ExerciseResponse (20 min)

**File:** `rust/crates/iqrah-core/src/exercises/types.rs`

Add semantic metadata to `ExerciseResponse`:
```rust
pub struct ExerciseResponse {
    pub is_correct: bool,
    pub correct_answer: Option<String>,
    pub hint: Option<String>,
    pub options: Option<Vec<String>>,

    // NEW: Semantic grading metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_grade: Option<String>,  // "Excellent" | "Partial" | "Incorrect"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_score: Option<f32>,
}
```

---

### Step 6: Update ExerciseService (30 min)

**File:** `rust/crates/iqrah-core/src/exercises/service.rs`

1. Add initialization function:
   ```rust
   pub fn init_semantic_model(model_path: &str) -> Result<()>
   ```

2. Update `check_answer()` to populate semantic fields in `ExerciseResponse`

3. Add tests for the new functionality

---

### Step 7: WebSocket API Integration (30 min)

**File:** `rust/crates/iqrah-server/src/main.rs`

Add model initialization in `main()`:
```rust
// After database init, before server start
ExerciseService::init_semantic_model("models/bge-m3.onnx")?;
tracing::info!("✅ Semantic grading model loaded");
```

**File:** `rust/crates/iqrah-server/src/protocol.rs`

Update `AnswerChecked` event to include semantic fields:
```rust
AnswerChecked {
    is_correct: bool,
    hint: Option<String>,
    correct_answer: Option<String>,
    options: Option<Vec<String>>,
    // NEW
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_grade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    similarity_score: Option<f32>,
}
```

**File:** `rust/crates/iqrah-server/src/websocket.rs`

Update `handle_check_answer()` to pass through semantic metadata from `ExerciseResponse`.

---

### Step 8: Integration Testing (45-60 min)

**Create:** `rust/tests/semantic_grading_test.rs`

Test scenarios:
1. Model initialization
2. Translation exercise with high similarity (should pass)
3. Translation exercise with low similarity (should fail)
4. Translation exercise with partial similarity
5. Fallback to fuzzy matching when model not loaded
6. WebSocket end-to-end test

**Run all tests:**
```bash
cargo test --package iqrah-core
cargo test --package iqrah-server
cargo test --workspace
```

---

### Step 9: Documentation (20 min)

**Create:** `docs/semantic-grading-architecture.md`

Document:
- Architecture overview
- Model loading strategy
- Threshold configuration
- How to add semantic grading to other exercise types
- Performance considerations
- Future improvements

---

### Step 10: Commit & Push (10 min)

**Create a clear commit message:**
```
Add semantic answer grading with model2vec-rs + BGE-M3

Implements semantic similarity grading for TranslationExercise using
model2vec-rs with BGE-M3 multilingual embeddings.

## Changes

### Core Modules (iqrah-core)
- Add `semantic/` module with embedding and grading logic
- Upgrade `TranslationExercise.check_answer()` to use semantic grading
- Add semantic metadata to `ExerciseResponse`
- Add `ExerciseService.init_semantic_model()` for initialization

### WebSocket API (iqrah-server)
- Initialize semantic model on server startup
- Update `AnswerChecked` event with semantic fields
- Pass semantic grades through WebSocket handlers

### Testing
- Add semantic grading unit tests
- Add integration tests for translation exercises
- All existing tests still pass

## Performance
- Target: < 100ms per answer
- Uses singleton pattern for model sharing
- Graceful fallback to fuzzy matching if model unavailable

## Future Work
- Add semantic grading to other exercise types
- Implement embedding caching
- A/B test threshold values
- Flutter UI integration

Addresses: docs/tasks/semantic-grading/README.md
Status: Phase 1-3 Complete (Core Rust Implementation)
```

---

## ⚠️ Important Constraints

1. **Don't break existing functionality:**
   - All current tests must still pass
   - Fuzzy matching should work as fallback
   - Other exercise types (MCQ, Memorization) should be unaffected

2. **Performance:**
   - Model loading should be lazy (don't block app startup)
   - Embedding computation should be fast (< 100ms target)
   - Use singleton pattern to avoid reloading

3. **Error handling:**
   - Gracefully handle model loading failures
   - Fall back to fuzzy matching on any semantic grading error
   - Log errors but don't crash

4. **Code quality:**
   - Follow existing code style in the project
   - Add comprehensive tests
   - Document public APIs
   - Use meaningful variable names

5. **Model file:**
   - For now, assume the model file exists at a known path
   - Don't worry about bundling/downloading in this task
   - Focus on the integration logic

---

## 📦 Deliverables Checklist

- [ ] `model2vec-rs` dependency added to `iqrah-core/Cargo.toml`
- [ ] `rust/crates/iqrah-core/src/semantic/` module created with:
  - [ ] `embedding.rs` - Model wrapper
  - [ ] `grader.rs` - Grading logic
  - [ ] `tests.rs` - Unit tests
- [ ] `TranslationExercise.check_answer()` upgraded to use semantic grading
- [ ] `ExerciseResponse` includes semantic metadata
- [ ] `ExerciseService.init_semantic_model()` implemented
- [ ] WebSocket API updated to pass through semantic grades
- [ ] Integration tests added (`rust/tests/semantic_grading_test.rs`)
- [ ] All existing tests still pass
- [ ] Documentation created (`docs/semantic-grading-architecture.md`)
- [ ] Code committed and pushed

---

## 🆘 If You Get Stuck

1. **Can't find a file?**
   - Use `find` or `rg` to search: `rg "TranslationExercise" --files-with-matches`

2. **Tests failing?**
   - Run specific test: `cargo test test_name -- --nocapture`
   - Check what changed: `git diff`

3. **Compilation errors?**
   - Read the error message carefully
   - Check imports and module structure
   - Ensure new modules are exported in `mod.rs`

4. **Model loading fails?**
   - For now, skip actual model loading in tests (use mocks)
   - Focus on the integration logic
   - Model file handling is a separate concern

5. **Unsure about architecture?**
   - Look at existing `exercises/` modules for patterns
   - Follow the same structure as `mcq.rs` or `translation.rs`
   - When in doubt, keep it simple

---

## 🎓 Learning Resources

- **model2vec-rs:** https://github.com/MinishLab/model2vec
- **BGE-M3:** https://huggingface.co/BAAI/bge-m3
- **Cosine Similarity:** https://en.wikipedia.org/wiki/Cosine_similarity
- **Existing Phase 4 Implementation:** See commits `5142cd3d6`, `bc74212cd`, `0099b5852`

---

## 🚀 Ready to Start?

Begin with **Step 1: Repository Recon**. Read the existing code, run the tests, and get familiar with the exercise system. Then proceed step-by-step through the implementation.

Good luck! 🎉
