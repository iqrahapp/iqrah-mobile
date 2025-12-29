# Iqrah Backend-Frontend Gap Analysis Report

**Version**: 1.0  \
**Date**: 2025-12-28  \
**Analyzed by**: Codex (GPT-5)

---

## Executive Summary

The Rust backend (iqrah-core + iqrah-api) has a complete exercise generator and scheduling stack, with database-backed content and user state, and a working Flutter Rust Bridge (FRB) integration for session generation and content fetch. Flutter currently uses those bindings to render only three exercise types (Memorization, McqArToEn, McqEnToAr) with basic UI/logic, and grades are computed entirely on the client without backend validation.

Critical gaps block the highest-priority EchoRecall flow: EchoRecall is implemented in Rust but is not exposed via FFI, has no Dart models, and has no renderer. Additionally, the frontend treats `nodeId` strings as numeric word IDs, but the backend generates `WORD_INSTANCE:*` IDs for word-level exercises, causing incorrect content fetches and translation lookups. Debug tooling requirements (node selector filters, energy propagation queries, DB inspection) are largely absent from the FFI surface.

Top recommendations: (1) expose EchoRecall (state + actions + metrics) via FRB and implement its Dart models/renderer, (2) fix word-instance ID handling and translation fetching in the Flutter layer, and (3) add debug APIs (node filters, energy snapshots, propagation queries, SQL inspector) and regenerate FRB artifacts to align Rust/Dart surfaces.

---

## 1. FFI Bindings Status

### 1.1 Exposed Functions

Source of truth: `rust/crates/iqrah-api/src/frb_generated.rs` (FRB-generated list) + `flutter_rust_bridge.yaml`.

| Rust Function | FFI Binding | Flutter Wrapper | Status |
|---------------|-------------|-----------------|--------|
| `setup_database()` | ✅ | ✅ `setupDatabase()` | Working |
| `setup_database_in_memory()` | ✅ | ✅ `setupDatabaseInMemory()` | Working |
| `get_exercises()` | ✅ | ✅ `getExercises()` | ⚠️ surah_filter ignored |
| `get_exercises_for_node()` | ✅ | ✅ `getExercisesForNode()` | Working |
| `fetch_node_with_metadata()` | ✅ | ✅ `fetchNodeWithMetadata()` | Working |
| `generate_exercise_v2()` | ✅ | ✅ `generateExerciseV2()` | Working |
| `get_verse()` | ✅ | ✅ `getVerse()` | Working |
| `get_word()` | ✅ | ✅ `getWord()` | Working |
| `get_words_for_verse()` | ✅ | ✅ `getWordsForVerse()` | Working |
| `get_word_translation()` | ✅ | ✅ `getWordTranslation()` | ⚠️ data availability needs verification |
| `process_review()` | ✅ | ✅ `processReview()` | Working |
| `get_dashboard_stats()` | ✅ | ✅ `getDashboardStats()` | Working |
| `get_debug_stats()` | ✅ | ✅ `getDebugStats()` | ⚠️ total_edges_count hardcoded 0 |
| `reseed_database()` | ✅ | ✅ `reseedDatabase()` | ⚠️ TODO stub |
| `get_session_preview()` | ✅ | ✅ `getSessionPreview()` | Working |
| `clear_session()` | ✅ | ✅ `clearSession()` | Working |
| `search_nodes()` | ✅ | ✅ `searchNodes()` | ⚠️ prefix search only |
| `get_available_surahs()` | ✅ | ✅ `getAvailableSurahs()` | ⚠️ TODO returns empty |
| `get_languages()` | ✅ | ✅ `getLanguages()` | Working |
| `get_translators_for_language()` | ✅ | ✅ `getTranslatorsForLanguage()` | Working |
| `get_translator()` | ✅ | ✅ `getTranslator()` | Working |
| `get_preferred_translator_id()` | ✅ | ✅ `getPreferredTranslatorId()` | Working |
| `set_preferred_translator_id()` | ✅ | ✅ `setPreferredTranslatorId()` | Working |
| `get_verse_translation_by_translator()` | ✅ | ✅ `getVerseTranslationByTranslator()` | Working |
| `init_app()` | ✅ | ⚠️ implicit via `RustLib.init()` | Working |
| `drain_telemetry_events()` | ❌ | ❌ | Missing from FRB surface |
| `get_telemetry_event_count()` | ❌ | ❌ | Missing from FRB surface |
| `debug_emit_test_event()` | ❌ | ❌ | Missing from FRB surface |

