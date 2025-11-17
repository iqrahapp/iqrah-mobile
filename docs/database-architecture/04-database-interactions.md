# Database Interactions

**Related Questions:**
- Q2 - How do Knowledge Graph and Content DB connect?
- Q5 - One database file or multiple?

## Overview

The Iqrah application uses **TWO separate SQLite database files** that interact through the application layer but have **no direct cross-database joins**.

## Q5: Database File Architecture

### Two Files

```
User's Device:
/data/app/iqrah/
├── content.db    (Read-only, shipped with app)
└── user.db       (Read-write, created on device)
```

**Why separate files?**

| Reason | Benefit |
|--------|---------|
| **Update isolation** | Can replace content.db without touching user data |
| **Backup strategy** | Users only need to backup small user.db file |
| **Performance** | Content queries don't lock user writes |
| **Security** | Different file permissions (content = read-only) |
| **Size management** | Large content.db shipped once, small user.db grows slowly |

### Initialization

**Content DB:**
```rust
// rust/crates/iqrah-storage/src/content/mod.rs:10-19
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

**User DB:**
```rust
// rust/crates/iqrah-storage/src/user/mod.rs:10-19
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_user")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

**Application Startup:**
```rust
// Conceptual - actual implementation may vary
let content_pool = init_content_db("/data/app/iqrah/content.db").await?;
let user_pool = init_user_db("/data/app/iqrah/user.db").await?;

let content_repo = SqliteContentRepository::new(content_pool);
let user_repo = SqliteUserRepository::new(user_pool);

let learning_service = LearningService::new(content_repo, user_repo);
```

## Q2: Knowledge Graph ↔ Content DB Connection

### The String-Based Join

**Key Insight:** The knowledge graph IS the Content DB. Graph nodes and edges are stored in content.db tables.

**Connection Mechanism:** Node IDs are used as **primary/foreign keys** across tables.

### Lookup Path Example

**Goal:** Get Arabic text for word instance "WORD_INSTANCE:1:1:3" (Al-Fatihah, verse 1, word position 3)

**Step-by-Step:**

```
User Query: "What is the text for the third word of verse 1:1?"

1. Construct node ID (application layer)
   → node_id = "WORD_INSTANCE:1:1:3"

2. Query content.db (via ContentRepository)
   ↓
   SELECT arabic FROM quran_text WHERE node_id = 'WORD_INSTANCE:1:1:3'
   ↓
   Result: "ٱلرَّحْمَٰنِ"
```

**Code:**
```rust
// rust/crates/iqrah-storage/src/content/repository.rs:57-65
async fn get_quran_text(&self, node_id: &str) -> Result<Option<String>> {
    let result = query_as::<_, QuranTextRow>(
        "SELECT node_id, arabic FROM quran_text WHERE node_id = ?"
    )
    .bind(node_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(|row| row.arabic))
}
```

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│  CONTENT.DB (content.db)                                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  nodes                      quran_text                  │
│  ┌─────────────────┐        ┌──────────────────────┐   │
│  │ id              │        │ node_id (FK)         │   │
│  │ node_type       │   ┌────│ arabic               │   │
│  │ created_at      │   │    └──────────────────────┘   │
│  └─────────────────┘   │                               │
│          ▲             │    translations                │
│          │             │    ┌──────────────────────┐   │
│          └─────────────┼────│ node_id (FK)         │   │
│                        │    │ language_code        │   │
│                        │    │ translation          │   │
│                        │    └──────────────────────┘   │
│                        │                               │
│  edges                 │                               │
│  ┌─────────────────┐   │                               │
│  │ source_id (FK)  │───┘                               │
│  │ target_id (FK)  │───┐                               │
│  │ edge_type       │   │                               │
│  │ ...             │   │                               │
│  └─────────────────┘   │                               │
│                        │                               │
└────────────────────────┼───────────────────────────────┘
                         │
                         │  Nodes referenced by ID
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  USER.DB (user.db)                                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  user_memory_states                                     │
│  ┌──────────────────────────────────────────┐          │
│  │ user_id                                  │          │
│  │ node_id  ◄───────────────────────────────┼──────────┤ References content.db
│  │ stability                                │          │ node IDs (no FK)
│  │ difficulty                               │          │
│  │ energy                                   │          │
│  │ ...                                      │          │
│  └──────────────────────────────────────────┘          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Critical Detail:** There are **NO foreign key constraints** between user.db and content.db (impossible across DB files). The relationship is maintained by **application logic** using string node IDs.

