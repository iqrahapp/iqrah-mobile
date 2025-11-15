# Step 7: Update API Layer (Flutter Bridge)

## Goal
Wire the new two-database architecture into the Flutter bridge (FRB) API.

## Changes Required

1. Update initialization to use two database paths
2. Create repository instances
3. Update all existing API functions to use repositories
4. Maintain backward compatibility with Flutter app

## Task 7.1: Update API Initialization

**File:** `rust/crates/iqrah-api/src/api.rs`

```rust
use std::sync::Arc;
use once_cell::sync::OnceCell;
use iqrah_core::{ContentRepository, UserRepository};
use iqrah_storage::{
    SqliteContentRepository, SqliteUserRepository,
    init_content_db, init_user_db,
    migrate_from_old_db, old_db_exists, is_migration_complete, mark_migration_complete,
};

pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
}

static APP: OnceCell<AppState> = OnceCell::new();

/// Initialize the app with two databases
#[flutter_rust_bridge::frb(sync)]
pub fn init_app(
    content_db_path: String,
    user_db_path: String,
    old_db_path: Option<String>,
) -> Result<String, String> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            init_app_async(content_db_path, user_db_path, old_db_path).await
        })
        .map_err(|e| e.to_string())
}

async fn init_app_async(
    content_db_path: String,
    user_db_path: String,
    old_db_path: Option<String>,
) -> anyhow::Result<String> {
    // Initialize content.db
    let content_pool = init_content_db(&content_db_path).await?;

    // Initialize user.db (runs migrations v1 and v2)
    let user_pool = init_user_db(&user_db_path).await?;

    // Check for one-time migration from old database
    if let Some(old_path) = old_db_path {
        let migration_marker = format!("{}.migrated", old_path);

        if old_db_exists(&old_path) && !is_migration_complete(&migration_marker) {
            println!("Migrating from old database...");

            migrate_from_old_db(&old_path, &content_pool, &user_pool).await?;

            mark_migration_complete(&migration_marker)?;

            // Rename old database
            std::fs::rename(&old_path, format!("{}.backup", old_path))?;

            println!("Migration complete!");
        }
    }

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool));

    // Store in global state
    APP.set(AppState {
        content_repo,
        user_repo,
    })
    .map_err(|_| anyhow::anyhow!("App already initialized"))?;

    Ok("App initialized successfully".to_string())
}

/// Get app state (helper function)
fn app() -> &'static AppState {
    APP.get().expect("App not initialized")
}
```

## Task 7.2: Update Existing API Functions

Now we need to update all existing API functions to use the repositories instead of the old `SqliteRepository`.

### Example: Get Due Items

**Old code (using monolithic repository):**
```rust
pub async fn get_due_items(user_id: String, limit: u32) -> Result<Vec<Exercise>> {
    let repo = app().repo.get_due_items(...);
    // ...
}
```

**New code (using separated repositories):**
```rust
#[flutter_rust_bridge::frb(sync)]
pub fn get_due_items(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    high_yield: bool,
) -> Result<Vec<ExerciseData>, String> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            get_due_items_async(user_id, limit, surah_filter, high_yield).await
        })
        .map_err(|e| e.to_string())
}

async fn get_due_items_async(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    high_yield: bool,
) -> anyhow::Result<Vec<ExerciseData>> {
    let app = app();
    let now = chrono::Utc::now();

    // 1. Get due memory states from user.db
    let mut states = app.user_repo.get_due_states(&user_id, now, limit).await?;

    // 2. Filter by surah if requested
    if let Some(surah) = surah_filter {
        states.retain(|state| {
            // Check if node belongs to surah (parse node_id)
            state.node_id.starts_with(&format!("WORD_INSTANCE:{}:", surah))
        });
    }

    // 3. Get metadata for each node from content.db
    let mut exercises = Vec::new();

    for state in states.iter().take(limit as usize) {
        let node_id = &state.node_id;

        // Get metadata
        let metadata = app.content_repo.get_all_metadata(node_id).await?;

        // Build exercise based on node type and metadata
        let exercise = build_exercise(node_id, &metadata)?;

        exercises.push(exercise);
    }

    Ok(exercises)
}

fn build_exercise(node_id: &str, metadata: &HashMap<String, String>) -> anyhow::Result<ExerciseData> {
    // Create exercise from metadata
    let arabic = metadata.get("arabic").cloned().unwrap_or_default();
    let translation = metadata.get("translation").cloned().unwrap_or_default();

    Ok(ExerciseData {
        node_id: node_id.to_string(),
        question: arabic,
        answer: translation,
        exercise_type: "recall".to_string(),
    })
}

// FRB-compatible struct
#[derive(Clone, Debug)]
pub struct ExerciseData {
    pub node_id: String,
    pub question: String,
    pub answer: String,
    pub exercise_type: String,
}
```