⚠️ NOTE: `rust/src/frb_generated.rs` references `get_existing_session`, `refresh_priority_scores`, and `query_propagation_details`, but these functions do not exist in `rust/crates/iqrah-api/src/api.rs`. This indicates stale FRB output in `rust/src/` and should be regenerated or removed to avoid confusion.

### 1.2 Missing Bindings (Critical)

1. EchoRecall lifecycle and state
   - `EchoRecallExercise::new(...)`
   - `submit_recall(word_node_id, recall_time_ms)`
   - `state()`, `get_stats()`, `finalize()`
   - **Required for EchoRecall UI + persistence**

2. Exercise validation
   - `ExerciseService::check_answer()` and `ExerciseResponse` not exposed
   - Frontend grades without backend validation

3. Session persistence API
   - `SessionService::get_session_state()` / `save_session_state()` not exposed
   - No resume support from Flutter

4. Debug tooling APIs
   - Node filtering / range parsing
   - Energy snapshots + propagation event queries
   - Edge inspection (incoming/outgoing)
   - DB inspector (arbitrary SQL)

5. Telemetry polling APIs
   - `drain_telemetry_events()` and `get_telemetry_event_count()` exist in Rust but are not in FRB output

### 1.3 Code References

```rust
// rust/crates/iqrah-api/src/api.rs
pub async fn get_exercises(
    user_id: String,
    limit: u32,
    _surah_filter: Option<i32>,
    is_high_yield: bool,
) -> Result<Vec<ExerciseDataDto>> {
    let app = app();
    let due_items = app
        .session_service
        .get_due_items(&user_id, limit, is_high_yield, None)
        .await?;

    let mut exercises = Vec::new();
    for item in due_items {
        let nid_val = item.node.id;
        let ukey = nid::to_ukey(nid_val).unwrap_or_default();
        match app
            .exercise_service
            .generate_exercise_v2(nid_val, &ukey)
            .await
        {
            Ok(ex) => exercises.push(ex.into()),
            Err(e) => tracing::error!("Failed to generate exercise for {}: {}", ukey, e),
        }
    }

    Ok(exercises)
}
```

```dart
// lib/rust_bridge/api.dart
Future<List<ExerciseDataDto>> getExercises({
  required String userId,
  required int limit,
  int? surahFilter,
  required bool isHighYield,
}) => RustLib.instance.api.crateApiGetExercises(
  userId: userId,
  limit: limit,
  surahFilter: surahFilter,
  isHighYield: isHighYield,
);
```

---

## 2. Exercise Implementation Matrix

### 2.1 Complete Status Table

⚠️ VERIFICATION NEEDED: `ExerciseData` enumerates 18 exercise variants. EchoRecall and MemorizationAyah are implemented separately and are not part of `ExerciseData`. The prompt states 17 types; current code suggests 18 + EchoRecall (and a separate MemorizationAyah).

| Exercise Type | Backend | FFI | Frontend | Data Model | Priority | Blockers |
|---------------|---------|-----|----------|------------|----------|----------|
| EchoRecall | ✅ (`echo_recall.rs`) | ❌ | ❌ | ❌ | **P0** | No FFI + no Dart models + no renderer |
| Memorization | ✅ | ✅ | ⚠️ basic UI only | ⚠️ word_instance vs word_id mismatch | **P0** | Word ID parsing incorrect; no backend validation |
| McqArToEn | ✅ | ✅ | ⚠️ basic MCQ | ⚠️ word_instance + translation fetch bug | **P0** | Wrong ID parsing + translation routing |
| McqEnToAr | ✅ | ✅ | ⚠️ basic MCQ | ⚠️ word_instance mismatch | **P0** | Wrong ID parsing |
| Translation | ✅ | ✅ | ❌ | ⚠️ translation routing uses `contains(':')` | P1 | No renderer; translation fetch likely wrong for word nodes |
| ContextualTranslation | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| ClozeDeletion | ✅ | ✅ | ⚠️ uses memorization UI | ✅ | P1 | No answer input/validation |
| FirstLetterHint | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| MissingWordMcq | ✅ | ✅ | ❌ | ⚠️ distractors are node IDs | P1 | No renderer + ID mapping |
| NextWordMcq | ✅ | ✅ | ❌ | ⚠️ distractors are node IDs | P1 | No renderer + ID mapping |
| FullVerseInput | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| AyahChain | ✅ | ✅ | ❌ | ⚠️ BigInt fields (`currentIndex`, `completedCount`) | P1 | No renderer + stateful flow |
| FindMistake | ✅ | ✅ | ❌ | ⚠️ word IDs vs word_instance | P1 | No renderer + ID mapping |
| AyahSequence | ✅ | ✅ | ❌ | ⚠️ sequence of node IDs (ukey) | P1 | No renderer |
| IdentifyRoot | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| ReverseCloze | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| TranslatePhrase | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| PosTagging | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| CrossVerseConnection | ✅ | ✅ | ❌ | ✅ | P1 | No renderer |
| MemorizationAyah (stateful) | ✅ (`memorization_ayah.rs`) | ❌ | ❌ | ❌ | P2 | No FFI + no Dart models |

