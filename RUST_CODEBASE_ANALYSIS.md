# COMPREHENSIVE RUST CODEBASE STRUCTURE REPORT
## Iqrah Mobile Project

---

## 1. OVERALL PROJECT STRUCTURE

### Package Organization

The Rust codebase is organized as a **monorepo workspace** with 5 crates:

```
rust/
├── Cargo.toml (workspace root)
├── Cargo.lock
├── src/
│   ├── lib.rs (re-exports iqrah-api)
│   └── frb_generated.rs (Flutter bridge auto-generated code)
├── crates/
│   ├── iqrah-core/          # Domain logic, services, CBOR import (28 files)
│   ├── iqrah-storage/       # SQLite repositories, migrations (8 files)
│   ├── iqrah-api/           # Flutter bridge API exports (3 files)
│   ├── iqrah-cli/           # CLI tool for development/testing (6 files)
│   └── iqrah-server/        # Headless test server HTTP/WebSocket (4 files)
├── tests/                   # Integration tests (7 test files)
└── archive/                 # Old code (propagation.rs, etc.)
```

### Crate Dependencies

```
iqrah-api
  ├─ iqrah-core (domain, services)
  ├─ iqrah-storage (repositories)
  └─ flutter_rust_bridge (FFI binding)

iqrah-server
  ├─ iqrah-core
  ├─ iqrah-storage
  ├─ axum (HTTP framework)
  └─ tokio (async runtime)

iqrah-cli
  ├─ iqrah-core
  ├─ iqrah-storage
  ├─ clap (CLI framework)
  ├─ reqwest (HTTP client)
  └─ tokio-tungstenite (WebSocket)

iqrah-storage
  └─ iqrah-core (for domain models)

iqrah-core
  ├─ fsrs (FSRS algorithm v5.1.0)
  ├─ model2vec-rs (semantic embeddings)
  ├─ ciborium (CBOR serialization)
  ├─ zstd (compression)
  └─ sqlx (async database)
```

---

## 2. EXISTING SCHEDULER CODE

### Status: NO EXISTING SCHEDULER v2.0 FOUND

Currently, there is **no dedicated scheduler v2.0** in the codebase. The existing scheduling logic is primitive and located in:

#### **SessionService** (`/home/user/iqrah-mobile/rust/crates/iqrah-core/src/services/session_service.rs` - 203 lines)

**Purpose**: Generate sessions by scoring and prioritizing review items

**Key Method**: `get_due_items()`
- Fetches all **due** memory states from user_repo (where `due_at <= now`)
- Scores each item using weighted criteria:
  - `w_due`: Days overdue
  - `w_need`: Mastery gap (1.0 - energy)
  - `w_yield`: Importance/yield (high-yield mode vs foundational mode)
- Sorts by priority score (highest first)
- Supports Knowledge Axis filtering (Phase 4)

**Limitations**:
- Only considers items that are **already due**
- No predictive scheduling
- No advanced time-based distribution
- No concept of "best study time" or user patterns
- Fixed thresholds (15 items * 3 for filtering)
- Energy-based importance is hardcoded per node type

#### **LearningService** (`/home/user/iqrah-mobile/rust/crates/iqrah-core/src/services/learning_service.rs` - 211 lines)

**Purpose**: Process reviews and update FSRS scheduling

**Key Method**: `process_review(user_id, node_id, grade)`
- Gets or creates memory state
- Updates FSRS parameters (stability, difficulty) based on grade
- Calculates energy delta from review grade
- Propagates energy to related nodes
- Updates due_at timestamp via FSRS algorithm

---

## 3. DATABASE ACCESS PATTERNS

### Database Architecture: Two-Database Design

#### **content.db (READ-ONLY at runtime)**

Location: `/home/user/iqrah-mobile/rust/crates/iqrah-storage/migrations_content/`

Schema (v2 "Purist" approach):
- **Quranic Structure**: chapters, verses, words
- **Morphology**: roots, lemmas, stems, morphology_segments
- **Flexible Content**: languages, translators, verse_translations, word_translations
- **Packages**: content_packages, installed_packages
- **Variants & Audio**: text_variants, word_transliterations, reciters, verse_recitations, word_audio
- **Graph**: edges table (kept from v1 for energy propagation)

**SQL Schema File**: `/home/user/iqrah-mobile/rust/crates/iqrah-storage/migrations_content/20241117000001_content_schema_v2_purist.sql` (418 lines)

#### **user.db (READ-WRITE at runtime)**

