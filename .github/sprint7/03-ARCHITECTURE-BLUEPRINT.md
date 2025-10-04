# Sprint 7: Architecture Blueprint

**Date:** 2025-10-04
**Purpose:** Define the new clean architecture for production-ready codebase

---

## Architectural Principles

### 1. **Hexagonal Architecture (Ports & Adapters)**
- **Domain Logic** = Core business rules (scheduler, FSRS, propagation)
- **Ports** = Interfaces/traits
- **Adapters** = Implementations (SQLite, Flutter bridge)

### 2. **Dependency Inversion**
- Core domain depends on abstractions (traits)
- Infrastructure depends on core
- Never the reverse

### 3. **Single Responsibility**
- Each module has ONE reason to change
- Separate concerns: storage, business logic, API

### 4. **Testability First**
- Every component mockable
- No global state
- Dependency injection everywhere

---

## New Project Structure

```
iqrah/
├── rust/
│   ├── Cargo.toml                 # Workspace root
│   ├── crates/
│   │   ├── iqrah-core/            # Domain logic (no DB, no IO)
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── domain/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── models.rs  # Core types
│   │   │   │   │   ├── scheduler.rs
│   │   │   │   │   ├── propagation.rs
│   │   │   │   │   └── exercises.rs
│   │   │   │   ├── ports/         # Trait definitions
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── content_repository.rs
│   │   │   │   │   ├── user_repository.rs
│   │   │   │   │   └── scheduler.rs
│   │   │   │   └── services/      # Business logic
│   │   │   │       ├── mod.rs
│   │   │   │       ├── learning_service.rs
│   │   │   │       ├── session_service.rs
│   │   │   │       └── stats_service.rs
│   │   │   └── Cargo.toml
│   │   │
│   │   ├── iqrah-storage/         # Data access layer
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── content/       # content.db access
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── repository.rs
│   │   │   │   │   ├── queries.rs
│   │   │   │   │   └── models.rs
│   │   │   │   ├── user/          # user.db access
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── repository.rs
│   │   │   │   │   ├── queries.rs
│   │   │   │   │   └── models.rs
│   │   │   │   └── migrations/    # SQL migration files
│   │   │   └── Cargo.toml
│   │   │
│   │   ├── iqrah-api/             # Flutter bridge (FRB)
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── api.rs         # FRB public functions
│   │   │   │   └── types.rs       # FRB types
│   │   │   └── Cargo.toml
│   │   │
│   │   └── iqrah-cli/             # CLI tool for developers
│   │       ├── src/
│   │       │   ├── main.rs
│   │       │   └── commands/
│   │       │       ├── mod.rs
│   │       │       ├── session.rs
│   │       │       ├── stats.rs
│   │       │       └── debug.rs
│   │       └── Cargo.toml
│   │
│   └── tests/                     # Integration tests
│       ├── common/
│       │   └── fixtures.rs
│       ├── scheduler_tests.rs
│       └── propagation_tests.rs
│
├── lib/                           # Flutter (unchanged)
└── assets/
```

---

## Module Boundaries

### `iqrah-core` (Domain Layer)

**Purpose:** Pure business logic, zero dependencies on storage or IO

**Responsibilities:**
- Define core domain models (`Node`, `MemoryState`, `Exercise`)
- Implement FSRS scheduling algorithm
- Energy propagation calculations
- Exercise generation logic
- Session priority scoring

**Key Files:**

#### `domain/models.rs`
```rust
// Core value objects
pub struct NodeId(String);
pub struct Energy(f64);  // 0.0-1.0, validated

// Domain entities
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
}

pub struct MemoryState {
    pub node_id: NodeId,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: Energy,
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub review_count: u32,
}

pub enum ReviewGrade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}
```

#### `ports/content_repository.rs`
```rust
#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn get_node(&self, id: &NodeId) -> Result<Node>;
    async fn get_quran_text(&self, id: &NodeId) -> Result<String>;
    async fn get_translation(&self, id: &NodeId, lang: &str) -> Result<String>;
    async fn get_importance_scores(&self, ids: &[NodeId]) -> Result<HashMap<NodeId, ImportanceScore>>;
    async fn get_children_nodes(&self, parent_id: &NodeId) -> Result<Vec<Node>>;
    async fn get_edges_from(&self, source_id: &NodeId) -> Result<Vec<Edge>>;
}
```