**Legend**:
- ✅ Implemented and working
- ⚠️ Exists but has issues
- ❌ Missing/not implemented

### 2.2 EchoRecall Deep Dive (Priority #1)

**Backend capabilities**:

```rust
// rust/crates/iqrah-core/src/exercises/echo_recall.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoRecallExercise {
    state: EchoRecallState,
    user_id: String,
    ayah_node_ids: Vec<String>,
}

pub fn submit_recall(&mut self, word_node_id: &str, recall_time_ms: u32) -> Result<f64> {
    let word_index = self
        .state
        .words
        .iter()
        .position(|w| w.node_id == word_node_id)
        .ok_or_else(|| anyhow!("Word not found in session: {}", word_node_id))?;

    let energy_delta = recall_model::calculate_energy_change(recall_time_ms);
    let new_energy = (self.state.words[word_index].energy + energy_delta).clamp(0.0, 1.0);
    self.state.words[word_index].energy = new_energy;
    self.recalculate_visibility_around(word_index);

    Ok(new_energy)
}
```

```rust
// rust/crates/iqrah-core/src/domain/models.rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoRecallWord {
    pub node_id: String,
    pub text: String,
    pub visibility: WordVisibility,
    pub energy: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoRecallStats {
    pub total_words: usize,
    pub visible_count: usize,
    pub obscured_count: usize,
    pub hidden_count: usize,
    pub average_energy: f64,
    pub mastery_percentage: f64,
}
```

```rust
// rust/crates/iqrah-core/src/services/energy_service.rs
pub fn map_energy_to_visibility(
    energy: f64,
    text: &str,
    prev_word_energy: Option<f64>,
    next_word_energy: Option<f64>,
) -> WordVisibility {
    const VISIBLE_THRESHOLD: f64 = 0.15;
    const HIDDEN_THRESHOLD: f64 = 0.85;
    const ANCHOR_THRESHOLD: f64 = 0.3;

    if energy < VISIBLE_THRESHOLD {
        return WordVisibility::Visible;
    }
    if energy >= HIDDEN_THRESHOLD {
        return WordVisibility::Hidden;
    }

    let normalized_energy = (energy - VISIBLE_THRESHOLD) / (HIDDEN_THRESHOLD - VISIBLE_THRESHOLD);
    let coverage = normalized_energy.powf(1.5).clamp(0.0, 1.0);

    let first_char = text.chars().next().unwrap_or('_');
    let last_char = text.chars().last().unwrap_or('_');

    let prev_is_anchor = prev_word_energy.unwrap_or(1.0) >= ANCHOR_THRESHOLD;
    let next_is_anchor = next_word_energy.unwrap_or(1.0) >= ANCHOR_THRESHOLD;

    let hint = match (prev_is_anchor, next_is_anchor) {
        (true, false) => Hint::First { char: first_char },
        (false, true) => Hint::Last { char: last_char },
        (true, true) => Hint::Both { first: first_char, last: last_char },
        (false, false) => Hint::First { char: first_char },
    };

    WordVisibility::Obscured { hint, coverage }
}
```

**Frontend requirements** (design intent):
- [ ] Word-by-word timing tracking
- [ ] Readiness computation per word
- [ ] Blur progression logic
- [ ] Struggle detection support

**Gaps identified**:
1. **CRITICAL**: No FFI binding for EchoRecall creation/submission/state (`EchoRecallExercise`) → cannot start or update sessions.
2. **HIGH**: `EchoRecallState`, `EchoRecallWord`, `WordVisibility`, `EchoRecallStats` are not exposed to Dart.
3. **HIGH**: No `WordTiming` struct or per-word timing history; only a single `recall_time_ms` per submission.
4. **HIGH**: No persistence API for EchoRecall state or energy updates (`finalize()` output not exposed).