### Example: Process Review

```rust
#[flutter_rust_bridge::frb(sync)]
pub fn process_review(
    user_id: String,
    node_id: String,
    grade: u8,
) -> Result<String, String> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            process_review_async(user_id, node_id, grade).await
        })
        .map_err(|e| e.to_string())
}

async fn process_review_async(
    user_id: String,
    node_id: String,
    grade: u8,
) -> anyhow::Result<String> {
    use iqrah_core::ReviewGrade;

    let app = app();
    let grade = ReviewGrade::from(grade);

    // 1. Get current memory state
    let state = app.user_repo.get_memory_state(&user_id, &node_id).await?
        .unwrap_or_else(|| {
            // Create new state if doesn't exist
            iqrah_core::MemoryState::new_for_node(user_id.clone(), node_id.clone())
        });

    // 2. Update FSRS state (using fsrs crate)
    let fsrs = fsrs::FSRS::default();
    let fsrs_state = fsrs::MemoryState {
        stability: state.stability,
        difficulty: state.difficulty,
    };

    let rating = match grade {
        ReviewGrade::Again => fsrs::Rating::Again,
        ReviewGrade::Hard => fsrs::Rating::Hard,
        ReviewGrade::Good => fsrs::Rating::Good,
        ReviewGrade::Easy => fsrs::Rating::Easy,
    };

    let scheduling_cards = fsrs.repeat(fsrs_state, chrono::Utc::now().timestamp_millis());
    let chosen_card = match rating {
        fsrs::Rating::Again => &scheduling_cards.again,
        fsrs::Rating::Hard => &scheduling_cards.hard,
        fsrs::Rating::Good => &scheduling_cards.good,
        fsrs::Rating::Easy => &scheduling_cards.easy,
    };

    // 3. Calculate new energy (simple formula for now)
    let energy_delta = match grade {
        ReviewGrade::Easy => 0.2,
        ReviewGrade::Good => 0.1,
        ReviewGrade::Hard => -0.05,
        ReviewGrade::Again => -0.15,
    };

    let new_energy = (state.energy + energy_delta).clamp(0.0, 1.0);

    // 4. Create updated state
    let new_state = iqrah_core::MemoryState {
        user_id: user_id.clone(),
        node_id: node_id.clone(),
        stability: chosen_card.memory_state.stability,
        difficulty: chosen_card.memory_state.difficulty,
        energy: new_energy,
        last_reviewed: chrono::Utc::now(),
        due_at: chrono::DateTime::from_timestamp_millis(chosen_card.due.timestamp_millis())
            .unwrap_or_else(|| chrono::Utc::now()),
        review_count: state.review_count + 1,
    };

    // 5. Save state
    app.user_repo.save_memory_state(&new_state).await?;

    // 6. Propagate energy (if significant change)
    if energy_delta.abs() > 0.01 {
        propagate_energy(&user_id, &node_id, energy_delta).await?;
    }

    // 7. Update stats
    update_stats(&user_id).await?;

    Ok(format!("Review processed. New energy: {:.2}", new_energy))
}

async fn propagate_energy(user_id: &str, node_id: &str, delta: f64) -> anyhow::Result<()> {
    let app = app();

    // Get outgoing edges
    let edges = app.content_repo.get_edges_from(node_id).await?;

    let mut details = Vec::new();

    for edge in edges {
        // Calculate propagated energy based on edge parameters
        let propagated = delta * edge.param1; // Simplified

        // Update target node energy
        if let Some(target_state) = app.user_repo.get_memory_state(user_id, &edge.target_id).await? {
            let new_energy = (target_state.energy + propagated).clamp(0.0, 1.0);
            app.user_repo.update_energy(user_id, &edge.target_id, new_energy).await?;

            details.push(iqrah_core::PropagationDetail {
                target_node_id: edge.target_id.clone(),
                energy_change: propagated,
                reason: "Propagated".to_string(),
            });
        }
    }

    // Log propagation event
    let event = iqrah_core::PropagationEvent {
        id: None,
        source_node_id: node_id.to_string(),
        event_timestamp: chrono::Utc::now(),
        details,
    };

    app.user_repo.log_propagation(&event).await?;

    Ok(())
}

async fn update_stats(user_id: &str) -> anyhow::Result<()> {
    let app = app();

    // Increment reviews_today
    let current = app.user_repo.get_stat("reviews_today").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    app.user_repo.set_stat("reviews_today", &(current + 1).to_string()).await?;

    Ok(())
}
```

