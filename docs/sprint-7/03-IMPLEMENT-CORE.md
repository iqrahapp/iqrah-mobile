# Step 3: Implement Core Domain Logic

## Goal
Create `iqrah-core` with domain models, repository traits, and business logic (zero database dependencies).

## Architecture Principle
**Hexagonal Architecture (Ports & Adapters)**
- **Domain** = Core business entities and logic
- **Ports** = Interfaces (traits) for external systems
- **Adapters** = Implementations (in iqrah-storage)

## Implementation Structure

```
iqrah-core/src/
├── lib.rs
├── domain/
│   ├── mod.rs
│   ├── models.rs      # Core types
│   └── errors.rs      # Domain errors
├── ports/
│   ├── mod.rs
│   ├── content_repository.rs
│   └── user_repository.rs
└── services/
    ├── mod.rs
    └── learning_service.rs
```

## Task 3.1: Domain Models

**File:** `rust/crates/iqrah-core/src/domain/models.rs`

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Node types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    WordInstance,
    Verse,
    Surah,
    Lemma,
    Root,
}

impl From<String> for NodeType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "word_instance" => NodeType::WordInstance,
            "verse" => NodeType::Verse,
            "surah" => NodeType::Surah,
            "lemma" => NodeType::Lemma,
            "root" => NodeType::Root,
            _ => NodeType::WordInstance,
        }
    }
}

impl From<NodeType> for String {
    fn from(nt: NodeType) -> Self {
        match nt {
            NodeType::WordInstance => "word_instance".to_string(),
            NodeType::Verse => "verse".to_string(),
            NodeType::Surah => "surah".to_string(),
            NodeType::Lemma => "lemma".to_string(),
            NodeType::Root => "root".to_string(),
        }
    }
}

// Core node entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
}

// Edge types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
}

// Distribution types for energy propagation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DistributionType {
    Const,
    Normal,
    Beta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,
    pub param1: f64,
    pub param2: f64,
}

// Memory state (FSRS + Energy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub review_count: u32,
}

impl MemoryState {
    pub fn new_for_node(user_id: String, node_id: String) -> Self {
        Self {
            user_id,
            node_id,
            stability: 0.0,
            difficulty: 0.0,
            energy: 0.0,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 0,
        }
    }
}

// Review grades
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewGrade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl From<u8> for ReviewGrade {
    fn from(val: u8) -> Self {
        match val {
            1 => ReviewGrade::Again,
            2 => ReviewGrade::Hard,
            3 => ReviewGrade::Good,
            4 => ReviewGrade::Easy,
            _ => ReviewGrade::Good,
        }
    }
}

// Propagation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationEvent {
    pub id: Option<i64>,
    pub source_node_id: String,
    pub event_timestamp: DateTime<Utc>,
    pub details: Vec<PropagationDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationDetail {
    pub target_node_id: String,
    pub energy_change: f64,
    pub reason: String,
}

// Exercise types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exercise {
    Recall {
        node_id: String,
        question: String,
        answer: String,
    },
    Cloze {
        node_id: String,
        text: String,
        blank_word: String,
    },
    McqArToEn {
        node_id: String,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
    McqEnToAr {
        node_id: String,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
}
```

**File:** `rust/crates/iqrah-core/src/domain/errors.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid energy value: {0} (must be 0.0-1.0)")]
    InvalidEnergy(f64),

    #[error("Invalid review grade: {0}")]
    InvalidGrade(u8),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
```

**File:** `rust/crates/iqrah-core/src/domain/mod.rs`

```rust
pub mod models;
pub mod errors;

pub use models::*;
pub use errors::*;
```

## Task 3.2: Repository Traits (Ports)

**File:** `rust/crates/iqrah-core/src/ports/content_repository.rs`

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use crate::domain::{Node, Edge};

#[async_trait]
pub trait ContentRepository: Send + Sync {
    /// Get a node by ID
    async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>>;

    /// Get edges from a source node
    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>>;

    /// Get node metadata by key
    async fn get_metadata(&self, node_id: &str, key: &str) -> anyhow::Result<Option<String>>;

    /// Get all metadata for a node
    async fn get_all_metadata(&self, node_id: &str) -> anyhow::Result<HashMap<String, String>>;

    /// Check if node exists
    async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool>;
}
```

**File:** `rust/crates/iqrah-core/src/ports/user_repository.rs`

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::domain::{MemoryState, PropagationEvent, ReviewGrade};

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Get memory state for a node
    async fn get_memory_state(&self, user_id: &str, node_id: &str) -> anyhow::Result<Option<MemoryState>>;

    /// Save or update memory state
    async fn save_memory_state(&self, state: &MemoryState) -> anyhow::Result<()>;

    /// Get all due memory states
    async fn get_due_states(&self, user_id: &str, due_before: DateTime<Utc>, limit: u32) -> anyhow::Result<Vec<MemoryState>>;

    /// Update energy for a node
    async fn update_energy(&self, user_id: &str, node_id: &str, new_energy: f64) -> anyhow::Result<()>;

    /// Log a propagation event
    async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()>;

    /// Get session state
    async fn get_session_state(&self) -> anyhow::Result<Vec<String>>;

    /// Save session state
    async fn save_session_state(&self, node_ids: &[String]) -> anyhow::Result<()>;

    /// Clear session state
    async fn clear_session_state(&self) -> anyhow::Result<()>;

    /// Get user stat
    async fn get_stat(&self, key: &str) -> anyhow::Result<Option<String>>;

    /// Set user stat
    async fn set_stat(&self, key: &str, value: &str) -> anyhow::Result<()>;
}
```

**File:** `rust/crates/iqrah-core/src/ports/mod.rs`

```rust
pub mod content_repository;
pub mod user_repository;

pub use content_repository::ContentRepository;
pub use user_repository::UserRepository;
```

## Task 3.3: Update lib.rs

**File:** `rust/crates/iqrah-core/src/lib.rs`

```rust
pub mod domain;
pub mod ports;

// Re-export commonly used types
pub use domain::{
    Node, NodeType, Edge, EdgeType, DistributionType,
    MemoryState, ReviewGrade, Exercise,
    PropagationEvent, PropagationDetail,
    DomainError,
};

pub use ports::{ContentRepository, UserRepository};
```

## Validation

### Build iqrah-core

```bash
cd /home/user/iqrah-mobile/rust
cargo build -p iqrah-core
```

Expected: Compiles without errors.

### Run Tests (Placeholder)

```bash
cargo test -p iqrah-core
```

Expected: No tests yet, but should run successfully.

## Success Criteria

- [ ] `iqrah-core` compiles without errors
- [ ] No database dependencies in Cargo.toml
- [ ] Domain models defined
- [ ] Repository traits defined
- [ ] All types are serializable (Serialize + Deserialize)

## Next Step

Proceed to `04-IMPLEMENT-STORAGE.md`