---

## 3. Data Model Mismatches

### 3.1 MemoryState

**Rust**:
```rust
// rust/crates/iqrah-core/src/domain/models.rs
pub struct MemoryState {
    pub user_id: String,
    pub node_id: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub review_count: u32,
}
```

**Dart**:
```dart
// lib/rust_bridge/repository.dart
class MemoryState {
  final double stability;
  final double difficulty;
  final double energy;
  final PlatformInt64 lastReviewed;
  final PlatformInt64 dueAt;
  final int reviewCount;
}
```

**Required fixes**:
1. Add `userId` and `nodeId` to Dart `MemoryState` or expose a new DTO that includes them.
2. Convert `lastReviewed` and `dueAt` to DateTime in Dart (or expose DateTime via FRB).
3. Align model usage with `ScoredItem`/session items (See Section 5.1).

### 3.2 ExerciseData vs ExerciseDataDto (WordInstance mismatch)

**Rust**:
```rust
// rust/crates/iqrah-core/src/exercises/exercise_data.rs
pub enum ExerciseData {
    Memorization { node_id: i64 },
    McqArToEn { node_id: i64, distractor_node_ids: Vec<i64> },
    McqEnToAr { node_id: i64, distractor_node_ids: Vec<i64> },
    // ...
    AyahChain { node_id: i64, verse_keys: Vec<String>, current_index: usize, completed_count: usize },
}
```

```rust
// rust/crates/iqrah-core/src/exercises/generators.rs
// MCQ expects WORD_INSTANCE:* ukeys
let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
    kn.base_node_id
} else {
    ukey.to_string()
};
let parts: Vec<&str> = base_ukey.split(':').collect();
if parts.len() != 4 {
    return Err(anyhow::anyhow!("Invalid word ukey format: {}", base_ukey));
}
```

**Dart**:
```dart
// lib/rust_bridge/api.dart
sealed class ExerciseDataDto {
  const factory ExerciseDataDto.memorization({required String nodeId}) = ...;
  const factory ExerciseDataDto.mcqArToEn({required String nodeId, required List<String> distractorNodeIds}) = ...;
  const factory ExerciseDataDto.ayahChain({required String nodeId, required List<String> verseKeys, required BigInt currentIndex, required BigInt completedCount}) = ...;
}
```

**Mismatch**:
- Backend `node_id` values are often `WORD_INSTANCE:chapter:verse:position`, while frontend assumes `WORD:<id>`.
- `ExerciseContentService.fetchTranslation()` treats any `contentKey` containing `:` as a verse key, causing word-instance translations to hit `get_verse_translation_by_translator()` incorrectly.
- `AyahChain` uses `usize` in Rust and maps to `BigInt` in Dart; UI likely expects `int`.

**Required fixes**:
1. Add a Dart helper to parse ukeys and resolve `WORD_INSTANCE` → `word_id` via `get_words_for_verse()`.
2. Route translations based on node type (WORD/WORD_INSTANCE vs VERSE), not `contains(':')`.
3. Normalize `BigInt` to `int` for AyahChain UI logic.

### 3.3 SessionItem

**Rust (closest equivalent)**:
```rust
// rust/crates/iqrah-core/src/services/session_service.rs
pub struct ScoredItem {
    pub node: Node,
    pub memory_state: MemoryState,
    pub priority_score: f64,
    pub days_overdue: f64,
    pub mastery_gap: f64,
    pub knowledge_axis: Option<KnowledgeAxis>,
}
```

**Dart**: ❌ No `SessionItem` class found in `lib/`.

**Required fixes**:
- Expose a `SessionItemDto` (or reuse `ScoredItem`) via FFI, or explicitly document that `ExerciseDataDto` is the only session payload.

### 3.4 ExerciseResult

**Rust**:
```rust
// rust/crates/iqrah-iss/src/exercises/mod.rs
pub struct ExerciseResult {
    pub score: f64,
    pub grade: ReviewGrade,
    pub details: ExerciseDetails,
    pub summary: String,
    pub sampled: usize,
    pub attempted: usize,
    pub unavailable: usize,
    pub availability_ratio: f64,
    pub mean_trials_attempted: Option<f64>,
    pub grade_attempted: Option<ReviewGrade>,
}
```

**Dart**: ❌ No equivalent model found.

