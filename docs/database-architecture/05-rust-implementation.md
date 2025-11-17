# Rust Implementation: Module Responsibilities

## Overview

The Rust codebase follows a **clean architecture** pattern with clear separation between:
- **Domain** (models, business logic)
- **Ports** (repository traits, abstractions)
- **Adapters** (concrete implementations)
- **Services** (application logic)

## Crate Structure

```
rust/crates/
├── iqrah-storage/     # Data persistence layer (adapters)
├── iqrah-core/        # Business logic (domain + services)
└── iqrah-cli/         # Command-line interface (entry point)
```

## iqrah-storage Crate

**Purpose:** Data persistence and database management.

**Location:** [rust/crates/iqrah-storage/](../../rust/crates/iqrah-storage/)

### Directory Structure

```
iqrah-storage/
├── src/
│   ├── lib.rs
│   ├── content/
│   │   ├── mod.rs           # Content DB initialization
│   │   ├── repository.rs    # SqliteContentRepository implementation
│   │   └── models.rs        # Database row models
│   ├── user/
│   │   ├── mod.rs           # User DB initialization
│   │   ├── repository.rs    # SqliteUserRepository implementation
│   │   └── models.rs        # Database row models
│   └── migrations/
│       └── mod.rs           # Migration utilities (placeholder)
├── migrations_content/
│   └── 20241116000001_content_schema.sql
└── migrations_user/
    ├── 20241116000001_user_schema.sql
    └── 20241116000002_initialize_settings.sql
```

### Key Modules

#### content/mod.rs

**Purpose:** Content database initialization and configuration.

**Location:** Lines 1-19