Location: `/home/user/iqrah-mobile/rust/crates/iqrah-storage/migrations_user/`

Schema:
- `user_memory_states`: (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
- `propagation_events`: (id, source_content_key, event_timestamp)
- `propagation_details`: (id, event_id, target_content_key, energy_change, path, reason)
- `session_state`: (content_key, session_order) - ephemeral
- `user_stats`: (key, value) - daily counts, streaks
- `app_settings`: (key, value) - app-wide settings

**SQL Schema File**: `/home/user/iqrah-mobile/rust/crates/iqrah-storage/migrations_user/20241116000001_user_schema.sql` (63 lines)

### Repository Pattern

**Port Interface** (`iqrah-core/src/ports/`):
- `ContentRepository`: trait for content queries
- `UserRepository`: trait for user data persistence

**Storage Implementation** (`iqrah-storage/src/`):

**ContentRepository Impl**: `SqliteContentRepository`
```
├── get_node(node_id)
├── get_edges_from(source_id)
├── get_quran_text(node_id)
├── get_translation(node_id, lang)
├── get_metadata(node_id, key)
├── get_all_metadata(node_id)
├── node_exists(node_id)
├── get_all_nodes()
├── get_nodes_by_type(node_type)
├── insert_nodes_batch([]) - CBOR import
├── insert_edges_batch([]) - CBOR import
├── get_chapter(chapter_number)
├── get_verse(verse_key)
├── get_words_for_verse(verse_key)
├── get_languages()
├── get_translators_for_language(lang_code)
└── ... (25+ methods total)
```

**UserRepository Impl**: `SqliteUserRepository`
```
├── get_memory_state(user_id, node_id)
├── save_memory_state(state)
├── get_due_states(user_id, due_before, limit) ⭐ CRITICAL FOR SCHEDULER
├── update_energy(user_id, node_id, new_energy)
├── log_propagation(event)
├── get_session_state()
├── save_session_state(node_ids[])
├── clear_session_state()
├── get_stat(key)
├── set_stat(key, value)
├── get_setting(key)
└── set_setting(key, value)
```

### Database Initialization

**Content DB Init**: `iqrah-storage/src/content/mod.rs`
```rust
pub async fn init_content_db(path: &str) -> Result<SqlitePool>
```

**User DB Init**: `iqrah-storage/src/user/mod.rs`
```rust
pub async fn init_user_db(path: &str) -> Result<SqlitePool>
```

Both use `sqlx` with compile-time checked migrations.

---

## 4. CLI STRUCTURE

### Location
`/home/user/iqrah-mobile/rust/crates/iqrah-cli/src/`

### Architecture

**Binary**: `iqrah` (defined in Cargo.toml as [[bin]] name = "iqrah")

**CLI Framework**: `clap` with derive macros

### Command Structure

```
iqrah [--server URL] <COMMAND>

Commands:
├── debug <DEBUG_COMMAND>
│   ├── get-node <NODE_ID>
│   ├── get-state <USER_ID> <NODE_ID>
│   ├── set-state <USER_ID> <NODE_ID> --energy <FLOAT>
│   └── process-review <USER_ID> <NODE_ID> <GRADE>
│
├── import <CBOR_FILE>
│   └── Imports CBOR graph file into local database
│
├── exercise <EXERCISE_COMMAND>
│   ├── run <TYPE> <NODE_ID>
│   ├── start <TYPE> <AYAH_IDS...>
│   ├── action <TYPE> <SESSION_ID> <WORD_NODE_ID> <RECALL_TIME_MS>
│   └── end <SESSION_ID>
│
├── translator <TRANSLATOR_COMMAND>
│   ├── list-languages
│   ├── list-translators <LANG_CODE>
│   ├── get-translator <ID>
│   ├── get-preferred <USER_ID>
│   ├── set-preferred <USER_ID> <ID>
│   ├── get-translation <VERSE_KEY> <TRANSLATOR_ID>
│   └── import <METADATA_FILE> <TRANSLATIONS_BASE>
│
└── package <PACKAGE_COMMAND>
    ├── list
    ├── get <PACKAGE_ID>
    ├── list-installed
    ├── install <PACKAGE_ID>
    ├── uninstall <PACKAGE_ID>
    ├── enable <PACKAGE_ID>
    └── disable <PACKAGE_ID>
```

### Modules

**debug.rs**: HTTP calls to debug endpoints
**exercise.rs**: WebSocket-based exercise session management
**import.rs**: CBOR file import via HTTP
**translator.rs**: Translator management
**package.rs**: Package installation/management
**main.rs**: CLI argument parsing and command routing

### How to Add a New Command

1. Define enum variant in `enum Commands` and respective `enum XxxCommands`
2. Implement handler function in corresponding module (e.g., `debug::handler()`)
3. Add match arm in `main()` to route command to handler
4. Handler makes HTTP/WebSocket calls to server

---

## 5. DATA MODELS

### Core Domain Models

**Location**: `/home/user/iqrah-mobile/rust/crates/iqrah-core/src/domain/models.rs` (500+ lines)

#### **Node Type System**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum NodeType {
    Root,           // Morphological root
    Lemma,          // Dictionary headword
    Word,           // Generic word (deprecated?)
    WordInstance,   // Word in a specific verse
    Verse,          // Ayah (verse)
    Chapter,        // Surah (chapter)
    Knowledge,      // Phase 4: Multi-dimensional learning nodes
}
```

#### **Knowledge Axis** (Phase 4)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowledgeAxis {
    Memorization,
    Translation,
    Tafsir,
    Tajweed,
    ContextualMemorization,
    Meaning,
}

pub struct KnowledgeNode {
    pub base_node_id: String,      // "VERSE:1:1"
    pub axis: KnowledgeAxis,        // "Memorization"
    pub full_id: String,            // "VERSE:1:1:memorization"
}
```