#### `ports/user_repository.rs`
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_memory_state(&self, user_id: &str, node_id: &NodeId) -> Result<Option<MemoryState>>;
    async fn save_memory_state(&self, user_id: &str, state: &MemoryState) -> Result<()>;
    async fn get_due_states(&self, user_id: &str, due_before: DateTime<Utc>) -> Result<Vec<MemoryState>>;
    async fn update_energy(&self, user_id: &str, updates: &[(NodeId, Energy)]) -> Result<()>;
    async fn log_review(&self, review: ReviewRecord) -> Result<()>;
}
```

#### `ports/question_repository.rs` (NEW - Sprint 7+)
```rust
#[async_trait]
pub trait QuestionRepository: Send + Sync {
    // Content-side (content.db)
    async fn get_questions_for_node(&self, node_id: &NodeId) -> Result<Vec<Question>>;
    async fn get_question(&self, question_id: &str) -> Result<Question>;
    async fn get_question_links(&self, question_id: &str) -> Result<Vec<QuestionNodeLink>>;
    async fn filter_questions(
        &self,
        filters: QuestionFilters,  // difficulty, aqeedah_school, tafsir_source
    ) -> Result<Vec<Question>>;

    // User-side (user.db)
    async fn get_question_memory_state(
        &self,
        user_id: &str,
        question_id: &str,
    ) -> Result<Option<QuestionMemoryState>>;
    async fn save_question_memory_state(
        &self,
        user_id: &str,
        state: &QuestionMemoryState,
    ) -> Result<()>;
    async fn get_due_questions(
        &self,
        user_id: &str,
        due_before: DateTime<Utc>,
        filters: QuestionFilters,
    ) -> Result<Vec<QuestionMemoryState>>;
    async fn log_question_review(&self, review: QuestionReviewRecord) -> Result<()>;
    async fn flag_question(&self, user_id: &str, question_id: &str, flag: QuestionFlag) -> Result<()>;
}

pub struct Question {
    pub question_id: String,
    pub question_text: String,
    pub question_type: QuestionType,        // MCQ, TypeAnswer
    pub difficulty: u8,                     // 1-4
    pub verification_status: VerificationStatus,
    pub aqeedah_school: Option<String>,
    pub tafsir_source: Option<String>,
    pub metadata_json: String,              // MCQ options, etc.
}

pub struct QuestionNodeLink {
    pub question_id: String,
    pub node_id: NodeId,
    pub link_strength: f32,                 // 0.0-1.0
}

pub struct QuestionMemoryState {
    pub user_id: String,
    pub question_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub mastery: f32,                       // 0.0-1.0, contributes to node energy
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub review_count: u32,
}

pub struct QuestionFilters {
    pub difficulty_range: Option<(u8, u8)>,
    pub aqeedah_school: Option<String>,
    pub tafsir_sources: Option<Vec<String>>,
    pub verified_only: bool,
}
```

#### `services/question_service.rs` (NEW - Sprint 7+)
```rust
pub struct QuestionService {
    question_repo: Arc<dyn QuestionRepository>,
    user_repo: Arc<dyn UserRepository>,
    scheduler: Arc<dyn Scheduler>,
}

impl QuestionService {
    /// Process a question review and recalculate linked node energies
    pub async fn process_question_review(
        &self,
        user_id: &str,
        question_id: &str,
        grade: ReviewGrade,
    ) -> Result<QuestionMemoryState> {
        // 1. Get or create question memory state
        let current_state = self.get_or_create_question_state(user_id, question_id).await?;

        // 2. Update FSRS state
        let new_state = self.scheduler.update_question_state(current_state, grade)?;

        // 3. Save new state
        self.question_repo.save_question_memory_state(user_id, &new_state).await?;

        // 4. Get all nodes linked to this question
        let links = self.question_repo.get_question_links(question_id).await?;

        // 5. Recalculate energy for each linked node
        for link in links {
            self.recalculate_node_energy_with_questions(user_id, &link.node_id).await?;
        }

        Ok(new_state)
    }