**Required fixes**:
- Expose `ExerciseResponse` (core) or `ExerciseResult` (ISS) through FFI if frontend will validate and score answers server-side.

### 3.5 WordTiming

**Rust**: ❌ No `WordTiming` struct found in `iqrah-core`.

**Dart**: ❌ No equivalent model.

**Required fixes**:
- Define a `WordTiming` struct (word_id, start_ts, duration_ms, attempts) if EchoRecall requires word-level timing histories.

### 3.6 EnergySnapshot

**Rust**: ❌ No `EnergySnapshot` struct found.

**Dart**: ❌ No equivalent model.

**Required fixes**:
- Define `EnergySnapshot` DTO with node energy + neighbor energies + propagation deltas for debug visualizations.

### 3.7 NodeFilter

**Rust**: ❌ No `NodeFilter` struct found.

**Dart**: ❌ No equivalent model.

**Required fixes**:
- Add a `NodeFilter` DTO (range/type/energy) and query API to support debug selectors.

---

## 4. Debug Infrastructure Gaps

### 4.1 Node Selector API

**Required functionality**:
```dart
Future<List<String>> queryNodes(NodeFilter filter);
Future<List<String>> parseNodeRange(String range); // "1:1-7"
```

**Backend support (current)**:
```rust
// rust/crates/iqrah-api/src/api.rs
pub async fn search_nodes(query: String, limit: u32) -> Result<Vec<NodeSearchDto>> {
    let all_nodes = app.content_repo.get_all_nodes().await?;
    let results: Vec<_> = all_nodes
        .into_iter()
        .filter(|n| {
            nid::to_ukey(n.id)
                .map(|s| s.starts_with(&query))
                .unwrap_or(false)
        })
        .take(limit as usize)
        .collect();
    // ...
}
```

**Gaps**:
1. **CRITICAL**: `search_nodes` only supports prefix search, no filters or ranges.
2. **HIGH**: `get_all_nodes()` currently returns only verse nodes, so node search is incomplete.
3. **HIGH**: No parsing of verse ranges or node selectors by type/energy.

### 4.2 Energy Propagation Monitor

**Backend support (write-only)**:
```rust
// rust/crates/iqrah-core/src/ports/user_repository.rs
async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()>;
```

**Gaps**:
1. **CRITICAL**: No API to query propagation events or energy snapshots.
2. **HIGH**: No way to retrieve incoming/outgoing edges via FFI.

### 4.3 Database Inspector

**Required functionality**:
```dart
Future<List<Map<String, dynamic>>> executeQuery(String sql);
```

**Backend support**: ❌ None. No SQL or raw query interface exposed via FFI.

**Gaps**:
- Debug tooling cannot inspect DB state from Flutter.

---

## 5. Session Management

### 5.1 Session Flow Analysis

**Backend (current)**:
```rust
// rust/crates/iqrah-api/src/api.rs
pub async fn process_review(user_id: String, node_id: String, grade: u8) -> Result<String> {
    let review_grade = ReviewGrade::from(grade);
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;
    app.learning_service.process_review(&user_id, nid_val, review_grade).await?;
    app.session_service.increment_stat("reviews_today").await?;
    Ok("Review processed".to_string())
}
```

**FFI**:
- `get_exercises()` → returns `ExerciseDataDto` list (no session object)
- `process_review()` → persists grade, updates energy
- `get_session_preview()` → read-only preview
- `clear_session()` → clears session state table

**Frontend (current)**:
```dart
// lib/providers/due_items_provider.dart
final exercisesProvider = FutureProvider.autoDispose<List<api.ExerciseDataDto>>((ref) async {
  return api.getExercises(userId: "test_user", limit: 20, surahFilter: surahFilter, isHighYield: isHighYieldMode);
});
```

```dart
// lib/providers/session_provider.dart
Future<void> submitReview(int grade) async {
  final nodeId = exercise.map(
    memorization: (e) => e.nodeId,
    mcqArToEn: (e) => e.nodeId,
    mcqEnToAr: (e) => e.nodeId,
    // ...
  );
  await api.processReview(userId: "test_user", nodeId: nodeId, grade: grade);
}
```

**Gaps**:
1. **CRITICAL**: No backend validation API; Flutter computes grades locally.
2. **HIGH**: No session start/stop or summary object; only lists of exercises.
3. **HIGH**: Session persistence exists in backend but not exposed (See Section 5.2).

### 5.2 State Persistence