#### **Node & Edge**

```rust
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub knowledge_node: Option<KnowledgeNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
}

pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,  // Const, Normal, Beta
    pub param1: f64,
    pub param2: f64,
}
```

#### **Memory State** (FSRS + Energy)

```rust
pub struct MemoryState {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,         // FSRS stability parameter
    pub difficulty: f64,        // FSRS difficulty parameter
    pub energy: f64,            // Custom: 0.0-1.0 mastery level
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,  // When to review next
    pub review_count: u32,
}
```

#### **Review Grades**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewGrade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}
```

#### **V2 Domain Models** (Purist/Relational)

```rust
pub struct Chapter {
    pub number: i32,
    pub name_arabic: String,
    pub name_transliteration: String,
    pub name_translation: String,
    pub revelation_place: Option<String>,
    pub verse_count: i32,
}

pub struct Verse {
    pub key: String,                // "1:1", "2:255"
    pub chapter_number: i32,
    pub verse_number: i32,
    pub text_uthmani: String,
    pub text_simple: Option<String>,
    pub juz: i32,
    pub page: i32,
}

pub struct Word {
    pub id: i32,
    pub verse_key: String,
    pub position: i32,              // 1-indexed position within verse
    pub text_uthmani: String,
    pub text_simple: Option<String>,
    pub transliteration: Option<String>,
}

pub struct Language {
    pub code: String,               // ISO 639-1
    pub english_name: String,
    pub native_name: String,
    pub direction: String,          // 'ltr' or 'rtl'
}

pub struct Translator {
    pub id: i32,
    pub slug: String,
    pub full_name: String,
    pub language_code: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub package_id: Option<String>,
}
```

#### **Exercise Models**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Hint {
    First { char: char },
    Last { char: char },
    Both { first: char, last: char },
}

pub enum WordVisibility {
    Visible,
    Obscured { hint: Hint, coverage: f64 },  // 0.0-1.0
    Hidden,
}

pub struct EchoRecallWord {
    pub node_id: String,
    pub text: String,
    pub visibility: WordVisibility,
    pub energy: f64,
}

pub struct EchoRecallState {
    pub words: Vec<EchoRecallWord>,
}
```

#### **Propagation Models**

```rust
pub struct PropagationEvent {
    pub source_node_id: String,
    pub event_timestamp: DateTime<Utc>,
    pub details: Vec<PropagationDetail>,
}

pub struct PropagationDetail {
    pub target_node_id: String,
    pub energy_change: f64,
    pub reason: String,
}
```

#### **Package Management**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageType {
    VerseTranslation,
    WordTranslation,
    TextVariant,
    VerseRecitation,
    WordAudio,
    Transliteration,
}

pub struct ContentPackage {
    pub package_id: String,
    pub package_type: PackageType,
    pub name: String,
    pub language_code: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub file_size: Option<i64>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub license: Option<String>,
}