### Example: Get Stats

```rust
#[flutter_rust_bridge::frb(sync)]
pub fn get_stats() -> Result<StatsData, String> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            get_stats_async().await
        })
        .map_err(|e| e.to_string())
}

async fn get_stats_async() -> anyhow::Result<StatsData> {
    let app = app();

    let reviews_today = app.user_repo.get_stat("reviews_today").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let streak = app.user_repo.get_stat("streak_days").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(StatsData {
        reviews_today,
        streak_days: streak,
    })
}

#[derive(Clone, Debug)]
pub struct StatsData {
    pub reviews_today: u32,
    pub streak_days: u32,
}
```

## Task 7.3: Update lib.rs

**File:** `rust/crates/iqrah-api/src/lib.rs`

```rust
pub mod api;

// Re-export for FRB
pub use api::*;
```

## Task 7.4: Update Flutter Integration

**File:** `lib/main.dart` (update initialization)

```dart
import 'package:path_provider/path_provider.dart';
import 'src/rust/api/api.dart' as rust_api;

Future<void> initializeApp() async {
  final docsDir = await getApplicationDocumentsDirectory();

  final contentDbPath = '${docsDir.path}/content.db';
  final userDbPath = '${docsDir.path}/user.db';
  final oldDbPath = '${docsDir.path}/iqrah.db'; // For migration

  try {
    final result = rust_api.initApp(
      contentDbPath: contentDbPath,
      userDbPath: userDbPath,
      oldDbPath: oldDbPath, // Will be used for one-time migration
    );

    print(result);
  } catch (e) {
    print('Initialization error: $e');
    rethrow;
  }
}
```

## Validation

### Build API Crate

```bash
cd /home/user/iqrah-mobile/rust
cargo build -p iqrah-api
```

### Generate Flutter Bindings

```bash
cd /home/user/iqrah-mobile
flutter_rust_bridge_codegen generate
```

### Test Initialization

```bash
flutter test test/initialization_test.dart
```

## Success Criteria

- [ ] API compiles without errors
- [ ] Flutter bindings regenerate successfully
- [ ] Initialization accepts two database paths
- [ ] Migration runs automatically on first launch
- [ ] All existing API functions work with new repositories
- [ ] No breaking changes to Flutter app interface

## Next Step

Proceed to `08-TESTING.md`