    /// Recalculate node energy considering ALL linked questions
    async fn recalculate_node_energy_with_questions(
        &self,
        user_id: &str,
        node_id: &NodeId,
    ) -> Result<()> {
        // 1. Get auto-exercise mastery (existing energy from FSRS reviews)
        let node_state = self.user_repo.get_memory_state(user_id, node_id).await?;
        let auto_mastery = node_state.map(|s| s.energy.0).unwrap_or(0.0);

        // 2. Get all questions linked to this node
        let questions = self.question_repo.get_questions_for_node(node_id).await?;

        if questions.is_empty() {
            // No questions: energy = auto_mastery only
            return Ok(());
        }

        // 3. Get user's mastery of each question
        let mut question_masteries = Vec::new();
        for question in questions {
            if let Some(q_state) = self.question_repo
                .get_question_memory_state(user_id, &question.question_id)
                .await?
            {
                question_masteries.push(q_state.mastery);
            } else {
                // Unreviewed question = 0.0 mastery
                question_masteries.push(0.0);
            }
        }

        // 4. Calculate combined energy: auto_mastery × avg(question_masteries)
        let avg_question_mastery = question_masteries.iter().sum::<f32>()
                                  / question_masteries.len() as f32;
        let new_energy = auto_mastery as f32 * avg_question_mastery;

        // 5. Update node energy
        self.user_repo.update_energy(user_id, &[(node_id.clone(), Energy(new_energy as f64))]).await?;

        Ok(())
    }

    /// Check for content updates and recalculate energies if needed
    pub async fn sync_content_version(&self, user_id: &str) -> Result<()> {
        // Check if content.db version changed
        // If yes, recalculate all nodes that have questions
        // Implementation details in migration strategy
        Ok(())
    }
}
```

#### `services/learning_service.rs`
```rust
pub struct LearningService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
    scheduler: Arc<dyn Scheduler>,
}

impl LearningService {
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        user_repo: Arc<dyn UserRepository>,
        scheduler: Arc<dyn Scheduler>,
    ) -> Self {
        Self { content_repo, user_repo, scheduler }
    }

    pub async fn process_review(
        &self,
        user_id: &str,
        node_id: &NodeId,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        // 1. Get current state
        let current_state = self.get_or_create_state(user_id, node_id).await?;

        // 2. Calculate new FSRS state
        let new_state = self.scheduler.update_state(current_state, grade)?;

        // 3. Calculate energy delta
        let energy_delta = calculate_energy_delta(&current_state, &new_state);

        // 4. Propagate if significant
        if energy_delta.abs() > 0.0001 {
            self.propagate_energy(user_id, node_id, energy_delta).await?;
        }

        // 5. Save new state
        self.user_repo.save_memory_state(user_id, &new_state).await?;

        // 6. Log review
        self.user_repo.log_review(ReviewRecord {
            user_id: user_id.to_string(),
            node_id: node_id.clone(),
            grade,
            reviewed_at: Utc::now(),
            previous_energy: current_state.energy,
            new_energy: new_state.energy,
        }).await?;

        Ok(new_state)
    }

    async fn get_or_create_state(&self, user_id: &str, node_id: &NodeId) -> Result<MemoryState> {
        match self.user_repo.get_memory_state(user_id, node_id).await? {
            Some(state) => Ok(state),
            None => {
                // Lazy creation on first review
                let state = MemoryState::new_for_node(node_id.clone());
                self.user_repo.save_memory_state(user_id, &state).await?;
                Ok(state)
            }
        }
    }
}
```

### `iqrah-storage` (Infrastructure Layer)

**Purpose:** Database access, query execution, migrations

**Responsibilities:**
- Implement `ContentRepository` trait (content.db)
- Implement `UserRepository` trait (user.db)
- Define SQL queries (compile-time checked with SQLx)
- Handle migrations
- Manage connection pools

**Key Files:**

#### `content/repository.rs`
```rust
pub struct SqliteContentRepository {
    pool: SqlitePool,
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    async fn get_node(&self, id: &NodeId) -> Result<Node> {
        let row = sqlx::query!(
            r#"SELECT id, node_type as "node_type: NodeType"
               FROM nodes
               WHERE id = ?"#,
            id.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Node {
            id: NodeId::from(row.id),
            node_type: row.node_type,
        })
    }