pub struct InstalledPackage {
    pub package_id: String,
    pub installed_at: DateTime<Utc>,
    pub enabled: bool,
}
```

---

## 6. TESTING PATTERNS

### Test Infrastructure

**Test Files Location**:
- **Unit tests**: Embedded in `.rs` files with `#[test]` and `#[cfg(test)]`
- **Integration tests**: `/home/user/iqrah-mobile/rust/tests/` (7 files)
- **Crate-specific tests**: `/home/user/iqrah-mobile/rust/crates/*/tests/` (1 file)

### Testing Frameworks Used

- `#[tokio::test]` - Async tests with tokio runtime
- `rstest` - Parameterized tests
- `mockall` - Mock objects
- `proptest` - Property-based testing
- `tempfile` - Temporary files for tests
- `tokio-test` - Testing utilities

### Test Coverage (24 tests passing)

**iqrah-core tests**:
- `semantic/tests.rs` - Semantic grading tests
- `services/learning_service_tests.rs` - 544 lines, comprehensive FSRS/energy tests
- `services/session_service_tests.rs` - 742 lines, session generation tests

**iqrah-storage tests**:
- `tests/integration_tests.rs` - 150+ lines, repository CRUD tests

**Top-level integration tests**:
- `tests/import_test.rs` - CBOR import validation
- `tests/knowledge_axis_test.rs` - Knowledge axis parsing
- `tests/translator_import_test.rs` - Translator import
- `tests/propagation_tests.rs` - Energy propagation
- `tests/semantic_grading_integration_test.rs` - Semantic model
- `tests/package_test.rs` - Package management
- `tests/common/mod.rs` - Test utilities

### Test Setup Pattern

```rust
#[tokio::test]
async fn test_xxx() {
    // 1. Create in-memory DB
    let pool = init_user_db(":memory:").await.unwrap();
    
    // 2. Create repository
    let repo = SqliteUserRepository::new(pool);
    
    // 3. Test operations
    repo.save_memory_state(&state).await.unwrap();
    
    // 4. Assert results
    assert_eq!(retrieved.energy, 0.9);
}
```

### Running Tests

```bash
cd rust
cargo test --all-features                    # All tests
cargo test --package iqrah-core              # Specific crate
cargo test learning_service                  # Specific test
cargo test -- --test-threads=1 --nocapture   # Single-threaded with output
```

### Key Test Examples

**SessionService Tests** (session_service_tests.rs):
- `test_scoring_weights_change_with_mode()` - High-yield vs foundational
- `test_priority_calculation_uses_correct_weights()` - Score formula
- `test_knowledge_axis_filtering()` - Phase 4 axis filtering
- `test_stat_operations()` - User stats CRUD

**LearningService Tests** (learning_service_tests.rs):
- `test_fsrs_state_updates_on_review()` - FSRS updates
- `test_energy_delta_calculation()` - Energy change algorithm
- `test_propagation_on_energy_change()` - Graph propagation
- `test_process_review_integration()` - Full review flow

---

## 7. CRITICAL FILE PATHS & LINE COUNTS

### Core Logic Files

```
iqrah-core/src/
├── lib.rs (58 lines) - Main exports
├── domain/
│   ├── models.rs (500+ lines) ⭐ DATA MODELS
│   ├── errors.rs
│   └── mod.rs
├── services/
│   ├── session_service.rs (203 lines) ⭐ SESSION SCHEDULING
│   ├── learning_service.rs (211 lines) ⭐ REVIEW PROCESSING
│   ├── session_service_tests.rs (742 lines)
│   ├── learning_service_tests.rs (544 lines)
│   ├── energy_service.rs (271 lines) - Echo Recall visibility mapping
│   ├── recall_model.rs (148 lines) - Recall time -> energy delta
│   ├── package_service.rs (645 lines) - Package management
│   └── mod.rs (15 lines)
├── exercises/
│   ├── service.rs - Exercise generation
│   ├── mcq.rs - Multiple choice
│   ├── memorization.rs
│   ├── translation.rs
│   ├── types.rs
│   └── mod.rs
├── ports/
│   ├── content_repository.rs ⭐ REPOSITORY TRAIT
│   ├── user_repository.rs ⭐ REPOSITORY TRAIT
│   └── mod.rs
├── semantic/
│   ├── embedding.rs - model2vec integration
│   ├── grader.rs - Semantic grading
│   ├── tests.rs
│   └── mod.rs
├── cbor_import.rs (9186 bytes) ⭐ CBOR IMPORT
└── import/
    ├── translator_import.rs
    └── mod.rs
```

### Storage/Repository Files