### Complete Lookup Example: Get Word Text and User Progress

**Scenario:** Display a word in a review session with its current energy level.

**Application Code (Conceptual):**
```rust
async fn get_word_for_review(
    node_id: &str,
    user_id: &str,
    content_repo: &dyn ContentRepository,
    user_repo: &dyn UserRepository,
) -> Result<WordReview> {
    // 1. Get text from content.db
    let arabic = content_repo.get_quran_text(node_id).await?;
    let translation = content_repo.get_translation(node_id, "en").await?;

    // 2. Get learning state from user.db
    let memory_state = user_repo.get_memory_state(user_id, node_id).await?;

    Ok(WordReview {
        node_id: node_id.to_string(),
        arabic,
        translation,
        energy: memory_state.energy,
        due_at: memory_state.due_at,
    })
}
```

**Database Queries Executed:**
```sql
-- Query 1: content.db
SELECT arabic FROM quran_text WHERE node_id = 'WORD_INSTANCE:1:1:3';

-- Query 2: content.db
SELECT translation FROM translations WHERE node_id = 'WORD_INSTANCE:1:1:3' AND language_code = 'en';

-- Query 3: user.db
SELECT * FROM user_memory_states WHERE user_id = 'default' AND node_id = 'WORD_INSTANCE:1:1:3';
```

## Interaction Patterns

### Pattern 1: Session Generation

**Goal:** Get all due items for review.

**Flow:**
```rust
// services/session_service.rs:38-171

// 1. Get due memory states from user.db
let due_states = user_repo.get_due_states(user_id, limit).await?;
// Returns: Vec<MemoryState> with node_ids

// 2. For each node_id, get metadata from content.db
for state in due_states {
    let node = content_repo.get_node(&state.node_id).await?;
    let text = content_repo.get_quran_text(&state.node_id).await?;

    session.push(SessionItem {
        node_id: state.node_id,
        node_type: node.node_type,
        arabic: text,
        energy: state.energy,
    });
}
```

**Databases Accessed:**
1. `user.db` - Get due node IDs
2. `content.db` - Get node details (one query per node)

**Performance:** Could be optimized with batch queries.

### Pattern 2: Review Processing

**Goal:** Process a user's review and update state.

**Flow:**
```rust
// services/learning_service.rs:10-150+

// 1. Get current memory state from user.db
let mut state = user_repo.get_memory_state(user_id, node_id).await?;

// 2. Update FSRS parameters (in-memory calculation)
let fsrs = FSRS::new()?;
let scheduling = fsrs.schedule(state, review_grade)?;
state.stability = scheduling.stability;
state.difficulty = scheduling.difficulty;
state.due_at = scheduling.due_at;

// 3. Save updated state to user.db
user_repo.save_memory_state(&state).await?;

// 4. Propagate energy through graph
let edges = content_repo.get_edges_from(node_id).await?;  // content.db
for edge in edges {
    let energy_change = calculate_transfer(state.energy, edge);
    user_repo.update_energy(edge.target_id, energy_change).await?;  // user.db
}
```

**Databases Accessed:**
1. `user.db` - Get current state
2. `user.db` - Save updated state
3. `content.db` - Get edges for propagation
4. `user.db` - Update energy for connected nodes (batch)

### Pattern 3: Graph Traversal

**Goal:** Get adjacent words for context display.