    async fn get_quran_text(&self, id: &NodeId) -> Result<String> {
        let row = sqlx::query!(
            "SELECT arabic FROM quran_text WHERE node_id = ?",
            id.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.arabic)
    }

    async fn get_translation(&self, id: &NodeId, lang: &str) -> Result<String> {
        let row = sqlx::query!(
            "SELECT translation FROM translations
             WHERE node_id = ? AND language_code = ?",
            id.as_str(),
            lang
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.translation)
    }

    // ... other implementations
}
```

#### `user/repository.rs`
```rust
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn get_memory_state(&self, user_id: &str, node_id: &NodeId) -> Result<Option<MemoryState>> {
        let row = sqlx::query_as!(
            MemoryStateRow,
            r#"SELECT node_id, stability, difficulty, energy,
                      last_reviewed, due_at, review_count
               FROM user_memory_states
               WHERE user_id = ? AND node_id = ?"#,
            user_id,
            node_id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn save_memory_state(&self, user_id: &str, state: &MemoryState) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO user_memory_states
               (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(user_id, node_id) DO UPDATE SET
                 stability = excluded.stability,
                 difficulty = excluded.difficulty,
                 energy = excluded.energy,
                 last_reviewed = excluded.last_reviewed,
                 due_at = excluded.due_at,
                 review_count = excluded.review_count"#,
            user_id,
            state.node_id.as_str(),
            state.stability,
            state.difficulty,
            state.energy.value(),
            state.last_reviewed.timestamp_millis(),
            state.due_at.timestamp_millis(),
            state.review_count
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ... other implementations
}
```

### `iqrah-api` (Presentation Layer)

**Purpose:** Flutter bridge, public API

**Responsibilities:**
- Expose FRB functions
- Convert between Dart and Rust types
- Manage app lifecycle
- Handle errors for Flutter

**Key Files:**

#### `api.rs`
```rust
use iqrah_core::services::LearningService;

pub struct AppState {
    learning_service: Arc<LearningService>,
    // ... other services
}

static APP: OnceCell<AppState> = OnceCell::new();

pub async fn init_app(content_db_path: String, user_db_path: String) -> Result<String> {
    let content_pool = SqlitePool::connect(&content_db_path).await?;
    let user_pool = SqlitePool::connect(&user_db_path).await?;

    // Run migrations on user.db
    sqlx::migrate!("../iqrah-storage/migrations")
        .run(&user_pool)
        .await?;

    let content_repo = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo = Arc::new(SqliteUserRepository::new(user_pool));
    let scheduler = Arc::new(FsrsScheduler::default());

    let learning_service = Arc::new(LearningService::new(
        content_repo,
        user_repo,
        scheduler,
    ));

    APP.set(AppState { learning_service })
        .map_err(|_| anyhow!("App already initialized"))?;

    Ok("Initialized".to_string())
}

pub async fn get_exercises(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    is_high_yield_mode: bool,
) -> Result<Vec<Exercise>> {
    let app = APP.get().ok_or_else(|| anyhow!("App not initialized"))?;

    let due_nodes = app.learning_service
        .get_due_items(&user_id, limit, surah_filter, is_high_yield_mode)
        .await?;

    // Build exercises...
    Ok(exercises)
}
```

### `iqrah-cli` (Developer Tool)

**Purpose:** Local testing, debugging, admin tasks

**Responsibilities:**
- Interactive session simulation
- Database inspection
- Migration testing
- Performance profiling

**Key Files:**

#### `main.rs`
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "iqrah-cli")]
#[command(about = "Developer CLI for Iqrah app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a session preview
    Session {
        #[arg(short, long, default_value = "default_user")]
        user_id: String,

        #[arg(short, long, default_value_t = 20)]
        limit: u32,

        #[arg(short, long)]
        surah: Option<i32>,

        #[arg(long)]
        high_yield: bool,
    },

    /// Test scoring algorithm
    TestScoring {
        #[arg(short, long)]
        node_id: String,
    },

    /// Run migrations
    Migrate {
        #[arg(short, long)]
        user_db: String,
    },

    /// Inspect propagation
    Propagation {
        #[arg(short, long)]
        node_id: String,

        #[arg(short, long, default_value_t = 10)]
        depth: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Session { user_id, limit, surah, high_yield } => {
            // Initialize services
            let service = init_service().await?;

            // Get session
            let items = service.get_due_items(&user_id, limit, surah, high_yield).await?;

            // Pretty print
            for (idx, item) in items.iter().enumerate() {
                println!("{:2}. {} (energy: {:.2})", idx + 1, item.id, item.energy);
            }
        }
        // ... other commands
    }

    Ok(())
}
```

---

## Dependency Graph

```
iqrah-api
    ├── iqrah-core (domain logic)
    └── iqrah-storage (repositories)
            └── iqrah-core (traits only)

iqrah-cli
    ├── iqrah-core
    └── iqrah-storage

Integration Tests
    ├── iqrah-core
    └── iqrah-storage
```

**Key Rule:** `iqrah-core` has ZERO dependencies on storage or IO

---

## Testing Strategy by Layer

### Unit Tests (iqrah-core)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_process_review_updates_energy() {
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_content_repo = MockContentRepository::new();
        let mock_scheduler = MockScheduler::new();

        // Setup expectations
        mock_user_repo.expect_get_memory_state()
            .returning(|_, _| Ok(Some(MemoryState::default())));

        mock_user_repo.expect_save_memory_state()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = LearningService::new(
            Arc::new(mock_content_repo),
            Arc::new(mock_user_repo),
            Arc::new(mock_scheduler),
        );

        let result = service.process_review("user1", &NodeId::from("node1"), ReviewGrade::Good).await;

        assert!(result.is_ok());
    }
}
```

### Integration Tests (tests/)
```rust
#[tokio::test]
async fn test_full_session_flow() {
    let (content_pool, user_pool) = create_test_dbs().await;

    let content_repo = SqliteContentRepository::new(content_pool);
    let user_repo = SqliteUserRepository::new(user_pool);
    let scheduler = FsrsScheduler::default();

    let service = LearningService::new(
        Arc::new(content_repo),
        Arc::new(user_repo),
        Arc::new(scheduler),
    );

    // Test real flow
    let items = service.get_due_items("test_user", 10, None, false).await?;
    assert_eq!(items.len(), 10);

    let state = service.process_review("test_user", &items[0].node_id, ReviewGrade::Good).await?;
    assert!(state.energy.value() > 0.0);
}
```

---

## Migration from Current Code

### Phase 1: Create New Structure (Week 1)
1. Create workspace with 4 crates
2. Move domain models to `iqrah-core`
3. Define repository traits in `iqrah-core/ports`
4. Implement services in `iqrah-core/services`

### Phase 2: Implement Storage (Week 1-2)
1. Create SQLx repositories in `iqrah-storage`
2. Write SQL queries with compile-time checks
3. Implement migration framework
4. Write integration tests

### Phase 3: Wire Up API (Week 2)
1. Update `iqrah-api` to use new services
2. Remove old `sqlite_repo.rs`
3. Update Flutter bridge
4. Test end-to-end

### Phase 4: Add CLI (Week 2-3)
1. Create `iqrah-cli` crate
2. Add debugging commands
3. Document usage

---

## Success Metrics

✅ **Code Quality**
- All business logic in `iqrah-core` (testable without DB)
- 80%+ test coverage
- Zero SQL strings in business logic

✅ **Performance**
- Queries 2x faster (fewer JOINs)
- user.db 90% smaller (lazy creation)
- Session generation < 50ms

✅ **Maintainability**
- New exercise variant: 1 file change
- Schema migration: 1 SQL file
- Feature flag: inject different service

---

Next: See `04-MIGRATION-STRATEGY.md` for step-by-step execution plan