**Backend state storage**:
```sql
-- rust/crates/iqrah-storage/migrations_user/20241126000001_user_schema.sql
CREATE TABLE session_state (
    content_key INTEGER NOT NULL PRIMARY KEY,
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;
```

```rust
// rust/crates/iqrah-core/src/services/session_service.rs
pub async fn get_session_state(&self) -> Result<Vec<i64>> { self.user_repo.get_session_state().await }
pub async fn save_session_state(&self, node_ids: &[i64]) -> Result<()> { self.user_repo.save_session_state(node_ids).await }
```

**Flutter**: ❌ No FFI access to `get_session_state`/`save_session_state`.

**Answer**:
- Session state is persisted in `user.db` (`session_state` table) and would survive app restarts.
- Flutter cannot resume sessions because the API is not exposed.

---

## 6. Database Architecture

### 6.1 Database Overview

**Database 1: Static Content**
- Location: `getApplicationDocumentsDirectory()/content.db` (see `lib/main.dart`)
- Tables: `nodes`, `chapters`, `verses`, `words`, `script_resources`, `script_contents`, `languages`, `translators`, `verse_translations`, `word_translations`, `edges`, `node_goals`, `node_metadata`, `content_packages`, `installed_packages`, etc.
- Access method: FFI only (via `ContentRepository` in Rust)

**Database 2: User State**
- Location: `getApplicationDocumentsDirectory()/user.db` (see `lib/main.dart`)
- Tables: `user_memory_states`, `session_state`, `propagation_events`, `propagation_details`, `user_stats`, `app_settings`, `user_bandit_state`
- Access method: FFI only (via `UserRepository` in Rust)

**Initialization path**:
```dart
// lib/main.dart
final contentDbPath = "$dbDir/content.db";
final userDbPath = "$dbDir/user.db";
await setupDatabase(contentDbPath: contentDbPath, userDbPath: userDbPath, kgBytes: bytes);
```

### 6.2 Debug Access

**Can debug tools query DBs?**
- SQLite accessible from Flutter: ❌ (no `sqflite` usage in repo)
- FFI provides query function: ❌ (no raw SQL API)
- Security/locking concerns: N/A (no direct access)

---

## 7. Priority Gap Summary

### P0 - Critical Blockers (Must fix for EchoRecall)

| Gap | Impact | Location | Effort |
|-----|--------|----------|--------|
| EchoRecall FFI bindings missing | Cannot start or update EchoRecall sessions | `rust/crates/iqrah-api/src/api.rs` | 2-4 hours (NEEDS VERIFICATION) |
| EchoRecall Dart models missing | Cannot render or store EchoRecall state | `lib/rust_bridge/` | 2-3 hours (NEEDS VERIFICATION) |
| WordInstance parsing mismatch | Memorization + MCQs fetch wrong content | `lib/features/exercises/widgets/exercise_container.dart`, `lib/services/exercise_content_service.dart` | 2-3 hours (NEEDS VERIFICATION) |
| Translation routing bug (`contains(':')`) | Word translations routed as verse translations | `lib/services/exercise_content_service.dart` | 1-2 hours (NEEDS VERIFICATION) |

### P1 - High Priority (Needed for multiple exercises)

| Gap | Impact | Location | Effort |
|-----|--------|----------|--------|
| No backend answer validation via FFI | Inconsistent grading, no server-side truth | `rust/crates/iqrah-core/src/exercises/service.rs` + FFI | 3-5 hours (NEEDS VERIFICATION) |
| Missing renderers for 15+ exercise types | Most exercises unplayable | `lib/features/exercises/widgets/` | 2-5 days (NEEDS VERIFICATION) |
| Session persistence not exposed | No resume / continuity | `rust/crates/iqrah-core/src/services/session_service.rs` + FFI | 2-4 hours (NEEDS VERIFICATION) |

### P2 - Medium Priority (Debug tools)

| Gap | Impact | Location | Effort |
|-----|--------|----------|--------|
| Node selector API (filters/range) missing | Debug tooling blocked | `rust/crates/iqrah-api/src/api.rs` | 1-2 days (NEEDS VERIFICATION) |
| Energy snapshot + propagation query missing | Debug energy monitor blocked | `rust/crates/iqrah-core/src/services/learning_service.rs` + FFI | 1-2 days (NEEDS VERIFICATION) |
| SQL inspector missing | No DB inspection | `rust/crates/iqrah-api/src/api.rs` | 1 day (NEEDS VERIFICATION) |

### P3 - Low Priority (Nice to have)