**Flow:**
```rust
// content/repository.rs:256-314

async fn get_adjacent_words(&self, word_node_id: &str)
    -> Result<(Option<Node>, Option<Node>)>
{
    // Parse ID: "WORD:1:1:3" → chapter=1, verse=1, position=3
    let parts: Vec<&str> = word_node_id.split(':').collect();
    let chapter: i32 = parts[1].parse()?;
    let verse: i32 = parts[2].parse()?;
    let position: i32 = parts[3].parse()?;

    // Infer previous word ID
    let prev_id = format!("WORD:{}:{}:{}", chapter, verse, position - 1);
    let prev = self.get_node(&prev_id).await?;  // content.db

    // Infer next word ID
    let next_id = format!("WORD:{}:{}:{}", chapter, verse, position + 1);
    let next = self.get_node(&next_id).await?;  // content.db

    Ok((prev, next))
}
```

**Database Accessed:** Only `content.db`

**Note:** Uses ID inference, not edge traversal. See [07-navigation-and-algorithms.md](07-navigation-and-algorithms.md) for details.

## No Cross-Database Joins

**Limitation:** SQLite does not support joins across database files (without ATTACH).

**Implication:** All coordination must happen in application layer.

**Example of what you CANNOT do:**
```sql
-- This won't work (content.db and user.db are separate)
SELECT
    c.arabic,
    u.energy
FROM content.db.quran_text c
JOIN user.db.user_memory_states u ON c.node_id = u.node_id
WHERE u.user_id = 'default';
```

**Instead, do this in Rust:**
```rust
let states = user_repo.get_all_memory_states(user_id).await?;
for state in states {
    let arabic = content_repo.get_quran_text(&state.node_id).await?;
    // Combine in application layer
}
```

## Potential Optimization: ATTACH DATABASE

**SQLite supports attaching databases:**
```sql
ATTACH DATABASE 'user.db' AS user_db;

SELECT
    c.arabic,
    u.energy
FROM quran_text c
JOIN user_db.user_memory_states u ON c.node_id = u.node_id
WHERE u.user_id = 'default';
```

**Pros:**
- Single query instead of N+1 queries
- Better performance for large result sets

**Cons:**
- Requires content.db to be writable (to execute ATTACH)
- Breaks read-only guarantee for content.db
- More complex connection management

**Current Status:** NOT USED. All joins happen in application layer.

## Data Consistency

### Referential Integrity

**Within content.db:**
```sql
-- Enforced by foreign keys
FOREIGN KEY (source_id) REFERENCES nodes(id)
FOREIGN KEY (target_id) REFERENCES nodes(id)
FOREIGN KEY (node_id) REFERENCES nodes(id)  -- in quran_text, translations
```

**Between content.db and user.db:**
- ❌ No foreign key enforcement
- ⚠️ Possible to have memory state for non-existent node
- ⚠️ Orphaned memory states if node removed from content

**Mitigation:**
- Content DB is read-only (nodes rarely deleted)
- Application validates node existence before creating memory state
- Orphaned states are harmless (just unused data)

### Transaction Isolation

**Content DB:** Read-only transactions (no writes during runtime).

**User DB:** Read-write transactions.
- FSRS updates are atomic (single row update)
- Energy propagation uses transactions (all-or-nothing)

**Cross-DB Consistency:**
- No distributed transactions
- If app crashes during propagation, some energy updates may be lost
- Acceptable risk (energy recalculates on next review)

## File Locations

**Repository Implementations:**
- Content: [src/content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) (lines 7-315)
- User: [src/user/repository.rs](../../rust/crates/iqrah-storage/src/user/repository.rs) (lines 7-199)

**Service Layer (Orchestration):**
- Learning: [services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs) (lines 10-150+)
- Session: [services/session_service.rs](../../rust/crates/iqrah-core/src/services/session_service.rs) (lines 38-171)

---

**Navigation:** [← Knowledge Graph](03-knowledge-graph.md) | [Next: Rust Implementation →](05-rust-implementation.md)