**Key Function:**
```rust
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

**Responsibilities:**
- Create connection pool for content.db
- Run migrations
- Return pool for repository instantiation

#### content/repository.rs

**Purpose:** Implement ContentRepository trait with SQLite.

**Location:** Lines 7-315

**Class:** `SqliteContentRepository`

**Key Methods:**

| Method | Lines | Purpose |
|--------|-------|---------|
| `new(pool)` | 7-15 | Constructor |
| `get_node(node_id)` | 17-29 | Fetch single node by ID |
| `get_nodes(node_ids)` | 31-45 | Batch fetch nodes |
| `get_quran_text(node_id)` | 57-65 | Get Arabic text |
| `get_translation(node_id, lang)` | 67-79 | Get translation |
| `get_edges_from(source_id)` | 81-95 | Get outgoing edges (for propagation) |
| `insert_nodes_batch(nodes)` | 97-127 | Bulk insert from CBOR import |
| `insert_edges_batch(edges)` | 129-159 | Bulk insert from CBOR import |
| `get_adjacent_words(word_id)` | 256-314 | Get previous/next word (ID inference) |
| `get_words_in_ayahs(verse_keys)` | 161-195 | Get all words for verses (LIKE pattern) |

**Implementation Details:**

**Simple Query Example:**
```rust
// Lines 17-29
async fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
    let row = query_as::<_, NodeRow>(
        "SELECT id, node_type, created_at FROM nodes WHERE id = ?"
    )
    .bind(node_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(row.map(|r| r.into()))  // Convert NodeRow → Node
}
```

**Batch Insert Example:**
```rust
// Lines 97-127 (simplified)
async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    for node in nodes {
        query("INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)")
            .bind(&node.id)
            .bind(&node.node_type.to_string())
            .bind(Utc::now().timestamp())
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

**Graph Traversal Example:**
```rust
// Lines 81-95
async fn get_edges_from(&self, source_id: &str) -> Result<Vec<Edge>> {
    let rows = query_as::<_, EdgeRow>(
        "SELECT source_id, target_id, edge_type, distribution_type, param1, param2
         FROM edges WHERE source_id = ?"
    )
    .bind(source_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}
```

#### content/models.rs

**Purpose:** Database row types (serialization/deserialization).

**Row Types:**
```rust
#[derive(sqlx::FromRow)]
pub struct NodeRow {
    pub id: String,
    pub node_type: String,
    pub created_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct EdgeRow {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: i32,
    pub distribution_type: i32,
    pub param1: Option<f64>,
    pub param2: Option<f64>,
}

#[derive(sqlx::FromRow)]
pub struct QuranTextRow {
    pub node_id: String,
    pub arabic: String,
}

#[derive(sqlx::FromRow)]
pub struct TranslationRow {
    pub node_id: String,
    pub language_code: String,
    pub translation: String,
}
```

**Conversion to Domain Models:**
```rust
impl From<NodeRow> for Node {
    fn from(row: NodeRow) -> Self {
        Node {
            id: row.id,
            node_type: NodeType::from_str(&row.node_type),
            created_at: row.created_at,
        }
    }
}
```

#### user/mod.rs

**Purpose:** User database initialization.

**Location:** Lines 1-19

**Key Function:**
```rust
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_user")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

#### user/repository.rs

**Purpose:** Implement UserRepository trait with SQLite.

**Location:** Lines 7-199

**Class:** `SqliteUserRepository`

**Key Methods:**

| Method | Lines | Purpose |
|--------|-------|---------|
| `new(pool)` | 7-15 | Constructor |
| `get_memory_state(user_id, node_id)` | 17-35 | Get FSRS + energy state |
| `save_memory_state(state)` | 37-61 | Save/update memory state |
| `get_due_states(user_id, limit)` | 63-89 | Get nodes due for review |
| `update_energy(node_id, change)` | 91-115 | Apply energy delta |
| `log_propagation(event)` | 117-145 | Audit trail for energy changes |
| `save_session_state(items)` | 147-171 | Store session for resume |
| `get_session_state()` | 173-189 | Retrieve session state |
| `clear_session_state()` | 191-199 | Clean up after session |

**Critical Implementation: Due Items Query**
```rust
// Lines 63-89
async fn get_due_states(&self, user_id: &str, limit: i64) -> Result<Vec<MemoryState>> {
    let now = Utc::now().timestamp();

    let rows = query_as::<_, MemoryStateRow>(
        "SELECT user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count
         FROM user_memory_states
         WHERE user_id = ? AND due_at <= ?
         ORDER BY due_at ASC
         LIMIT ?"
    )
    .bind(user_id)
    .bind(now)
    .bind(limit)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}
```

**Energy Propagation Logging:**
```rust
// Lines 117-145 (simplified)
async fn log_propagation(&self, event: &PropagationEvent) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    // Insert event header
    let event_id = query(
        "INSERT INTO propagation_events (source_node_id, event_timestamp) VALUES (?, ?)"
    )
    .bind(&event.source_node_id)
    .bind(event.timestamp)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // Insert details
    for detail in &event.details {
        query(
            "INSERT INTO propagation_details (event_id, target_node_id, energy_change, reason)
             VALUES (?, ?, ?, ?)"
        )
        .bind(event_id)
        .bind(&detail.target_node_id)
        .bind(detail.energy_change)
        .bind(&detail.reason)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

#### user/models.rs

**Purpose:** User database row types.

**Row Types:**
```rust
#[derive(sqlx::FromRow)]
pub struct MemoryStateRow {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: Option<i64>,
    pub due_at: Option<i64>,
    pub review_count: i32,
}

#[derive(sqlx::FromRow)]
pub struct PropagationEventRow {
    pub id: i64,
    pub source_node_id: String,
    pub event_timestamp: i64,
}

#[derive(sqlx::FromRow)]
pub struct PropagationDetailRow {
    pub event_id: i64,
    pub target_node_id: String,
    pub energy_change: f64,
    pub reason: Option<String>,
}
```

## iqrah-core Crate

**Purpose:** Business logic and application services.

**Location:** [rust/crates/iqrah-core/](../../rust/crates/iqrah-core/)

### Directory Structure

```
iqrah-core/
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   └── models.rs        # Domain entities (Node, Edge, MemoryState)
│   ├── ports/
│   │   ├── content_repository.rs  # ContentRepository trait
│   │   └── user_repository.rs     # UserRepository trait
│   ├── services/
│   │   ├── learning_service.rs    # Review processing + propagation
│   │   ├── session_service.rs     # Session generation
│   │   └── energy_service.rs      # Energy calculations
│   └── cbor_import.rs       # Graph import from Python
```

### Key Modules

#### domain/models.rs

**Purpose:** Core domain entities (repository-agnostic).

**Location:** Lines 1-100+

**Key Types:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Root,
    Lemma,
    Word,
    WordInstance,
    Verse,
    Chapter,
    Knowledge,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,
    pub param1: Option<f64>,
    pub param2: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
}

#[derive(Debug, Clone)]
pub enum DistributionType {
    Const = 0,
    Normal = 1,
    Beta = 2,
}

#[derive(Debug, Clone)]
pub struct MemoryState {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: Option<i64>,
    pub due_at: Option<i64>,
    pub review_count: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum ReviewGrade {
    Again = 0,
    Hard = 1,
    Good = 2,
    Easy = 3,
}
```

#### ports/content_repository.rs

**Purpose:** Define interface for content storage (dependency inversion).

**Trait:**
```rust
#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn get_node(&self, node_id: &str) -> Result<Option<Node>>;
    async fn get_nodes(&self, node_ids: &[String]) -> Result<Vec<Node>>;
    async fn get_quran_text(&self, node_id: &str) -> Result<Option<String>>;
    async fn get_translation(&self, node_id: &str, language_code: &str) -> Result<Option<String>>;
    async fn get_edges_from(&self, source_id: &str) -> Result<Vec<Edge>>;
    async fn get_adjacent_words(&self, word_node_id: &str) -> Result<(Option<Node>, Option<Node>)>;
    async fn get_words_in_ayahs(&self, verse_keys: &[String]) -> Result<Vec<Node>>;

    // Bulk insert for CBOR import
    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> Result<()>;
    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> Result<()>;
}
```

**Benefits:**
- Services depend on trait, not concrete implementation
- Easy to mock for testing
- Can swap SQLite for PostgreSQL without changing services

#### ports/user_repository.rs

**Purpose:** Define interface for user storage.

**Trait:**
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_memory_state(&self, user_id: &str, node_id: &str) -> Result<Option<MemoryState>>;
    async fn save_memory_state(&self, state: &MemoryState) -> Result<()>;
    async fn get_due_states(&self, user_id: &str, limit: i64) -> Result<Vec<MemoryState>>;
    async fn update_energy(&self, node_id: &str, energy_change: f64) -> Result<()>;
    async fn log_propagation(&self, event: &PropagationEvent) -> Result<()>;

    async fn save_session_state(&self, items: &[SessionStateItem]) -> Result<()>;
    async fn get_session_state(&self) -> Result<Vec<SessionStateItem>>;
    async fn clear_session_state(&self) -> Result<()>;
}
```

#### services/learning_service.rs

**Purpose:** Process reviews and propagate energy.

**Location:** Lines 10-150+

**Class:** `LearningService`

**Key Method:**
```rust
pub async fn process_review(
    &self,
    user_id: &str,
    node_id: &str,
    grade: ReviewGrade,
) -> Result<ReviewResult> {
    // 1. Get current memory state
    let mut state = self.user_repo.get_memory_state(user_id, node_id).await?
        .unwrap_or_else(|| MemoryState::new(user_id, node_id));

    // 2. Update FSRS parameters
    let fsrs = FSRS::new()?;
    let card = state.to_fsrs_card();
    let scheduling = fsrs.schedule(card, grade)?;

    state.stability = scheduling.stability;
    state.difficulty = scheduling.difficulty;
    state.due_at = scheduling.due_at;
    state.review_count += 1;
    state.last_reviewed = Some(Utc::now().timestamp());

    // 3. Save state
    self.user_repo.save_memory_state(&state).await?;

    // 4. Propagate energy
    self.propagate_energy(user_id, node_id, state.energy).await?;

    Ok(ReviewResult {
        new_stability: state.stability,
        new_due_at: state.due_at,
    })
}
```

**Energy Propagation:**
```rust
async fn propagate_energy(
    &self,
    user_id: &str,
    source_node_id: &str,
    source_energy: f64,
) -> Result<()> {
    // Get outgoing edges
    let edges = self.content_repo.get_edges_from(source_node_id).await?;

    let mut propagation_event = PropagationEvent {
        source_node_id: source_node_id.to_string(),
        timestamp: Utc::now().timestamp(),
        details: Vec::new(),
    };

    for edge in edges {
        // Calculate energy transfer based on distribution
        let energy_change = self.calculate_energy_transfer(
            source_energy,
            edge.distribution_type,
            edge.param1,
            edge.param2,
        );

        // Update target node energy
        self.user_repo.update_energy(&edge.target_id, energy_change).await?;

        // Log detail
        propagation_event.details.push(PropagationDetail {
            target_node_id: edge.target_id.clone(),
            energy_change,
            reason: format!("{:?} edge", edge.edge_type),
        });
    }

    // Log propagation event
    self.user_repo.log_propagation(&propagation_event).await?;

    Ok(())
}
```

#### services/session_service.rs

**Purpose:** Generate review sessions with priority scoring.

**Location:** Lines 38-171

**Class:** `SessionService`

**Key Method:**
```rust
pub async fn get_due_items(
    &self,
    user_id: &str,
    limit: i64,
) -> Result<Vec<SessionItem>> {
    // 1. Get due memory states
    let due_states = self.user_repo.get_due_states(user_id, limit * 2).await?;

    // 2. Score and filter
    let mut items = Vec::new();
    for state in due_states {
        let node = self.content_repo.get_node(&state.node_id).await?;

        // Filter: Only WordInstance and Verse types
        if !matches!(node.node_type, NodeType::WordInstance | NodeType::Verse) {
            continue;
        }

        // Calculate priority score
        let score = self.calculate_priority(&state, &node);

        items.push(SessionItem {
            node_id: state.node_id,
            node_type: node.node_type,
            energy: state.energy,
            priority_score: score,
        });
    }

    // 3. Sort by priority
    items.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());

    // 4. Take top N
    items.truncate(limit as usize);

    Ok(items)
}
```

**Priority Scoring:**
```rust
fn calculate_priority(&self, state: &MemoryState, node: &Node) -> f64 {
    let now = Utc::now().timestamp();
    let days_overdue = ((now - state.due_at.unwrap_or(now)) as f64) / 86400.0;
    let mastery_gap = 1.0 - state.energy;

    // Weighted scoring
    let w_due = 0.5;
    let w_need = 0.3;
    let w_yield = 0.2;

    w_due * days_overdue + w_need * mastery_gap + w_yield * state.energy
}
```

#### services/energy_service.rs

**Purpose:** Energy-related calculations.

**Functions:**
- `map_energy_to_visibility(energy)` - Convert energy (0-1) to opacity (0-100)
- `calculate_mastery_level(energy)` - Categorize as "new", "learning", "known", "mastered"

#### cbor_import.rs

**Purpose:** Import graph from Python-generated CBOR file.

**Location:** Lines 98-226

**Key Function:**
```rust
pub async fn import_cbor_graph_from_bytes(
    cbor_bytes: &[u8],
    content_repo: &dyn ContentRepository,
) -> Result<ImportStats> {
    // 1. Deserialize CBOR
    let records: Vec<CborRecord> = serde_cbor::from_slice(cbor_bytes)?;

    // 2. Separate nodes and edges
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for record in records {
        match record {
            CborRecord::Node { id, a } => nodes.push(ImportedNode { id, ... }),
            CborRecord::Edge { ... } => edges.push(ImportedEdge { ... }),
        }
    }

    // 3. Batch insert
    content_repo.insert_nodes_batch(&nodes).await?;
    content_repo.insert_edges_batch(&edges).await?;

    Ok(ImportStats {
        nodes_imported: nodes.len(),
        edges_imported: edges.len(),
    })
}
```

## Clean Architecture Flow

```
┌─────────────────────────────────────────────────────┐
│                   CLI Layer                         │
│         (iqrah-cli: commands, UI)                   │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│                Services Layer                       │
│  (iqrah-core/services: LearningService, etc.)       │
└──────────┬──────────────────────────────────────────┘
           │
           │ Depends on traits (ports), not implementations
           │
           ▼
┌─────────────────────────────────────────────────────┐
│              Ports Layer (Traits)                   │
│  (iqrah-core/ports: ContentRepository trait, etc.)  │
└──────────┬──────────────────────────────────────────┘
           │
           │ Implemented by adapters
           │
           ▼
┌─────────────────────────────────────────────────────┐
│            Adapters Layer                           │
│  (iqrah-storage: SqliteContentRepository, etc.)     │
└──────────┬──────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│               Database Layer                        │
│         (SQLite: content.db, user.db)               │
└─────────────────────────────────────────────────────┘
```

**Benefits:**
- Domain logic independent of database
- Easy to test services with mock repositories
- Can swap storage implementations without touching business logic

## File Reference Summary

**iqrah-storage:**
- [src/content/mod.rs](../../rust/crates/iqrah-storage/src/content/mod.rs) - Content DB init
- [src/content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) - ContentRepository impl (lines 7-315)
- [src/user/mod.rs](../../rust/crates/iqrah-storage/src/user/mod.rs) - User DB init
- [src/user/repository.rs](../../rust/crates/iqrah-storage/src/user/repository.rs) - UserRepository impl (lines 7-199)

**iqrah-core:**
- [src/domain/models.rs](../../rust/crates/iqrah-core/src/domain/models.rs) - Domain entities
- [src/ports/content_repository.rs](../../rust/crates/iqrah-core/src/ports/content_repository.rs) - ContentRepository trait
- [src/ports/user_repository.rs](../../rust/crates/iqrah-core/src/ports/user_repository.rs) - UserRepository trait
- [src/services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs) - Review processing (lines 10-150+)
- [src/services/session_service.rs](../../rust/crates/iqrah-core/src/services/session_service.rs) - Session generation (lines 38-171)
- [src/cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs) - CBOR import (lines 98-226)

---

**Navigation:** [← Database Interactions](04-database-interactions.md) | [Next: Knowledge Axis Design →](06-knowledge-axis-design.md)