| Gap | Impact | Location | Effort |
|-----|--------|----------|--------|
| `get_available_surahs()` returns empty | UI filter always empty | `rust/crates/iqrah-api/src/api.rs` | 2-4 hours (NEEDS VERIFICATION) |
| `get_debug_stats()` edges count is hardcoded | Misleading debug stats | `rust/crates/iqrah-api/src/api.rs` | 1-2 hours (NEEDS VERIFICATION) |
| Telemetry APIs not exposed via FRB | Missing diagnostics feed | `rust/crates/iqrah-api/src/api.rs` + FRB | 1-2 hours (NEEDS VERIFICATION) |

---

## 8. Recommendations

### 8.1 Immediate Actions (Week 1)

1. **Add missing EchoRecall FFI bindings**
   - Files to modify: `rust/crates/iqrah-api/src/api.rs`, `flutter_rust_bridge.yaml`, regenerate `lib/rust_bridge/*`
   - Also expose `EchoRecallState`, `EchoRecallWord`, `WordVisibility`, `EchoRecallStats`
   - Estimated effort: 4-6 hours (NEEDS VERIFICATION)

2. **Fix word-instance ID handling and translation routing**
   - Update `lib/services/exercise_content_service.dart` to parse `WORD_INSTANCE:chapter:verse:position`
   - Fetch `word_id` via `getWordsForVerse()` and match by position
   - Replace `contains(':')` heuristic with node-type parsing
   - Estimated effort: 3-5 hours (NEEDS VERIFICATION)

3. **Expose answer validation API**
   - Add FFI for `ExerciseService::check_answer()` and `ExerciseResponse`
   - Wire into `ExcercisePage` to compute grades using backend truth
   - Estimated effort: 3-5 hours (NEEDS VERIFICATION)

### 8.2 Architecture Decisions Needed

1. **Database access strategy**
   - Option A: Direct sqflite access from Flutter (faster but riskier concurrency)
   - Option B: FFI-only access (centralized, safer)
   - **Recommendation**: Option B (FFI-only) to keep schema logic and locking in Rust, matching current architecture.

2. **State management for exercises**
   - Current: Riverpod + local timers/grading
   - Recommendation: Continue Riverpod but add per-exercise view models and backend validation hooks to avoid divergent logic.

### 8.3 Risk Assessment

**High Risk**:
- WordInstance/word_id mismatch breaks the only three currently-rendered exercises.
- EchoRecall is entirely blocked without FFI and Dart models.

**Medium Risk**:
- FRB generation mismatch (`rust/src/frb_generated.rs`) may cause confusion or build issues.
- No backend validation for answers risks inconsistent scheduling updates.

---

## 9. Code Reference Map

### 9.1 Critical Files

**Backend**:
```
rust/crates/iqrah-core/src/
├── exercises/
│   ├── echo_recall.rs         [Status: ✅ Implemented]
│   ├── memorization.rs        [Status: ✅ Implemented]
│   ├── mcq.rs                 [Status: ✅ Implemented]
│   └── ...
├── domain/models.rs           [Status: ⚠️ Needs FFI exposure for EchoRecall]
├── services/session_service.rs [Status: ✅ Working, not exposed]
└── services/learning_service.rs [Status: ✅ Working, no debug query API]

rust/crates/iqrah-api/src/
├── api.rs                     [Status: ⚠️ Missing EchoRecall + debug bindings]
├── frb_generated.rs           [Status: ✅ Generated FFI surface]
```

**Frontend**:
```
lib/
├── features/exercises/widgets/exercise_container.dart [Status: ⚠️ Only 3 types + mismatched IDs]
├── providers/session_provider.dart                    [Status: ⚠️ Client-side grading]
├── services/exercise_content_service.dart             [Status: ⚠️ TODOs + routing bug]
├── pages/excercise_page.dart                          [Status: ⚠️ No backend validation]
└── rust_bridge/api.dart                               [Status: ✅ Generated wrappers]
```

**FFI Bindings**:
```
flutter_rust_bridge.yaml                               [Status: ✅ Points to iqrah-api]
lib/rust_bridge/frb_generated.dart                     [Status: ✅ Generated Dart bindings]
```

---

## 10. Appendices

### Appendix A: All Exercise Types (Backend)

Source: `rust/crates/iqrah-core/src/exercises/exercise_data.rs` + `echo_recall.rs`.