```
iqrah-storage/src/
├── lib.rs (10 lines) - Exports
├── content/
│   ├── repository.rs (400+ lines) ⭐ CONTENT REPO IMPL
│   ├── models.rs - SQL row types
│   └── mod.rs
├── user/
│   ├── repository.rs (200+ lines) ⭐ USER REPO IMPL
│   ├── models.rs - SQL row types
│   └── mod.rs
└── migrations/
    └── mod.rs
```

### Migration Files

```
migrations_content/
├── 20241116000001_content_schema_v1_archived.sql (archived)
└── 20241117000001_content_schema_v2_purist.sql (418 lines) ⭐ MAIN SCHEMA

migrations_user/
├── 20241116000001_user_schema.sql (63 lines) ⭐ MAIN SCHEMA
├── 20241116000002_initialize_settings.sql
└── 20241117000001_content_keys.sql
```

### API/Server Files

```
iqrah-api/src/
├── api.rs (200+ lines) ⭐ FLUTTER BRIDGE API
├── lib.rs (10 lines)
└── frb_generated.rs (84KB, auto-generated)

iqrah-server/src/
├── main.rs (126 lines) ⭐ SERVER SETUP
├── http.rs (18.6KB) - HTTP routes
├── websocket.rs (25KB) - WebSocket handlers
└── protocol.rs (3.9KB) - Message protocol

iqrah-cli/src/
├── main.rs (327 lines) ⭐ CLI COMMAND ROUTING
├── debug.rs - Debug commands
├── exercise.rs - Exercise session commands
├── import.rs - CBOR import commands
├── translator.rs - Translator commands
└── package.rs - Package commands
```

---

## 8. KEY OBSERVATIONS FOR SCHEDULER v2.0 IMPLEMENTATION

### Existing Architecture That Will Help

1. **Repository Pattern**: Clean abstraction via `ContentRepository` and `UserRepository` traits
   - Easy to swap implementations (already in place for SQLite)
   - `get_due_states()` method is the key data accessor for scheduling

2. **FSRS Integration**: Already using `fsrs` crate v5.1.0
   - Calculates next review interval based on stability/difficulty/grade
   - Updates `due_at` timestamp automatically
   - Can be extended for v2.0 features

3. **Energy System**: Custom layer on top of FSRS
   - Separate `energy` field (0.0-1.0) for mastery tracking
   - Propagates through graph edges
   - Independent of scheduling but influences priority

4. **Knowledge Axis**: Phase 4 support for multi-dimensional learning
   - Can filter sessions by learning dimension
   - Already in data models and session service

5. **Testing Infrastructure**: Solid test suite with in-memory DB support
   - Can write comprehensive scheduler tests
   - Mocking via `mockall` available

### Gaps That Scheduler v2.0 Should Fill

1. **Only considers items already due**: Need predictive scheduling
2. **No intelligent time-of-day selection**: Could optimize for user's peak learning times
3. **No spaced distribution**: Doesn't spread reviews throughout day
4. **No user adaptation**: Fixed weights, doesn't learn from user behavior
5. **No long-term planning**: Doesn't forecast future workload
6. **Limited filtering/personalization**: Basic mode toggle only
7. **No priority rebalancing**: Doesn't adjust weights based on user performance

---

## 9. SUMMARY TABLE

| Aspect | Location | Lines | Notes |
|--------|----------|-------|-------|
| **Data Models** | `iqrah-core/src/domain/models.rs` | 500+ | Node, Edge, MemoryState, KnowledgeAxis, etc. |
| **SessionService** | `iqrah-core/src/services/session_service.rs` | 203 | Current scheduler (basic) |
| **LearningService** | `iqrah-core/src/services/learning_service.rs` | 211 | FSRS + energy processing |
| **Content Schema** | `migrations_content/20241117...sql` | 418 | Purist relational schema |
| **User Schema** | `migrations_user/20241116...sql` | 63 | Memory states, stats, settings |
| **Content Repo** | `iqrah-storage/src/content/repository.rs` | 400+ | Query implementation |
| **User Repo** | `iqrah-storage/src/user/repository.rs` | 200+ | State persistence |
| **API Bridge** | `iqrah-api/src/api.rs` | 200+ | Flutter FFI entry points |
| **Server HTTP** | `iqrah-server/src/http.rs` | 18.6KB | REST endpoints |
| **Server WebSocket** | `iqrah-server/src/websocket.rs` | 25KB | Real-time communication |
| **CLI** | `iqrah-cli/src/main.rs` | 327 | Development tool |
| **Test Suite** | Various | 2000+ | Unit + integration tests |

