# TODO: Enum-Based Exercise Architecture Integration

**Status:** Foundation complete, integration pending
**Priority:** High
**Estimated Effort:** 2-3 weeks

---

## What's Done ✅

- Modern `ExerciseData` enum with 18 variants (stores only keys/IDs)
- `ExerciseValidator` trait with Arabic/English normalization
- Generator functions for all exercise types
- 56 comprehensive tests (all passing)
- Ayah Chain UX improvements (time tracking, mistake feedback)
- Echo Recall session statistics

---

## What's Left 🚧

### 1. Rust Integration (High Priority)

**File:** `rust/crates/iqrah-core/src/exercises/service.rs`

```rust
// Add this method:
impl ExerciseService {
    pub async fn generate_exercise_v2(&self, node_id: &str) -> Result<ExerciseData> {
        // Call appropriate generator from generators.rs
        // Example: generate_memorization(node_id, &self.content_repo).await
    }
}
```

**Tasks:**
- [ ] Add `generate_exercise_v2()` method to `ExerciseService`
- [ ] Route calls to appropriate generator based on node type
- [ ] Add integration tests (18 tests, one per exercise type)

---

### 2. Serialization Fixes (Critical)

**File:** `rust/crates/iqrah-core/src/exercises/ayah_chain.rs`

```rust
// Add serde derives to MistakeDetails:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeDetails { /* ... */ }

// Add to AyahChainStats:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AyahChainStats { /* ... */ }
```

**Tasks:**
- [ ] Add `Serialize, Deserialize` to `MistakeDetails`
- [ ] Add `Serialize, Deserialize` to `AyahChainStats`
- [ ] Add `Serialize, Deserialize` to `EchoRecallStats` (in `domain/models.rs`)

---

### 3. FFI Bridge (Critical)

**Commands:**
```bash
cd /home/user/iqrah-mobile
flutter_rust_bridge_codegen generate
```

**Tasks:**
- [ ] Run codegen to expose new types to Flutter
- [ ] Verify `ExerciseData` enum is accessible in Dart
- [ ] Verify `ValidationResult` is accessible in Dart
- [ ] Add API method to expose `generate_exercise_v2()`

---

### 4. Flutter Content Fetching Service (High Priority)

**File:** `lib/features/exercises/services/exercise_content_service.dart`

```dart
class ExerciseContentService {
  // Fetch content based on user preferences
  Future<VerseContent> fetchVerseContent(
    String verseKey,
    UserPreferences prefs,
  ) async {
    final textVariant = prefs.textVariant; // 'uthmani', 'tajweed', 'indopak'
    final text = await db.getVerseText(verseKey, textVariant);
    return VerseContent(text: text, verseKey: verseKey);
  }

  Future<WordContent> fetchWordContent(int wordId, UserPreferences prefs);
  Future<String> fetchTranslation(int wordId, int translatorId);
}
```

**Tasks:**
- [ ] Create `ExerciseContentService` class
- [ ] Implement `fetchVerseContent()` with text variant support
- [ ] Implement `fetchWordContent()`
- [ ] Implement `fetchTranslation()`
- [ ] Add caching layer for performance

---

### 5. Flutter Widget Updates (Medium Priority)

**Example:** `lib/features/exercises/widgets/full_verse_input_widget.dart`

```dart
// Before (uses pre-fetched text):
Text(exercise.correctVerseText)

// After (fetches based on user preference):
FutureBuilder<VerseContent>(
  future: contentService.fetchVerseContent(
    exercise.verseKey,
    userPrefs,
  ),
  builder: (context, snapshot) => Text(snapshot.data?.text ?? ''),
)
```

**Tasks:**
- [ ] Update all 17 exercise widgets to use `ExerciseContentService`
- [ ] Remove direct text field access
- [ ] Add loading states for content fetching
- [ ] Add error handling

---

### 6. Database Query Optimization (Low Priority)

**File:** `rust/crates/iqrah-core/src/ports/content_repository.rs`

```rust
// Add batch query methods:
async fn get_verses_batch(&self, verse_keys: &[String])
    -> Result<HashMap<String, Verse>>;

async fn get_words_batch(&self, word_ids: &[i32])
    -> Result<HashMap<i32, Word>>;
```

**Tasks:**
- [ ] Add batch query methods to `ContentRepository` trait
- [ ] Implement in `SqliteContentRepository`
- [ ] Update `ExerciseContentService` to use batch queries
- [ ] Profile and optimize query performance

---

### 7. Documentation (Low Priority)

**Tasks:**
- [ ] Write Flutter integration guide with examples
- [ ] Document text variant system (Uthmani, Tajweed, Indopak, Simple)
- [ ] Add API usage examples
- [ ] Create migration guide for existing widgets

---

## Testing Checklist

- [ ] 18 generator tests (one per exercise type)
- [ ] Integration tests for `generate_exercise_v2()`
- [ ] FFI serialization tests
- [ ] Flutter widget tests with mock content service
- [ ] End-to-end test: Generate exercise → Fetch content → Display

---

## Performance Targets

- Exercise generation: < 100ms
- Content fetching: < 50ms (with caching)
- Serialization: < 10ms
- Total end-to-end: < 200ms

---

## Files to Modify

### Rust
- `rust/crates/iqrah-core/src/exercises/service.rs` - Add v2 method
- `rust/crates/iqrah-core/src/exercises/ayah_chain.rs` - Add serde derives
- `rust/crates/iqrah-core/src/domain/models.rs` - Add serde to EchoRecallStats
- `rust/src/api.rs` - Expose new API methods

### Flutter
- `lib/features/exercises/services/exercise_content_service.dart` - Create
- All widgets in `lib/features/exercises/widgets/` - Update (17 files)

---

## Migration Strategy

**Phase 1:** Rust integration (1 week)
- Add `generate_exercise_v2()` method
- Fix serialization issues
- Run FFI codegen

**Phase 2:** Flutter service layer (3-5 days)
- Create `ExerciseContentService`
- Implement content fetching with text variant support

**Phase 3:** Widget migration (1 week)
- Update widgets one by one
- Add feature flag to toggle between old/new system
- Test each widget thoroughly

**Phase 4:** Cleanup (2-3 days)
- Remove old code paths
- Remove feature flags
- Update documentation

---

## Breaking Changes

**None** - This is additive. Old system continues to work.

The new enum system exists in parallel. Widgets can gradually migrate from:
```dart
exercise.correctVerseText → contentService.fetchVerseContent(exercise.verseKey)
```

---

## Questions for Product/Design

1. **Text Variants:** Which text variants should we support?
   - Uthmani (current)
   - Tajweed (color-coded rules)
   - Indopak (different script style)
   - Simple (no diacritics)

2. **Translator Selection:** Should users select translator per-exercise or globally?

3. **Caching Strategy:** Cache exercise structures? Cache content? Both?

4. **Offline Mode:** Pre-fetch content for offline use?

---

## Contact

**Code Location:** `rust/crates/iqrah-core/src/exercises/`
- `exercise_data.rs` - Enum definitions
- `generators.rs` - Exercise generators
- `validator.rs` - Answer validation

**Previous Work:** See commit `0666b1e` for full implementation details

**Questions:** Refer to architectural analysis in PR description