1. Memorization - recall exact Arabic word/verse
2. McqArToEn - multiple choice Arabic → English
3. McqEnToAr - multiple choice English → Arabic
4. Translation - type English translation
5. ContextualTranslation - translation with verse context
6. ClozeDeletion - fill missing word in verse
7. FirstLetterHint - recall with first-letter hint
8. MissingWordMcq - MCQ for missing word
9. NextWordMcq - predict next word
10. FullVerseInput - type entire verse
11. AyahChain - sequential verse typing (stateful)
12. FindMistake - identify incorrect word
13. AyahSequence - order verses/words
14. IdentifyRoot - identify root letters
15. ReverseCloze - translate with blank Arabic
16. TranslatePhrase - translate verse/phrase
17. PosTagging - part-of-speech MCQ
18. CrossVerseConnection - thematic connections
19. EchoRecall (separate module) - progressive word obscuring (stateful)
20. MemorizationAyah (separate module) - verse-level energy tracking

⚠️ VERIFICATION NEEDED: Prompt states 17 exercises; current code enumerates 18 in `ExerciseData`, plus EchoRecall and MemorizationAyah as separate stateful exercises.

### Appendix B: FFI Function Signatures

From `rust/crates/iqrah-api/src/api.rs`:

```rust
pub async fn setup_database(content_db_path: String, user_db_path: String, kg_bytes: Vec<u8>) -> Result<String>;
pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String>;

pub async fn get_exercises(user_id: String, limit: u32, _surah_filter: Option<i32>, is_high_yield: bool) -> Result<Vec<ExerciseDataDto>>;
pub async fn get_exercises_for_node(node_id: String) -> Result<Vec<ExerciseDataDto>>;
pub async fn fetch_node_with_metadata(node_id: String) -> Result<Option<NodeData>>;
pub async fn generate_exercise_v2(node_id: String) -> Result<ExerciseDataDto>;

pub async fn get_verse(verse_key: String) -> Result<Option<VerseDto>>;
pub async fn get_word(word_id: i32) -> Result<Option<WordDto>>;
pub async fn get_words_for_verse(verse_key: String) -> Result<Vec<WordDto>>;
pub async fn get_word_translation(word_id: i32, translator_id: i32) -> Result<Option<String>>;

pub async fn process_review(user_id: String, node_id: String, grade: u8) -> Result<String>;
pub async fn get_dashboard_stats(user_id: String) -> Result<DashboardStatsDto>;
pub async fn get_debug_stats(user_id: String) -> Result<DebugStatsDto>;
pub async fn reseed_database(user_id: String) -> Result<String>;
pub async fn get_session_preview(user_id: String, limit: u32, is_high_yield: bool) -> Result<Vec<SessionPreviewDto>>;
pub async fn clear_session() -> Result<String>;

pub async fn search_nodes(query: String, limit: u32) -> Result<Vec<NodeSearchDto>>;
pub async fn get_available_surahs() -> Result<Vec<SurahInfo>>;

pub async fn get_languages() -> Result<Vec<LanguageDto>>;
pub async fn get_translators_for_language(language_code: String) -> Result<Vec<TranslatorDto>>;
pub async fn get_translator(translator_id: i32) -> Result<Option<TranslatorDto>>;

pub async fn get_preferred_translator_id() -> Result<i32>;
pub async fn set_preferred_translator_id(translator_id: i32) -> Result<String>;
pub async fn get_verse_translation_by_translator(verse_key: String, translator_id: i32) -> Result<Option<String>>;

pub fn init_app();

// Telemetry (not exposed via FRB currently)
pub fn drain_telemetry_events() -> Result<Vec<String>>;
pub fn get_telemetry_event_count() -> Result<u32>;
pub fn debug_emit_test_event() -> Result<String>;
```

### Appendix C: Database Schema

**User DB (`user.db`)**:
- `user_memory_states(user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)`
- `session_state(content_key, session_order)`
- `propagation_events(id, source_content_key, event_timestamp)`
- `propagation_details(id, event_id, target_content_key, energy_change, path, reason)`
- `user_stats(key, value)`
- `app_settings(key, value)`
- `user_bandit_state(user_id, goal_group, profile_name, successes, failures, last_updated)`

**Content DB (`content.db`)**:
- `nodes(id, ukey, node_type)`
- `chapters`, `verses`, `words`
- `script_resources`, `script_contents`
- `languages`, `translators`, `verse_translations`, `word_translations`
- `edges`, `node_goals`, `node_metadata`
- `content_packages`, `installed_packages`

---

**End of Report**
