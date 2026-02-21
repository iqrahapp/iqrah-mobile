use anyhow::Result;
// Re-exported for frb_generated access
use iqrah_core::domain::node_id as nid;
pub use iqrah_core::exercises::{ExerciseData, ExerciseService};
use iqrah_core::{import_cbor_graph_from_bytes, KnowledgeNode, ReviewGrade};
use iqrah_core::{ContentPackage, InstalledPackage, PackageService, PackageType};
pub use iqrah_core::{ContentRepository, LearningService, SessionService, UserRepository};
use iqrah_storage::{
    create_content_repository, init_content_db, init_user_db, SqliteUserRepository,
};
use once_cell::sync::OnceCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

const SESSION_ITEM_LIMIT: u32 = 20;

pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub learning_service: Arc<LearningService>,
    pub session_service: Arc<SessionService>,
    pub exercise_service: Arc<ExerciseService>,
    user_repo_sqlite: Arc<SqliteUserRepository>,
}

static APP: OnceCell<AppState> = OnceCell::new();

// Debug-only: store content pool separately to avoid FRB trying to serialize it
#[cfg(debug_assertions)]
static DEBUG_CONTENT_POOL: OnceCell<sqlx::SqlitePool> = OnceCell::new();
#[cfg(debug_assertions)]
static DEBUG_USER_POOL: OnceCell<sqlx::SqlitePool> = OnceCell::new();

/// Get app state (helper function)
fn app() -> &'static AppState {
    APP.get()
        .expect("App not initialized - call setup_database first")
}

fn sync_setting_key(user_id: &str) -> String {
    format!("sync_last_timestamp:{}", user_id)
}

fn session_budget_mix_setting_key(session_id: &str) -> String {
    format!("session_budget_mix:{}", session_id)
}

fn stable_session_item_id(
    session_id: &str,
    node_id: i64,
    exercise_type: &str,
    completed_at_ms: i64,
) -> String {
    let key = format!("{session_id}:{node_id}:{exercise_type}:{completed_at_ms}");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, key.as_bytes()).to_string()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PersistedSessionBudgetMix {
    user_id: String,
    goal_id: String,
    items_count: u32,
    continuity_count: u32,
    due_review_count: u32,
    lexical_count: u32,
}

/// Populate the nodes table from existing verses/words/chapters data.
/// Uses INSERT OR IGNORE to be idempotent - safe to call multiple times.
async fn populate_nodes_from_content(pool: &sqlx::SqlitePool) -> Result<()> {
    // Check if nodes table already has data
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM nodes")
        .fetch_one(pool)
        .await?;

    if count > 0 {
        tracing::info!(
            "Nodes table already has {} entries, skipping population",
            count
        );
        return Ok(());
    }

    tracing::info!("Populating nodes table from existing content data...");

    // Populate chapter nodes (type = 1)
    // ID encoding: (TYPE_CHAPTER << 56) | chapter_number
    sqlx::query!(
        "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
         SELECT (CAST(1 AS INTEGER) << 56) | chapter_number,
                'CHAPTER:' || chapter_number,
                1
         FROM chapters",
    )
    .execute(pool)
    .await?;

    // Populate verse nodes (type = 2)
    // ID encoding: (TYPE_VERSE << 56) | (chapter_number << 16) | verse_number
    sqlx::query!(
        "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
         SELECT (CAST(2 AS INTEGER) << 56) | (chapter_number << 16) | verse_number,
                'VERSE:' || verse_key,
                2
         FROM verses",
    )
    .execute(pool)
    .await?;

    // Populate word nodes (type = 3)
    // ID encoding: (TYPE_WORD << 56) | word_id
    sqlx::query!(
        "INSERT OR IGNORE INTO nodes (id, ukey, node_type)
         SELECT (CAST(3 AS INTEGER) << 56) | word_id,
                'WORD:' || word_id,
                3
         FROM words",
    )
    .execute(pool)
    .await?;

    // Count how many nodes were inserted
    let new_count = sqlx::query_scalar!("SELECT COUNT(*) FROM nodes")
        .fetch_one(pool)
        .await?;

    tracing::info!("Populated {} nodes from content tables", new_count);

    Ok(())
}

/// One-time setup: initializes databases and imports graph
pub async fn setup_database(
    content_db_path: String,
    user_db_path: String,
    kg_bytes: Vec<u8>,
) -> Result<String> {
    tracing::info!("Initializing databases...");

    // Initialize databases
    let content_pool = init_content_db(&content_db_path).await?;
    let user_pool = init_user_db(&user_db_path).await?;

    // Populate nodes table from existing verses/words/chapters if empty
    // This ensures the knowledge graph is available even if CBOR import didn't run
    populate_nodes_from_content(&content_pool).await?;

    // Clone pool for debug builds before consuming it
    #[cfg(debug_assertions)]
    let debug_content_pool = content_pool.clone();
    #[cfg(debug_assertions)]
    let debug_user_pool = user_pool.clone();

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(create_content_repository(content_pool));
    let user_repo_sqlite = Arc::new(SqliteUserRepository::new(user_pool));
    let user_repo: Arc<dyn UserRepository> = user_repo_sqlite.clone();

    // Check if data already imported (efficient O(1) check)
    let has_data = content_repo.has_nodes().await?;

    if !has_data && !kg_bytes.is_empty() {
        tracing::info!("Importing knowledge graph...");
        let cursor = std::io::Cursor::new(kg_bytes);
        let stats = import_cbor_graph_from_bytes(Arc::clone(&content_repo), cursor).await?;
        tracing::info!(
            "Import complete: {} nodes, {} edges",
            stats.nodes_imported,
            stats.edges_imported
        );
    } else if has_data {
        tracing::info!("Database already contains data, skipping import");
    }

    // Create services
    let learning_service = Arc::new(LearningService::new(
        Arc::clone(&content_repo),
        Arc::clone(&user_repo),
    ));

    let session_service = Arc::new(SessionService::new(
        Arc::clone(&content_repo),
        Arc::clone(&user_repo),
    ));

    let exercise_service = Arc::new(ExerciseService::new(Arc::clone(&content_repo)));

    // Store debug pool separately (debug builds only)
    #[cfg(debug_assertions)]
    {
        let _ = DEBUG_CONTENT_POOL.set(debug_content_pool);
        let _ = DEBUG_USER_POOL.set(debug_user_pool);
    }

    // Store in global state
    APP.set(AppState {
        content_repo,
        user_repo,
        learning_service,
        session_service,
        exercise_service,
        user_repo_sqlite,
    })
    .map_err(|_| anyhow::anyhow!("App already initialized"))?;

    Ok("Database setup complete".to_string())
}

/// Setup with in-memory databases (for testing)
pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String> {
    setup_database(":memory:".to_string(), ":memory:".to_string(), kg_bytes).await
}

/// Get exercises for review session
pub async fn get_exercises(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    is_high_yield: bool,
) -> Result<Vec<ExerciseDataDto>> {
    let app = app();

    // Get due items from learning service
    // Note: surah_filter is not yet supported in V2 session service
    let due_items = app
        .session_service
        .get_due_items(&user_id, chrono::Utc::now(), limit, is_high_yield, None)
        .await?;

    let mut exercises = Vec::new();
    for item in due_items {
        let nid_val = item.node.id;
        // We need the ukey for the exercise service
        let base_ukey = nid::to_ukey(nid_val).unwrap_or_default();
        let ukey = match item.knowledge_axis {
            Some(axis) if !base_ukey.is_empty() => {
                if KnowledgeNode::parse(&base_ukey).is_some() {
                    base_ukey
                } else {
                    nid::knowledge(&base_ukey, axis)
                }
            }
            _ => base_ukey,
        };

        if let Some(surah_number) = surah_filter {
            let chapter = chapter_for_ukey(app, &ukey).await?;
            if chapter != Some(surah_number) {
                continue;
            }
        }

        // Generate V2 exercise
        match app
            .exercise_service
            .generate_exercise_v2(nid_val, &ukey)
            .await
        {
            Ok(ex) => {
                let mut dto: ExerciseDataDto = ex.into();
                // Inject user_id for EchoRecall exercises (lost during From conversion)
                if let ExerciseDataDto::EchoRecall {
                    user_id: ref mut uid,
                    ..
                } = dto
                {
                    *uid = user_id.clone();
                }
                exercises.push(dto);
            }
            Err(e) => tracing::error!("Failed to generate exercise for {}: {}", ukey, e),
        }
    }

    Ok(exercises)
}

/// Start a new session and persist its state
pub async fn start_session(user_id: String, goal_id: String) -> Result<SessionDto> {
    let app = app();

    let due_items = app
        .session_service
        .get_due_items_for_goal(
            &user_id,
            chrono::Utc::now(),
            SESSION_ITEM_LIMIT,
            false,
            Some(&goal_id),
            None,
        )
        .await?;

    let node_ids: Vec<i64> = due_items.iter().map(|item| item.node.id).collect();
    let continuity_count = due_items
        .iter()
        .filter(|item| item.session_budget.as_str() == "continuity")
        .count() as u32;
    let due_review_count = due_items
        .iter()
        .filter(|item| item.session_budget.as_str() == "due_review")
        .count() as u32;
    let lexical_count = due_items
        .iter()
        .filter(|item| item.session_budget.as_str() == "lexical")
        .count() as u32;

    app.session_service.save_session_state(&node_ids).await?;

    let session = iqrah_core::Session {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.clone(),
        goal_id: goal_id.clone(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        items_count: node_ids.len() as i32,
        items_completed: 0,
    };

    app.user_repo.create_session(&session).await?;
    let persisted_mix = PersistedSessionBudgetMix {
        user_id: user_id.clone(),
        goal_id: goal_id.clone(),
        items_count: node_ids.len() as u32,
        continuity_count,
        due_review_count,
        lexical_count,
    };
    if let Ok(mix_json) = serde_json::to_string(&persisted_mix) {
        let _ = app
            .user_repo
            .set_setting(&session_budget_mix_setting_key(&session.id), &mix_json)
            .await;
    }
    crate::telemetry::emit_session_budget_mix(
        &session.id,
        &user_id,
        &goal_id,
        node_ids.len() as u32,
        continuity_count,
        due_review_count,
        lexical_count,
    );

    Ok(SessionDto {
        id: session.id,
        user_id: session.user_id,
        goal_id: session.goal_id,
        started_at: session.started_at.timestamp_millis(),
        completed_at: None,
        items_count: session.items_count,
        items_completed: session.items_completed,
    })
}

/// Get the active (incomplete) session for a user
pub async fn get_active_session(user_id: String) -> Result<Option<SessionDto>> {
    let app = app();
    let session = app.user_repo.get_active_session(&user_id).await?;
    Ok(session.map(|s| SessionDto {
        id: s.id,
        user_id: s.user_id,
        goal_id: s.goal_id,
        started_at: s.started_at.timestamp_millis(),
        completed_at: s.completed_at.map(|t| t.timestamp_millis()),
        items_count: s.items_count,
        items_completed: s.items_completed,
    }))
}

/// Get the next session item (exercise) to present
pub async fn get_next_session_item(session_id: String) -> Result<Option<SessionItemDto>> {
    let app = app();
    let session = app
        .user_repo
        .get_session(&session_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    let node_ids = app.session_service.get_session_state().await?;
    let index = session.items_completed as usize;
    if index >= node_ids.len() {
        return Ok(None);
    }

    let node_id = node_ids[index];
    let ukey = nid::to_ukey(node_id).unwrap_or_default();
    let data = app
        .exercise_service
        .generate_exercise_v2(node_id, &ukey)
        .await?;
    let exercise_type = data.type_name().to_string();
    let mut dto: ExerciseDataDto = data.into();
    if let ExerciseDataDto::EchoRecall {
        user_id: ref mut uid,
        ..
    } = dto
    {
        *uid = session.user_id.clone();
    }

    Ok(Some(SessionItemDto {
        session_id,
        position: index as i32,
        node_id: ukey,
        exercise_type,
        exercise: dto,
    }))
}

/// Submit a completed session item
pub async fn submit_session_item(
    session_id: String,
    node_id: String,
    exercise_type: String,
    grade: u8,
    duration_ms: u64,
) -> Result<String> {
    let app = app();
    let session = app
        .user_repo
        .get_session(&session_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;

    let item = iqrah_core::SessionItem {
        id: 0,
        session_id: session_id.clone(),
        node_id: nid_val,
        exercise_type: exercise_type.clone(),
        grade: grade as i32,
        duration_ms: Some(duration_ms as i64),
        completed_at: Some(chrono::Utc::now()),
    };

    app.user_repo.insert_session_item(&item).await?;
    app.user_repo
        .update_session_progress(&session_id, session.items_completed + 1)
        .await?;

    if exercise_type != "echo_recall" {
        let review_grade = ReviewGrade::from(grade);
        app.learning_service
            .process_review(&session.user_id, nid_val, review_grade)
            .await?;
        let _ = app.session_service.increment_stat("reviews_today").await;
    }

    Ok("Session item recorded".to_string())
}

/// Complete a session and return summary
pub async fn complete_session(session_id: String) -> Result<SessionSummaryDto> {
    let app = app();
    app.user_repo.complete_session(&session_id).await?;
    app.session_service.clear_session_state().await?;

    let summary = app.user_repo.get_session_summary(&session_id).await?;
    let session_meta = app.user_repo.get_session(&session_id).await?;
    let mix_setting_key = session_budget_mix_setting_key(&session_id);
    let mix = app
        .user_repo
        .get_setting(&mix_setting_key)
        .await?
        .and_then(|raw| serde_json::from_str::<PersistedSessionBudgetMix>(&raw).ok());
    if let Err(err) = app.user_repo.delete_setting(&mix_setting_key).await {
        tracing::warn!(
            "failed to cleanup session budget mix cache for session {}: {}",
            session_id,
            err
        );
    }
    let (user_id, goal_id, continuity_count, due_review_count, lexical_count) =
        if let Some(mix) = mix {
            (
                mix.user_id,
                mix.goal_id,
                mix.continuity_count,
                mix.due_review_count,
                mix.lexical_count,
            )
        } else if let Some(session) = session_meta {
            (session.user_id, session.goal_id, 0, 0, 0)
        } else {
            ("unknown".to_string(), "unknown".to_string(), 0, 0, 0)
        };
    crate::telemetry::emit_session_outcome_quality(
        &session_id,
        &user_id,
        &goal_id,
        summary.items_count.max(0) as u32,
        summary.items_completed.max(0) as u32,
        summary.again_count.max(0) as u32,
        summary.good_count.max(0) as u32,
        summary.easy_count.max(0) as u32,
        continuity_count,
        due_review_count,
        lexical_count,
    );

    Ok(SessionSummaryDto {
        session_id: summary.session_id,
        items_count: summary.items_count,
        items_completed: summary.items_completed,
        duration_ms: summary.duration_ms,
        again_count: summary.again_count,
        hard_count: summary.hard_count,
        good_count: summary.good_count,
        easy_count: summary.easy_count,
    })
}

// ========================================================================
// Sync (Phase 2)
// ========================================================================

pub async fn get_memory_states_since(
    user_id: String,
    since_millis: i64,
) -> Result<Vec<SyncMemoryStateDto>> {
    let app = app();
    let states = app
        .user_repo_sqlite
        .get_memory_states_since(&user_id, since_millis)
        .await?;

    Ok(states
        .into_iter()
        .map(|state| SyncMemoryStateDto {
            node_id: state.node_id,
            energy: state.energy,
            fsrs_stability: Some(state.stability),
            fsrs_difficulty: Some(state.difficulty),
            last_reviewed_at: Some(state.last_reviewed.timestamp_millis()),
            next_review_at: Some(state.due_at.timestamp_millis()),
            client_updated_at: state.last_reviewed.timestamp_millis(),
        })
        .collect())
}

pub async fn get_sessions_since(user_id: String, since_millis: i64) -> Result<Vec<SyncSessionDto>> {
    let app = app();
    let sessions = app
        .user_repo_sqlite
        .get_sessions_since(&user_id, since_millis)
        .await?;

    Ok(sessions
        .into_iter()
        .map(|session| {
            let completed_at = session.completed_at.map(|t| t.timestamp_millis());
            let updated_at = completed_at.unwrap_or_else(|| session.started_at.timestamp_millis());
            SyncSessionDto {
                id: session.id,
                goal_id: Some(session.goal_id),
                started_at: session.started_at.timestamp_millis(),
                completed_at,
                items_completed: session.items_completed,
                client_updated_at: updated_at,
            }
        })
        .collect())
}

pub async fn get_session_items_since(
    user_id: String,
    since_millis: i64,
) -> Result<Vec<SyncSessionItemDto>> {
    let app = app();
    let items = app
        .user_repo_sqlite
        .get_session_items_since(&user_id, since_millis)
        .await?;

    let mut result = Vec::new();
    for item in items {
        let Some(completed_at) = item.completed_at else {
            continue;
        };
        let completed_ms = completed_at.timestamp_millis();
        result.push(SyncSessionItemDto {
            id: stable_session_item_id(
                &item.session_id,
                item.node_id,
                &item.exercise_type,
                completed_ms,
            ),
            session_id: item.session_id,
            node_id: item.node_id,
            exercise_type: item.exercise_type,
            grade: Some(item.grade),
            duration_ms: item.duration_ms,
            completed_at: Some(completed_ms),
            client_updated_at: completed_ms,
        });
    }

    Ok(result)
}

pub async fn upsert_memory_states_from_remote(
    user_id: String,
    states: Vec<SyncMemoryStateDto>,
) -> Result<String> {
    let app = app();

    for state in states {
        let last_reviewed_ms = state
            .last_reviewed_at
            .filter(|ts| *ts > 0)
            .unwrap_or(state.client_updated_at);
        let due_at_ms = state
            .next_review_at
            .filter(|ts| *ts > 0)
            .unwrap_or(last_reviewed_ms);

        let last_reviewed = chrono::DateTime::from_timestamp_millis(last_reviewed_ms)
            .unwrap_or_else(chrono::Utc::now);
        let due_at =
            chrono::DateTime::from_timestamp_millis(due_at_ms).unwrap_or_else(chrono::Utc::now);

        let memory_state = iqrah_core::MemoryState {
            user_id: user_id.clone(),
            node_id: state.node_id,
            stability: state.fsrs_stability.unwrap_or(0.0),
            difficulty: state.fsrs_difficulty.unwrap_or(0.0),
            energy: state.energy,
            last_reviewed,
            due_at,
            review_count: 0,
        };

        app.user_repo_sqlite
            .upsert_memory_state_if_newer(&memory_state)
            .await?;
    }

    Ok("ok".to_string())
}

pub async fn upsert_sessions_from_remote(
    user_id: String,
    sessions: Vec<SyncSessionDto>,
) -> Result<String> {
    let app = app();

    for session in sessions {
        let started_at = chrono::DateTime::from_timestamp_millis(session.started_at)
            .unwrap_or_else(chrono::Utc::now);
        let completed_at = session
            .completed_at
            .and_then(chrono::DateTime::from_timestamp_millis);
        let items_completed = session.items_completed.max(0);

        let session = iqrah_core::Session {
            id: session.id,
            user_id: user_id.clone(),
            goal_id: session.goal_id.unwrap_or_default(),
            started_at,
            completed_at,
            items_count: items_completed,
            items_completed,
        };

        app.user_repo_sqlite
            .upsert_session_if_newer(&session)
            .await?;
    }

    Ok("ok".to_string())
}

pub async fn upsert_session_items_from_remote(
    user_id: String,
    items: Vec<SyncSessionItemDto>,
) -> Result<String> {
    let app = app();
    let _ = &user_id;

    for item in items {
        let Some(completed_at_ms) = item.completed_at else {
            continue;
        };
        let completed_at = chrono::DateTime::from_timestamp_millis(completed_at_ms)
            .unwrap_or_else(chrono::Utc::now);

        let session_item = iqrah_core::SessionItem {
            id: 0,
            session_id: item.session_id,
            node_id: item.node_id,
            exercise_type: item.exercise_type,
            grade: item.grade.unwrap_or(0),
            duration_ms: item.duration_ms,
            completed_at: Some(completed_at),
        };

        app.user_repo_sqlite
            .insert_session_item_if_absent(&session_item)
            .await?;
    }

    Ok("ok".to_string())
}

pub async fn get_last_sync_timestamp(user_id: String) -> Result<i64> {
    let app = app();
    let key = sync_setting_key(&user_id);
    let value = app.user_repo.get_setting(&key).await?;
    Ok(value.and_then(|v| v.parse::<i64>().ok()).unwrap_or(0))
}

pub async fn set_last_sync_timestamp(user_id: String, timestamp: i64) -> Result<String> {
    let app = app();
    let key = sync_setting_key(&user_id);
    app.user_repo
        .set_setting(&key, &timestamp.to_string())
        .await?;
    Ok("ok".to_string())
}

/// Get exercises for a specific node (Sandbox/Preview)
pub async fn get_exercises_for_node(node_id: String) -> Result<Vec<ExerciseDataDto>> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;
    let base_ukey = KnowledgeNode::parse(&node_id)
        .map(|kn| kn.base_node_id)
        .unwrap_or_else(|| node_id.clone());
    let base_node_id = nid::from_ukey(&base_ukey).unwrap_or(nid_val);

    let mut exercises = Vec::new();
    let mut seen = HashSet::new();

    let mut push_unique = |exercise: ExerciseData| {
        let name = exercise.type_name();
        if seen.insert(name) {
            exercises.push(exercise.into());
        }
    };

    let default_ex = app
        .exercise_service
        .generate_exercise_v2(base_node_id, &base_ukey)
        .await?;
    push_unique(default_ex);

    if base_ukey.starts_with(nid::PREFIX_WORD) || base_ukey.starts_with(nid::PREFIX_WORD_INSTANCE) {
        if let Ok(ex) = iqrah_core::exercises::generate_translation(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_contextual_translation(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_identify_root(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_pos_tagging(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }
    }

    if base_ukey.starts_with(nid::PREFIX_VERSE) {
        if let Ok(ex) = iqrah_core::exercises::generate_sequence_recall(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_first_word_recall(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_find_mistake(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_ayah_sequence(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_cloze_deletion(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_first_letter_hint(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_missing_word_mcq(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_next_word_mcq(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_reverse_cloze(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_translate_phrase(
            base_node_id,
            &base_ukey,
            1,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_full_verse_input(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_cross_verse_connection(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }
    }

    if base_ukey.starts_with(nid::PREFIX_CHAPTER) {
        if let Ok(ex) = iqrah_core::exercises::generate_ayah_chain(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }

        if let Ok(ex) = iqrah_core::exercises::generate_ayah_sequence(
            base_node_id,
            &base_ukey,
            app.content_repo.as_ref(),
        )
        .await
        {
            push_unique(ex);
        }
    }

    Ok(exercises)
}

/// Fetch node with metadata for Sandbox
pub async fn fetch_node_with_metadata(node_id: String) -> Result<Option<NodeData>> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;
    let node = app.content_repo.get_node(nid_val).await?;

    if let Some(node) = node {
        let mut metadata = HashMap::new();
        if let Some(text) = app.content_repo.get_quran_text(nid_val).await? {
            metadata.insert("text".to_string(), text);
        }

        Ok(Some(NodeData {
            id: nid::to_ukey(node.id).unwrap_or_default(),
            node_type: node.node_type.into(),
            metadata,
        }))
    } else {
        Ok(None)
    }
}

/// Generate exercise using modern enum-based architecture (V2)
///
/// This returns lightweight ExerciseData containing only keys/IDs.
/// Flutter can then fetch content based on user preferences (Tajweed, Indopak, etc.)
pub async fn generate_exercise_v2(node_id: String) -> Result<ExerciseDataDto> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;
    let data = app
        .exercise_service
        .generate_exercise_v2(nid_val, &node_id)
        .await?;
    Ok(data.into())
}

/// Get verse content
pub async fn get_verse(verse_key: String) -> Result<Option<VerseDto>> {
    let app = app();
    let verse = app.content_repo.get_verse(&verse_key).await?;
    Ok(verse.map(|v| VerseDto {
        key: v.key,
        text_uthmani: v.text_uthmani,
        chapter_number: v.chapter_number,
        verse_number: v.verse_number,
    }))
}

/// Get word content
pub async fn get_word(word_id: i32) -> Result<Option<WordDto>> {
    let app = app();
    let word = app.content_repo.get_word(word_id as i64).await?;
    Ok(word.map(|w| w.into()))
}

/// Get all words for a verse
pub async fn get_words_for_verse(verse_key: String) -> Result<Vec<WordDto>> {
    let app = app();
    let words = app.content_repo.get_words_for_verse(&verse_key).await?;
    Ok(words.into_iter().map(|w| w.into()).collect())
}

/// Get word at specific position in a verse (resolves WORD_INSTANCE nodes)
pub async fn get_word_at_position(
    chapter: i32,
    verse: i32,
    position: i32,
) -> Result<Option<WordDto>> {
    let app = app();
    let verse_key = format!("{}:{}", chapter, verse);
    let words = app.content_repo.get_words_for_verse(&verse_key).await?;

    // Find word at the specified position (1-based)
    Ok(words
        .into_iter()
        .map(|w| w.into())
        .find(|w: &WordDto| w.position == position))
}

/// Get word translation
pub async fn get_word_translation(word_id: i32, translator_id: i32) -> Result<Option<String>> {
    let app = app();
    // Note: content_repo needs to implement get_word_translation
    // For now, we might need to add this method to ContentRepository trait if missing
    // Or use existing methods if available.
    // Checking ContentRepository trait... assuming it exists or we need to add it.
    // Based on previous context, we might need to check if get_word_translation exists.
    // If not, we'll add a placeholder or implement it.
    // Let's assume for now we need to use what's available.
    // If get_word_translation is not in ContentRepository, we might need to add it.
    // Let's check ContentRepository first.
    app.content_repo
        .get_word_translation(word_id as i64, translator_id)
        .await
}

/// Process a review
pub async fn process_review(user_id: String, node_id: String, grade: u8) -> Result<String> {
    let review_grade = ReviewGrade::from(grade);
    let app = app();

    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;

    app.learning_service
        .process_review(&user_id, nid_val, review_grade)
        .await?;
    app.session_service.increment_stat("reviews_today").await?;

    Ok("Review processed".to_string())
}

/// Get dashboard stats
pub async fn get_dashboard_stats(user_id: String) -> Result<DashboardStatsDto> {
    let app = app();

    let reviews_today = app
        .session_service
        .get_stat("reviews_today")
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let streak = app
        .session_service
        .get_stat("streak_days")
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let due_items = app
        .session_service
        .get_due_items(&user_id, chrono::Utc::now(), 1000, false, None)
        .await?;

    Ok(DashboardStatsDto {
        reviews_today,
        streak_days: streak,
        due_count: due_items.len() as u32,
    })
}

/// Get detailed stats for charts
pub async fn get_detailed_stats(_user_id: String) -> Result<DetailedStatsDto> {
    use chrono::{Duration, Utc};

    let mut activity_history = Vec::new();
    let now = Utc::now();

    // Generate last 7 days of activity (simulated)
    for i in 0..7 {
        let date = now - Duration::days(6 - i);
        let count = 10 + (date.timestamp() % 20) as i32;

        activity_history.push(ActivityPointDto {
            date: date.format("%Y-%m-%d").to_string(),
            count,
        });
    }

    Ok(DetailedStatsDto {
        activity_history,
        comprehension: ComprehensionDto {
            memorization: 0.75, // 75%
            understanding: 0.45,
            context: 0.30,
        },
    })
}

/// Get debug stats
pub async fn get_debug_stats(user_id: String) -> Result<DebugStatsDto> {
    let app = app();

    let all_nodes = app.content_repo.get_all_nodes().await?;
    let due_items = app
        .session_service
        .get_due_items(&user_id, chrono::Utc::now(), 1000, false, None)
        .await?;

    Ok(DebugStatsDto {
        total_nodes_count: all_nodes.len() as u32,
        total_edges_count: 0, // TODO: add edge count method
        due_count: due_items.len() as u32,
    })
}

/// Close database pools (debug only)
/// After calling this, the app must be restarted to reinitialize databases.
#[cfg(debug_assertions)]
pub async fn close_databases() -> Result<String> {
    // Close the debug pools if they exist
    if let Some(pool) = DEBUG_CONTENT_POOL.get() {
        pool.close().await;
        tracing::info!("Content database pool closed");
    }
    if let Some(pool) = DEBUG_USER_POOL.get() {
        pool.close().await;
        tracing::info!("User database pool closed");
    }

    Ok("Database pools closed. Please restart the app to reinitialize.".to_string())
}

/// Get database health status with table counts and issues
#[cfg(debug_assertions)]
pub async fn get_db_health() -> Result<DbHealthDto> {
    let content_pool = DEBUG_CONTENT_POOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("Content pool not available"))?;
    let user_pool = DEBUG_USER_POOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("User pool not available"))?;

    // Query content database counts
    let chapters_count = sqlx::query_scalar!("SELECT COUNT(*) FROM chapters")
        .fetch_one(content_pool)
        .await?;
    let verses_count = sqlx::query_scalar!("SELECT COUNT(*) FROM verses")
        .fetch_one(content_pool)
        .await?;
    let words_count = sqlx::query_scalar!("SELECT COUNT(*) FROM words")
        .fetch_one(content_pool)
        .await?;
    let nodes_count = sqlx::query_scalar!("SELECT COUNT(*) FROM nodes")
        .fetch_one(content_pool)
        .await?;
    let edges_count = sqlx::query_scalar!("SELECT COUNT(*) FROM edges")
        .fetch_one(content_pool)
        .await
        .unwrap_or(0);

    // Query user database counts
    let user_memory_count = sqlx::query_scalar!("SELECT COUNT(*) FROM user_memory_states")
        .fetch_one(user_pool)
        .await
        .unwrap_or(0);

    // Check for issues
    let mut issues = Vec::new();
    if chapters_count == 0 {
        issues.push("chapters table is empty".to_string());
    } else if chapters_count != 114 {
        issues.push(format!("expected 114 chapters, found {}", chapters_count));
    }
    if verses_count == 0 {
        issues.push("verses table is empty".to_string());
    }
    if words_count == 0 {
        issues.push("words table is empty".to_string());
    }
    if nodes_count == 0 {
        issues.push("nodes table is empty - knowledge graph not populated".to_string());
    }

    let is_healthy = issues.is_empty();

    Ok(DbHealthDto {
        chapters_count,
        verses_count,
        words_count,
        nodes_count,
        edges_count,
        user_memory_count,
        is_healthy,
        issues,
    })
}

/// Close database pools (release stub - returns error)
#[cfg(not(debug_assertions))]
pub async fn close_databases() -> Result<String> {
    Err(anyhow::anyhow!(
        "close_databases is only available in debug builds"
    ))
}

/// Get database health status (release stub - returns error)
#[cfg(not(debug_assertions))]
pub async fn get_db_health() -> Result<DbHealthDto> {
    Err(anyhow::anyhow!(
        "get_db_health is only available in debug builds"
    ))
}

/// Reset user progress
pub async fn reseed_database(user_id: String) -> Result<String> {
    // TODO: Implement user progress reset
    Ok(format!(
        "User {} progress reset (not yet implemented)",
        user_id
    ))
}

/// Get session preview
pub async fn get_session_preview(
    user_id: String,
    limit: u32,
    is_high_yield: bool,
) -> Result<Vec<SessionPreviewDto>> {
    let app = app();

    let items = app
        .session_service
        .get_due_items(&user_id, chrono::Utc::now(), limit, is_high_yield, None)
        .await?;

    let mut preview = Vec::new();
    for item in items {
        let arabic = app
            .content_repo
            .get_quran_text(item.node.id)
            .await?
            .unwrap_or_default();

        preview.push(SessionPreviewDto {
            node_id: nid::to_ukey(item.node.id).unwrap_or_default(),
            node_type: format!("{:?}", item.node.node_type),
            preview_text: arabic.chars().take(50).collect(),
            energy: item.memory_state.energy,
            priority_score: item.priority_score,
        });
    }

    Ok(preview)
}

/// Clear session
pub async fn clear_session() -> Result<String> {
    let app = app();
    app.session_service.clear_session_state().await?;
    Ok("Session cleared".to_string())
}

/// Search nodes
pub async fn search_nodes(query: String, limit: u32) -> Result<Vec<NodeSearchDto>> {
    let app = app();
    let limit_i64 = limit as i64;

    let mut seen = std::collections::HashSet::new();
    let mut combined_ids = Vec::new();

    if nid::node_type(&query).is_ok() {
        if let Some(node) = app.content_repo.get_node_by_ukey(&query).await? {
            if seen.insert(node.id) {
                combined_ids.push(node.id);
            }
        }
    }

    if query.contains(':') {
        #[cfg(debug_assertions)]
        {
            if let Some(pool) = DEBUG_CONTENT_POOL.get() {
                let pattern = format!("{}%", query);
                let pattern_ref = pattern.as_str();
                let rows = sqlx::query!(
                    "SELECT id as \"id!\" FROM nodes WHERE ukey LIKE ?1 LIMIT ?2",
                    pattern_ref,
                    limit_i64
                )
                .fetch_all(pool)
                .await
                .unwrap_or_default();

                for row in rows {
                    if seen.insert(row.id) && combined_ids.len() < limit as usize {
                        combined_ids.push(row.id);
                    }
                }
            }
        }
    }

    // Search by content (Arabic text) - primary search method
    let content_results = app
        .content_repo
        .search_by_content(&query, limit_i64)
        .await?;

    // Also search by node ID prefix (for power users who know IDs like "VERSE:1:1")
    let id_results: Vec<_> = if query.starts_with("VERSE:") {
        app.content_repo
            .get_all_nodes()
            .await?
            .into_iter()
            .filter(|n| {
                nid::to_ukey(n.id)
                    .map(|s| s.starts_with(&query))
                    .unwrap_or(false)
            })
            .take(limit as usize)
            .collect()
    } else {
        Vec::new()
    };

    // Combine results, content search first
    for node in content_results.into_iter().chain(id_results.into_iter()) {
        if seen.insert(node.id) && combined_ids.len() < limit as usize {
            combined_ids.push(node.id);
        }
    }

    // Build DTOs with preview text
    let mut dtos = Vec::new();
    for node_id in combined_ids {
        let arabic = app
            .content_repo
            .get_quran_text(node_id)
            .await?
            .unwrap_or_default();
        let knowledge_axis = nid::decode_knowledge_id(node_id).map(|(_, axis)| axis.to_string());
        dtos.push(NodeSearchDto {
            node_id: nid::to_ukey(node_id).unwrap_or_default(),
            node_type: nid::decode_type(node_id)
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "Unknown".to_string()),
            knowledge_axis,
            preview: arabic.chars().take(100).collect(),
        });
    }

    Ok(dtos)
}

/// Get available surahs
pub async fn get_available_surahs() -> Result<Vec<SurahInfo>> {
    let app = app();
    let chapters = app.content_repo.get_chapters().await?;
    let surahs = chapters
        .into_iter()
        .map(|chapter| {
            let name = if !chapter.name_translation.trim().is_empty() {
                chapter.name_translation.clone()
            } else if !chapter.name_transliteration.trim().is_empty() {
                chapter.name_transliteration.clone()
            } else {
                chapter.name_arabic.clone()
            };
            SurahInfo {
                number: chapter.number,
                name,
                name_arabic: chapter.name_arabic,
                name_transliteration: chapter.name_transliteration,
                name_translation: chapter.name_translation,
                verse_count: chapter.verse_count,
                revelation_place: chapter.revelation_place,
            }
        })
        .collect();
    Ok(surahs)
}

/// Get all verses for a specific surah with user's preferred translation
pub async fn get_surah_verses(chapter_number: i32) -> Result<Vec<VerseWithTranslationDto>> {
    let app = app();
    let translator_id = get_preferred_translator_id().await?;

    let verses = app
        .content_repo
        .get_verses_for_chapter(chapter_number)
        .await?;
    let mut results = Vec::new();

    for verse in verses {
        let translation = app
            .content_repo
            .get_verse_translation(&verse.key, translator_id)
            .await?;

        results.push(VerseWithTranslationDto {
            key: verse.key,
            text_uthmani: verse.text_uthmani,
            translation,
            number: verse.verse_number,
        });
    }

    Ok(results)
}

async fn chapter_for_ukey(app: &AppState, ukey: &str) -> Result<Option<i32>> {
    let base_ukey = KnowledgeNode::parse(ukey)
        .map(|kn| kn.base_node_id)
        .unwrap_or_else(|| ukey.to_string());

    if base_ukey.starts_with(nid::PREFIX_VERSE) {
        let (chapter, _) = nid::parse_verse(&base_ukey)?;
        return Ok(Some(chapter as i32));
    }

    if base_ukey.starts_with(nid::PREFIX_WORD_INSTANCE) {
        let (chapter, _, _) = nid::parse_word_instance(&base_ukey)?;
        return Ok(Some(chapter as i32));
    }

    if base_ukey.starts_with(nid::PREFIX_CHAPTER) {
        let chapter = nid::parse_chapter(&base_ukey)?;
        return Ok(Some(chapter as i32));
    }

    if base_ukey.starts_with(nid::PREFIX_WORD) {
        let word_id = nid::parse_word(&base_ukey)?;
        if let Some(word) = app.content_repo.get_word(word_id as i64).await? {
            let parts: Vec<&str> = word.verse_key.split(':').collect();
            if parts.len() == 2 {
                if let Ok(chapter) = parts[0].parse::<i32>() {
                    return Ok(Some(chapter));
                }
            }
        }
    }

    Ok(None)
}

// ========================================================================
// Translator Selection API
// ========================================================================

/// Get all available languages
pub async fn get_languages() -> Result<Vec<LanguageDto>> {
    let app = app();
    let languages = app.content_repo.get_languages().await?;

    Ok(languages
        .into_iter()
        .map(|l| LanguageDto {
            code: l.code,
            english_name: l.english_name,
            native_name: l.native_name,
            direction: l.direction,
        })
        .collect())
}

/// Get all translators for a given language
pub async fn get_translators_for_language(language_code: String) -> Result<Vec<TranslatorDto>> {
    let app = app();
    let translators = app
        .content_repo
        .get_translators_for_language(&language_code)
        .await?;

    Ok(translators
        .into_iter()
        .map(|t| TranslatorDto {
            id: t.id,
            slug: t.slug,
            full_name: t.full_name,
            language_code: t.language_code,
            description: t.description,
            license: t.license,
        })
        .collect())
}

/// Get a specific translator by ID
pub async fn get_translator(translator_id: i32) -> Result<Option<TranslatorDto>> {
    let app = app();
    let translator = app.content_repo.get_translator(translator_id).await?;

    Ok(translator.map(|t| TranslatorDto {
        id: t.id,
        slug: t.slug,
        full_name: t.full_name,
        language_code: t.language_code,
        description: t.description,
        license: t.license,
    }))
}

/// Get user's preferred translator ID
pub async fn get_preferred_translator_id() -> Result<i32> {
    let app = app();
    let translator_id = app
        .user_repo
        .get_setting("preferred_translator_id")
        .await?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1); // Default to translator_id 1 (Sahih International)

    Ok(translator_id)
}

/// Set user's preferred translator ID
pub async fn set_preferred_translator_id(translator_id: i32) -> Result<String> {
    let app = app();
    app.user_repo
        .set_setting("preferred_translator_id", &translator_id.to_string())
        .await?;

    Ok(format!("Preferred translator set to ID: {}", translator_id))
}

/// Get verse translation for a specific translator
pub async fn get_verse_translation_by_translator(
    verse_key: String,
    translator_id: i32,
) -> Result<Option<String>> {
    let app = app();
    app.content_repo
        .get_verse_translation(&verse_key, translator_id)
        .await
}

// ========================================================================
// Translation Package Management API
// ========================================================================

/// List available content packages (optionally filtered)
pub async fn get_available_packages(
    package_type: Option<String>,
    language_code: Option<String>,
) -> Result<Vec<ContentPackageDto>> {
    let app = app();
    let package_type = match package_type {
        Some(pt) => Some(pt.parse::<PackageType>()?),
        None => None,
    };

    let packages = app
        .content_repo
        .get_available_packages(package_type, language_code)
        .await?;

    Ok(packages.into_iter().map(ContentPackageDto::from).collect())
}

/// List installed content packages
pub async fn get_installed_packages() -> Result<Vec<InstalledPackageDto>> {
    let app = app();
    let packages = app.content_repo.get_installed_packages().await?;
    Ok(packages
        .into_iter()
        .map(InstalledPackageDto::from)
        .collect())
}

/// Enable an installed package
pub async fn enable_package(package_id: String) -> Result<String> {
    let app = app();
    app.content_repo.enable_package(&package_id).await?;
    Ok(format!("Package enabled: {}", package_id))
}

/// Disable an installed package
pub async fn disable_package(package_id: String) -> Result<String> {
    let app = app();
    app.content_repo.disable_package(&package_id).await?;
    Ok(format!("Package disabled: {}", package_id))
}

/// Install a translation package from raw bytes
pub async fn install_translation_pack_from_bytes(
    package_id: String,
    bytes: Vec<u8>,
) -> Result<String> {
    let app = app();
    let service = PackageService::new(Arc::clone(&app.content_repo));
    service.install_package(&package_id, bytes).await?;
    Ok(format!("Package installed: {}", package_id))
}

/// Initialize app (for Flutter bridge)
#[allow(unexpected_cfgs)]
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    use once_cell::sync::OnceCell;
    static LOG_INIT: OnceCell<()> = OnceCell::new();

    LOG_INIT.get_or_init(|| {
        if tracing_subscriber::fmt::try_init().is_err() {
            tracing::debug!("tracing subscriber already initialized");
        }
    });

    flutter_rust_bridge::setup_default_user_utils();
}

// DTOs for API responses
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExerciseDto {
    pub node_id: String,
    pub question: String,
    pub answer: String,
    pub node_type: String,
    pub translator_name: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardStatsDto {
    pub reviews_today: u32,
    pub streak_days: u32,
    pub due_count: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DebugStatsDto {
    pub total_nodes_count: u32,
    pub total_edges_count: u32,
    pub due_count: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DbHealthDto {
    pub chapters_count: i64,
    pub verses_count: i64,
    pub words_count: i64,
    pub nodes_count: i64,
    pub edges_count: i64,
    pub user_memory_count: i64,
    pub is_healthy: bool,
    pub issues: Vec<String>,
}

// ========================================================================
// Debug Infrastructure DTOs (Phase 2)
// ========================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EnergySnapshotDto {
    pub node_id: String,
    pub energy: f64,
    pub node_type: Option<String>,
    pub knowledge_axis: Option<String>,
    pub neighbors: Vec<NodeEnergyDto>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeEnergyDto {
    pub node_id: String,
    pub energy: f64,
    pub edge_weight: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PropagationDiagnosticsDto {
    pub node_found: bool,
    pub node_type: Option<String>,
    pub total_edges: u32,
    pub message: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PropagationResultDto {
    pub before: Vec<NodeEnergyDto>,
    pub after: Vec<NodeEnergyDto>,
    pub diagnostics: PropagationDiagnosticsDto,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeFilterDto {
    pub node_type: Option<String>,
    pub min_energy: Option<f64>,
    pub max_energy: Option<f64>,
    pub range: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DbQueryResultDto {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionPreviewDto {
    pub node_id: String,
    pub node_type: String,
    pub preview_text: String,
    pub energy: f64,
    pub priority_score: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionDto {
    pub id: String,
    pub user_id: String,
    pub goal_id: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub items_count: i32,
    pub items_completed: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionItemDto {
    pub session_id: String,
    pub position: i32,
    pub node_id: String,
    pub exercise_type: String,
    pub exercise: ExerciseDataDto,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionSummaryDto {
    pub session_id: String,
    pub items_count: i32,
    pub items_completed: i32,
    pub duration_ms: i64,
    pub again_count: i32,
    pub hard_count: i32,
    pub good_count: i32,
    pub easy_count: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SyncMemoryStateDto {
    pub node_id: i64,
    pub energy: f64,
    pub fsrs_stability: Option<f64>,
    pub fsrs_difficulty: Option<f64>,
    pub last_reviewed_at: Option<i64>,
    pub next_review_at: Option<i64>,
    pub client_updated_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SyncSessionDto {
    pub id: String,
    pub goal_id: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub items_completed: i32,
    pub client_updated_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SyncSessionItemDto {
    pub id: String,
    pub session_id: String,
    pub node_id: i64,
    pub exercise_type: String,
    pub grade: Option<i32>,
    pub duration_ms: Option<i64>,
    pub completed_at: Option<i64>,
    pub client_updated_at: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeSearchDto {
    pub node_id: String,
    pub node_type: String,
    pub knowledge_axis: Option<String>,
    pub preview: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name: String,
    pub name_arabic: String,
    pub name_transliteration: String,
    pub name_translation: String,
    pub verse_count: i32,
    pub revelation_place: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LanguageDto {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub direction: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TranslatorDto {
    pub id: i32,
    pub slug: String,
    pub full_name: String,
    pub language_code: String,
    pub description: Option<String>,
    pub license: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentPackageDto {
    pub package_id: String,
    pub package_type: String,
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InstalledPackageDto {
    pub package_id: String,
    pub installed_at: i64,
    pub enabled: bool,
}

impl From<ContentPackage> for ContentPackageDto {
    fn from(value: ContentPackage) -> Self {
        Self {
            package_id: value.package_id,
            package_type: value.package_type.to_string(),
            name: value.name,
            language_code: value.language_code,
            author: value.author,
            version: value.version,
            description: value.description,
            file_size: value.file_size,
            download_url: value.download_url,
            checksum: value.checksum,
            license: value.license,
        }
    }
}

impl From<InstalledPackage> for InstalledPackageDto {
    fn from(value: InstalledPackage) -> Self {
        Self {
            package_id: value.package_id,
            installed_at: value.installed_at.timestamp_millis(),
            enabled: value.enabled,
        }
    }
}

// Lightweight node + metadata surface for sandbox / previews
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeData {
    pub id: String,
    pub node_type: String,
    pub metadata: HashMap<String, String>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ExerciseDataDto {
    Memorization {
        node_id: String,
    },
    McqArToEn {
        node_id: String,
        distractor_node_ids: Vec<String>,
    },
    McqEnToAr {
        node_id: String,
        distractor_node_ids: Vec<String>,
    },
    Translation {
        node_id: String,
    },
    ContextualTranslation {
        node_id: String,
        verse_key: String,
    },
    ClozeDeletion {
        node_id: String,
        blank_position: i32,
    },
    FirstLetterHint {
        node_id: String,
        word_position: i32,
    },
    MissingWordMcq {
        node_id: String,
        blank_position: i32,
        distractor_node_ids: Vec<String>,
    },
    NextWordMcq {
        node_id: String,
        context_position: i32,
        distractor_node_ids: Vec<String>,
    },
    FullVerseInput {
        node_id: String,
    },
    AyahChain {
        node_id: String,
        verse_keys: Vec<String>,
        current_index: usize,
        completed_count: usize,
    },
    FindMistake {
        node_id: String,
        mistake_position: i32,
        correct_word_node_id: String,
        incorrect_word_node_id: String,
    },
    AyahSequence {
        node_id: String,
        correct_sequence: Vec<String>,
    },
    SequenceRecall {
        node_id: String,
        correct_sequence: Vec<String>,
        options: Vec<Vec<String>>,
    },
    FirstWordRecall {
        node_id: String,
        verse_key: String,
    },
    IdentifyRoot {
        node_id: String,
        root: String,
    },
    ReverseCloze {
        node_id: String,
        blank_position: i32,
    },
    TranslatePhrase {
        node_id: String,
        translator_id: i32,
    },
    PosTagging {
        node_id: String,
        correct_pos: String,
        options: Vec<String>,
    },
    CrossVerseConnection {
        node_id: String,
        related_verse_ids: Vec<String>,
        connection_theme: String,
    },
    /// Echo Recall exercise - progressive blur memorization
    EchoRecall {
        /// User ID for session tracking
        user_id: String,
        /// List of ayah node IDs to practice (e.g., ["VERSE:1:1", "VERSE:1:2"])
        ayah_node_ids: Vec<String>,
    },
}

impl From<iqrah_core::exercises::ExerciseData> for ExerciseDataDto {
    fn from(data: iqrah_core::exercises::ExerciseData) -> Self {
        use iqrah_core::exercises::ExerciseData::*;
        match data {
            Memorization { node_id } => ExerciseDataDto::Memorization {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
            },
            McqArToEn {
                node_id,
                distractor_node_ids,
            } => ExerciseDataDto::McqArToEn {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                distractor_node_ids: distractor_node_ids
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
            },
            McqEnToAr {
                node_id,
                distractor_node_ids,
            } => ExerciseDataDto::McqEnToAr {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                distractor_node_ids: distractor_node_ids
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
            },
            Translation { node_id } => ExerciseDataDto::Translation {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
            },
            ContextualTranslation { node_id, verse_key } => {
                ExerciseDataDto::ContextualTranslation {
                    node_id: nid::to_ukey(node_id).unwrap_or_default(),
                    verse_key,
                }
            }
            ClozeDeletion {
                node_id,
                blank_position,
            } => ExerciseDataDto::ClozeDeletion {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                blank_position,
            },
            FirstLetterHint {
                node_id,
                word_position,
            } => ExerciseDataDto::FirstLetterHint {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                word_position,
            },
            MissingWordMcq {
                node_id,
                blank_position,
                distractor_node_ids,
            } => ExerciseDataDto::MissingWordMcq {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                blank_position,
                distractor_node_ids: distractor_node_ids
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
            },
            NextWordMcq {
                node_id,
                context_position,
                distractor_node_ids,
            } => ExerciseDataDto::NextWordMcq {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                context_position,
                distractor_node_ids: distractor_node_ids
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
            },
            FullVerseInput { node_id } => ExerciseDataDto::FullVerseInput {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
            },
            AyahChain {
                node_id,
                verse_keys,
                current_index,
                completed_count,
            } => ExerciseDataDto::AyahChain {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                verse_keys,
                current_index,
                completed_count,
            },
            FindMistake {
                node_id,
                mistake_position,
                correct_word_node_id,
                incorrect_word_node_id,
            } => ExerciseDataDto::FindMistake {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                mistake_position,
                correct_word_node_id: nid::to_ukey(correct_word_node_id).unwrap_or_default(),
                incorrect_word_node_id: nid::to_ukey(incorrect_word_node_id).unwrap_or_default(),
            },
            AyahSequence {
                node_id,
                correct_sequence,
            } => ExerciseDataDto::AyahSequence {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                correct_sequence: correct_sequence
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
            },
            SequenceRecall {
                node_id,
                correct_sequence,
                options,
            } => ExerciseDataDto::SequenceRecall {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                correct_sequence: correct_sequence
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
                options: options
                    .into_iter()
                    .map(|sequence| {
                        sequence
                            .into_iter()
                            .map(|id| nid::to_ukey(id).unwrap_or_default())
                            .collect()
                    })
                    .collect(),
            },
            FirstWordRecall { node_id, verse_key } => ExerciseDataDto::FirstWordRecall {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                verse_key,
            },
            IdentifyRoot { node_id, root } => ExerciseDataDto::IdentifyRoot {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                root,
            },
            ReverseCloze {
                node_id,
                blank_position,
            } => ExerciseDataDto::ReverseCloze {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                blank_position,
            },
            TranslatePhrase {
                node_id,
                translator_id,
            } => ExerciseDataDto::TranslatePhrase {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                translator_id,
            },
            PosTagging {
                node_id,
                correct_pos,
                options,
            } => ExerciseDataDto::PosTagging {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                correct_pos,
                options,
            },
            CrossVerseConnection {
                node_id,
                related_verse_ids,
                connection_theme,
            } => ExerciseDataDto::CrossVerseConnection {
                node_id: nid::to_ukey(node_id).unwrap_or_default(),
                related_verse_ids: related_verse_ids
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
                connection_theme,
            },
            EchoRecall { ayah_node_ids } => ExerciseDataDto::EchoRecall {
                // user_id is not stored in core ExerciseData, injected at session level
                user_id: String::new(),
                ayah_node_ids: ayah_node_ids
                    .into_iter()
                    .map(|id| nid::to_ukey(id).unwrap_or_default())
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VerseDto {
    pub key: String,
    pub text_uthmani: String,
    pub chapter_number: i32,
    pub verse_number: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WordDto {
    pub id: i32,
    pub text_uthmani: String,
    pub verse_key: String,
    pub position: i32,
}

impl From<iqrah_core::Word> for WordDto {
    fn from(word: iqrah_core::Word) -> Self {
        WordDto {
            id: word.id as i32,
            text_uthmani: word.text_uthmani,
            verse_key: word.verse_key,
            position: word.position,
        }
    }
}

// ========================================================================
// Echo Recall DTOs (Phase 3)
// ========================================================================

/// Hint shown for obscured words
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HintDto {
    /// "first", "last", or "both"
    pub hint_type: String,
    /// First character hint (if applicable)
    pub first_char: Option<String>,
    /// Last character hint (if applicable)
    pub last_char: Option<String>,
}

/// Visibility state for a word in Echo Recall
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WordVisibilityDto {
    /// "visible", "obscured", or "hidden"
    pub visibility_type: String,
    /// Hint shown when obscured
    pub hint: Option<HintDto>,
    /// Blur coverage (0.0 to 1.0) when obscured
    pub coverage: Option<f64>,
}

/// A single word in an Echo Recall session
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EchoRecallWordDto {
    /// Node ID (e.g., "WORD:101")
    pub node_id: String,
    /// Arabic text
    pub text: String,
    /// Word visibility state
    pub visibility: WordVisibilityDto,
    /// Current energy level (0.0 to 1.0)
    pub energy: f64,
}

/// Complete state of an Echo Recall session
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EchoRecallStateDto {
    /// All words in the session
    pub words: Vec<EchoRecallWordDto>,
}

/// Statistics for an Echo Recall session
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EchoRecallStatsDto {
    pub total_words: u32,
    pub visible_count: u32,
    pub obscured_count: u32,
    pub hidden_count: u32,
    pub average_energy: f64,
    pub mastery_percentage: f64,
}

/// Energy update result from finalizing Echo Recall
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EnergyUpdateDto {
    pub node_id: String,
    pub energy: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VerseWithTranslationDto {
    pub key: String,
    pub text_uthmani: String,
    pub translation: Option<String>,
    pub number: i32,
}

/// Per-word timing for metrics
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WordTimingDto {
    pub word_node_id: String,
    pub duration_ms: u64,
}

/// Complete metrics for an Echo Recall session
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EchoRecallMetricsDto {
    pub word_timings: Vec<WordTimingDto>,
    pub total_duration_ms: u64,
    pub struggles: u32,
}

/// Result from finalizing Echo Recall (energy updates + metrics acknowledgement)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EchoRecallResultDto {
    pub energy_updates: Vec<EnergyUpdateDto>,
    pub words_processed: u32,
    pub average_energy: f64,
}

// ========================================================================
// Echo Recall Conversions
// ========================================================================

impl From<iqrah_core::domain::models::Hint> for HintDto {
    fn from(hint: iqrah_core::domain::models::Hint) -> Self {
        use iqrah_core::domain::models::Hint;
        match hint {
            Hint::First { char } => HintDto {
                hint_type: "first".to_string(),
                first_char: Some(char.to_string()),
                last_char: None,
            },
            Hint::Last { char } => HintDto {
                hint_type: "last".to_string(),
                first_char: None,
                last_char: Some(char.to_string()),
            },
            Hint::Both { first, last } => HintDto {
                hint_type: "both".to_string(),
                first_char: Some(first.to_string()),
                last_char: Some(last.to_string()),
            },
        }
    }
}

impl From<iqrah_core::domain::models::WordVisibility> for WordVisibilityDto {
    fn from(vis: iqrah_core::domain::models::WordVisibility) -> Self {
        use iqrah_core::domain::models::WordVisibility;
        match vis {
            WordVisibility::Visible => WordVisibilityDto {
                visibility_type: "visible".to_string(),
                hint: None,
                coverage: None,
            },
            WordVisibility::Obscured { hint, coverage } => WordVisibilityDto {
                visibility_type: "obscured".to_string(),
                hint: Some(hint.into()),
                coverage: Some(coverage),
            },
            WordVisibility::Hidden => WordVisibilityDto {
                visibility_type: "hidden".to_string(),
                hint: None,
                coverage: None,
            },
        }
    }
}

impl From<iqrah_core::domain::models::EchoRecallWord> for EchoRecallWordDto {
    fn from(word: iqrah_core::domain::models::EchoRecallWord) -> Self {
        EchoRecallWordDto {
            node_id: word.node_id,
            text: word.text,
            visibility: word.visibility.into(),
            energy: word.energy,
        }
    }
}

impl From<iqrah_core::domain::models::EchoRecallState> for EchoRecallStateDto {
    fn from(state: iqrah_core::domain::models::EchoRecallState) -> Self {
        EchoRecallStateDto {
            words: state.words.into_iter().map(|w| w.into()).collect(),
        }
    }
}

impl From<iqrah_core::domain::models::EchoRecallStats> for EchoRecallStatsDto {
    fn from(stats: iqrah_core::domain::models::EchoRecallStats) -> Self {
        EchoRecallStatsDto {
            total_words: stats.total_words as u32,
            visible_count: stats.visible_count as u32,
            obscured_count: stats.obscured_count as u32,
            hidden_count: stats.hidden_count as u32,
            average_energy: stats.average_energy,
            mastery_percentage: stats.mastery_percentage,
        }
    }
}

// Reverse conversions (DTO -> Domain) for state passing
impl From<HintDto> for iqrah_core::domain::models::Hint {
    fn from(dto: HintDto) -> Self {
        use iqrah_core::domain::models::Hint;
        match dto.hint_type.as_str() {
            "first" => Hint::First {
                char: dto
                    .first_char
                    .unwrap_or_default()
                    .chars()
                    .next()
                    .unwrap_or('_'),
            },
            "last" => Hint::Last {
                char: dto
                    .last_char
                    .unwrap_or_default()
                    .chars()
                    .next()
                    .unwrap_or('_'),
            },
            "both" => Hint::Both {
                first: dto
                    .first_char
                    .unwrap_or_default()
                    .chars()
                    .next()
                    .unwrap_or('_'),
                last: dto
                    .last_char
                    .unwrap_or_default()
                    .chars()
                    .next()
                    .unwrap_or('_'),
            },
            _ => Hint::First { char: '_' },
        }
    }
}

impl From<WordVisibilityDto> for iqrah_core::domain::models::WordVisibility {
    fn from(dto: WordVisibilityDto) -> Self {
        use iqrah_core::domain::models::WordVisibility;
        match dto.visibility_type.as_str() {
            "visible" => WordVisibility::Visible,
            "obscured" => WordVisibility::Obscured {
                hint: dto
                    .hint
                    .map(|h| h.into())
                    .unwrap_or(iqrah_core::domain::models::Hint::First { char: '_' }),
                coverage: dto.coverage.unwrap_or(0.0),
            },
            "hidden" => WordVisibility::Hidden,
            _ => WordVisibility::Visible,
        }
    }
}

impl From<EchoRecallWordDto> for iqrah_core::domain::models::EchoRecallWord {
    fn from(dto: EchoRecallWordDto) -> Self {
        iqrah_core::domain::models::EchoRecallWord {
            node_id: dto.node_id,
            text: dto.text,
            visibility: dto.visibility.into(),
            energy: dto.energy,
        }
    }
}

impl From<EchoRecallStateDto> for iqrah_core::domain::models::EchoRecallState {
    fn from(dto: EchoRecallStateDto) -> Self {
        iqrah_core::domain::models::EchoRecallState {
            words: dto.words.into_iter().map(|w| w.into()).collect(),
        }
    }
}

// ========================================================================
// Echo Recall FFI Functions (Phase 3)
// ========================================================================

/// Start a new Echo Recall session
///
/// Fetches all words from the specified ayahs, retrieves their current
/// energy levels, and calculates initial visibility.
pub async fn start_echo_recall(
    user_id: String,
    ayah_node_ids: Vec<String>,
) -> Result<EchoRecallStateDto> {
    use iqrah_core::exercises::EchoRecallExercise;

    let app = app();

    let exercise = EchoRecallExercise::new(
        &user_id,
        ayah_node_ids,
        app.content_repo.as_ref(),
        app.user_repo.as_ref(),
    )
    .await?;

    Ok(exercise.state().clone().into())
}

/// Submit a word recall and get updated state
///
/// Calculates energy change based on recall time, updates the word's
/// energy and visibility, and recalculates neighbor visibility.
///
/// Note: user_id and ayah_node_ids are included for API consistency but
/// the stateless pattern means they're only used for context/logging.
pub async fn submit_echo_recall(
    user_id: String,
    ayah_node_ids: Vec<String>,
    state: EchoRecallStateDto,
    word_node_id: String,
    recall_time_ms: u32,
) -> Result<EchoRecallStateDto> {
    use iqrah_core::exercises::EchoRecallExercise;

    // Convert DTO to domain state
    let domain_state: iqrah_core::domain::models::EchoRecallState = state.into();

    // Create exercise from state with context
    let mut exercise = EchoRecallExercise::from_state(&user_id, ayah_node_ids, domain_state);

    // Submit the recall
    exercise.submit_recall(&word_node_id, recall_time_ms)?;

    // Return updated state
    Ok(exercise.state().clone().into())
}

/// Get statistics for an Echo Recall session
pub fn echo_recall_stats(state: EchoRecallStateDto) -> EchoRecallStatsDto {
    // Convert to domain state and get stats
    let domain_state: iqrah_core::domain::models::EchoRecallState = state.into();
    domain_state.get_stats().into()
}

/// Finalize an Echo Recall session
///
/// Persists energy updates to the user's memory states and emits telemetry.
/// Accepts per-word timing metrics for detailed analytics.
pub async fn finalize_echo_recall(
    user_id: String,
    state: EchoRecallStateDto,
    metrics: EchoRecallMetricsDto,
) -> Result<EchoRecallResultDto> {
    use iqrah_core::exercises::EchoRecallExercise;

    let app = app();

    // Convert DTO to domain state
    let domain_state: iqrah_core::domain::models::EchoRecallState = state.into();

    // Create exercise from state to get finalize data
    let exercise = EchoRecallExercise::from_state(&user_id, Vec::new(), domain_state.clone());

    // Get energy updates
    let updates = exercise.finalize();

    // Persist each energy update
    for (node_id, energy) in &updates {
        // Update memory state with new energy
        if let Ok(Some(mut mem_state)) = app.user_repo.get_memory_state(&user_id, *node_id).await {
            mem_state.energy = *energy;
            let _ = app.user_repo.save_memory_state(&mem_state).await;
        }
    }

    // Increment stats - each word in EchoRecall counts as a review
    let word_count = domain_state.words.len() as u32;
    for _ in 0..word_count {
        let _ = app.session_service.increment_stat("reviews_today").await;
    }

    // Emit telemetry with detailed metrics
    let stats = domain_state.get_stats();
    crate::telemetry::emit_echo_recall_completed(
        &user_id,
        stats.total_words as u32,
        metrics.total_duration_ms,
        metrics.struggles,
        stats.average_energy,
    );

    // Log per-word timings for analytics (can be extended to persist)
    for timing in &metrics.word_timings {
        tracing::debug!(
            "Word timing: {} = {}ms",
            timing.word_node_id,
            timing.duration_ms
        );
    }

    // Convert energy updates to DTOs
    let energy_updates: Vec<EnergyUpdateDto> = updates
        .into_iter()
        .filter_map(|(id, energy)| {
            nid::to_ukey(id).map(|node_id| EnergyUpdateDto { node_id, energy })
        })
        .collect();

    Ok(EchoRecallResultDto {
        energy_updates,
        words_processed: stats.total_words as u32,
        average_energy: stats.average_energy,
    })
}

// ========================================================================
// Telemetry API v1 (Polling-based, Rust→Dart)
// ========================================================================

/// Drain all pending telemetry events as JSON strings
/// Call periodically from Dart (e.g. every 30s or on app background)
pub fn drain_telemetry_events() -> Result<Vec<String>> {
    Ok(crate::telemetry::drain_events())
}

/// Get count of pending telemetry events
pub fn get_telemetry_event_count() -> Result<u32> {
    Ok(crate::telemetry::pending_event_count() as u32)
}

/// Debug: manually emit a test event (dev only)
#[cfg(debug_assertions)]
pub fn debug_emit_test_event() -> Result<String> {
    crate::telemetry::emit_daily_health(0, 100, 5, 20, 0.95, 0.90, 0.05, 30.0, 564);
    Ok("Test event emitted".to_string())
}

/// Debug: manually emit a test event (release stub - returns error)
#[cfg(not(debug_assertions))]
pub fn debug_emit_test_event() -> Result<String> {
    Err(anyhow::anyhow!(
        "debug_emit_test_event is only available in debug builds"
    ))
}

// ========================================================================
// Debug Infrastructure API (Phase 2)
// ========================================================================

/// Get energy snapshot for a node including neighbor energies
pub async fn get_energy_snapshot(user_id: String, node_id: String) -> Result<EnergySnapshotDto> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;

    // Get node type and knowledge axis from ID
    let node_type = nid::decode_type(nid_val).map(|t| t.to_string());
    let knowledge_axis = nid::decode_knowledge_id(nid_val).map(|(_, axis)| axis.to_string());

    // Get node energy
    let state = app.user_repo.get_memory_state(&user_id, nid_val).await?;
    let energy = state.map(|s| s.energy).unwrap_or(0.0);

    // Get neighbor edges
    let edges = app.content_repo.get_edges_from(nid_val).await?;

    // Get neighbor energies
    let mut neighbors = Vec::new();
    for edge in edges {
        let neighbor_state = app
            .user_repo
            .get_memory_state(&user_id, edge.target_id)
            .await?;
        neighbors.push(NodeEnergyDto {
            node_id: nid::to_ukey(edge.target_id).unwrap_or_default(),
            energy: neighbor_state.map(|s| s.energy).unwrap_or(0.0),
            edge_weight: edge.param1, // param1 holds propagation weight
        });
    }

    Ok(EnergySnapshotDto {
        node_id,
        energy,
        node_type,
        knowledge_axis,
        neighbors,
    })
}

/// Simulate energy propagation without persisting changes
pub async fn simulate_propagation(
    user_id: String,
    node_id: String,
    energy_delta: f64,
) -> Result<PropagationResultDto> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;

    // Look up the node to get diagnostic info
    let node = app.content_repo.get_node(nid_val).await?;
    let node_found = node.is_some();
    let node_type = node.as_ref().map(|n| n.node_type.to_string());

    // Get edges from this node
    let edges = app.content_repo.get_edges_from(nid_val).await?;
    let total_edges = edges.len() as u32;

    // Build diagnostic message
    let message = if !node_found {
        format!("Node '{}' not found in the knowledge graph", node_id)
    } else if edges.is_empty() {
        format!(
            "Node '{}' (type: {}) has no outgoing edges. This node may be a leaf node or isolated.",
            node_id,
            node_type.as_deref().unwrap_or("unknown")
        )
    } else {
        format!(
            "Found {} connected nodes from '{}' (type: {})",
            edges.len(),
            node_id,
            node_type.as_deref().unwrap_or("unknown")
        )
    };

    let diagnostics = PropagationDiagnosticsDto {
        node_found,
        node_type,
        total_edges,
        message,
    };

    // Collect before states
    let mut before = Vec::new();
    let mut after = Vec::new();

    for edge in edges {
        let target_ukey = nid::to_ukey(edge.target_id).unwrap_or_default();
        let target_state = app
            .user_repo
            .get_memory_state(&user_id, edge.target_id)
            .await?;
        let current_energy = target_state.map(|s| s.energy).unwrap_or(0.0);

        before.push(NodeEnergyDto {
            node_id: target_ukey.clone(),
            energy: current_energy,
            edge_weight: edge.param1,
        });

        // Calculate propagated delta (simplified: weight * delta * 0.1 decay)
        let propagated = energy_delta * edge.param1 * 0.1;
        let new_energy = (current_energy + propagated).clamp(0.0, 1.0);

        after.push(NodeEnergyDto {
            node_id: target_ukey,
            energy: new_energy,
            edge_weight: edge.param1,
        });
    }

    Ok(PropagationResultDto {
        before,
        after,
        diagnostics,
    })
}

/// Parse a verse range string into individual node IDs
/// Supports: "1:1-1:7" or "1:1-7" (shorthand for same chapter)
pub fn parse_node_range(range: String) -> Result<Vec<String>> {
    let range = range.trim();

    // Try to parse as "chapter:start-end" (shorthand)
    if let Some((prefix, end_str)) = range.rsplit_once('-') {
        if let Some((chapter_str, start_str)) = prefix.rsplit_once(':') {
            // Shorthand: "1:1-7"
            if let (Ok(chapter), Ok(start), Ok(end)) = (
                chapter_str.parse::<u8>(),
                start_str.parse::<u16>(),
                end_str.parse::<u16>(),
            ) {
                if start <= end && (1..=114).contains(&chapter) {
                    return Ok((start..=end)
                        .map(|v| format!("VERSE:{}:{}", chapter, v))
                        .collect());
                }
            }
        }

        // Try full format: "1:1-2:5" (cross-chapter - not supported for simplicity)
        // For now, return error for cross-chapter ranges
        if prefix.contains(':') && end_str.contains(':') {
            return Err(anyhow::anyhow!(
                "Cross-chapter ranges not supported. Use same-chapter format: 1:1-7"
            ));
        }
    }

    Err(anyhow::anyhow!(
        "Invalid range format. Use: chapter:start-end (e.g., 1:1-7)"
    ))
}

/// Query nodes with filters (type, energy range)
pub async fn query_nodes_filtered(
    user_id: String,
    filter: NodeFilterDto,
    limit: u32,
) -> Result<Vec<NodeSearchDto>> {
    let app = app();
    let all_nodes = app.content_repo.get_all_nodes().await?;

    let mut results = Vec::new();

    for node in all_nodes {
        // Apply node type filter
        if let Some(ref type_filter) = filter.node_type {
            let node_type_str: String = node.node_type.into();
            if !node_type_str.eq_ignore_ascii_case(type_filter) {
                continue;
            }
        }

        // Apply energy range filter
        if filter.min_energy.is_some() || filter.max_energy.is_some() {
            let state = app.user_repo.get_memory_state(&user_id, node.id).await?;
            let energy = state.map(|s| s.energy).unwrap_or(0.0);

            if let Some(min) = filter.min_energy {
                if energy < min {
                    continue;
                }
            }
            if let Some(max) = filter.max_energy {
                if energy > max {
                    continue;
                }
            }
        }

        // Apply range filter (verse range)
        if let Some(ref range_str) = filter.range {
            if let Ok(valid_ids) = parse_node_range(range_str.clone()) {
                let ukey = nid::to_ukey(node.id).unwrap_or_default();
                if !valid_ids.contains(&ukey) {
                    continue;
                }
            }
        }

        // Get preview text
        let preview = app
            .content_repo
            .get_quran_text(node.id)
            .await?
            .unwrap_or_default();

        let knowledge_axis = nid::decode_knowledge_id(node.id).map(|(_, axis)| axis.to_string());

        results.push(NodeSearchDto {
            node_id: nid::to_ukey(node.id).unwrap_or_default(),
            node_type: format!("{:?}", node.node_type),
            knowledge_axis,
            preview: preview.chars().take(100).collect(),
        });

        if results.len() >= limit as usize {
            break;
        }
    }

    Ok(results)
}

/// Execute a debug SQL query (debug builds only)
/// Only SELECT queries are allowed for safety
#[cfg(debug_assertions)]
pub async fn execute_debug_query(sql: String) -> Result<DbQueryResultDto> {
    use sqlx::{Column, Row};

    let trimmed = sql.trim();
    let upper = trimmed.to_uppercase();

    // Only allow SELECT queries for safety
    if !upper.starts_with("SELECT") {
        return Err(anyhow::anyhow!("Only SELECT queries allowed in debug mode"));
    }

    let user_tables = [
        "USER_MEMORY_STATES",
        "SESSION_STATE",
        "PROPAGATION_EVENTS",
        "PROPAGATION_DETAILS",
        "USER_STATS",
        "APP_SETTINGS",
        "USER_BANDIT_STATE",
    ];
    let content_tables = [
        "NODES",
        "EDGES",
        "CHAPTERS",
        "VERSES",
        "WORDS",
        "SCRIPT_RESOURCES",
        "SCRIPT_CONTENTS",
        "VERSE_TRANSLATIONS",
        "WORD_TRANSLATIONS",
        "TRANSLATORS",
        "LANGUAGES",
        "ROOTS",
        "LEMMAS",
        "MORPHOLOGY_SEGMENTS",
        "GOALS",
        "GOAL_NODES",
        "RECITERS",
        "VERSE_RECITATIONS",
        "WORD_AUDIO",
        "TEXT_VARIANTS",
        "CONTENT_PACKAGES",
        "INSTALLED_PACKAGES",
    ];

    let user_match = user_tables.iter().any(|t| upper.contains(t));
    let content_match = content_tables.iter().any(|t| upper.contains(t));

    if user_match && content_match {
        return Err(anyhow::anyhow!(
            "Cross-database queries are not supported in debug mode"
        ));
    }

    let pool = if user_match {
        DEBUG_USER_POOL
            .get()
            .ok_or_else(|| anyhow::anyhow!("Debug user pool not initialized"))?
    } else {
        DEBUG_CONTENT_POOL
            .get()
            .ok_or_else(|| anyhow::anyhow!("Debug content pool not initialized"))?
    };
    let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(trimmed)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Query failed: {}", e))?;

    // Extract column names from first row if available
    let columns: Vec<String> = if rows.is_empty() {
        Vec::new()
    } else {
        rows[0]
            .columns()
            .iter()
            .map(|c: &sqlx::sqlite::SqliteColumn| c.name().to_string())
            .collect()
    };

    // Convert rows to string values
    let result_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row: &sqlx::sqlite::SqliteRow| {
            columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    // Try to get value as different types
                    row.try_get::<String, _>(i)
                        .or_else(|_| row.try_get::<i64, _>(i).map(|v: i64| v.to_string()))
                        .or_else(|_| row.try_get::<f64, _>(i).map(|v: f64| v.to_string()))
                        .or_else(|_| row.try_get::<bool, _>(i).map(|v: bool| v.to_string()))
                        .unwrap_or_else(|_| "NULL".to_string())
                })
                .collect()
        })
        .collect();

    Ok(DbQueryResultDto {
        columns,
        rows: result_rows,
    })
}

/// Execute a debug SQL query (release stub - returns error)
#[cfg(not(debug_assertions))]
pub async fn execute_debug_query(_sql: String) -> Result<DbQueryResultDto> {
    Err(anyhow::anyhow!(
        "execute_debug_query is only available in debug builds"
    ))
}
// ========================================================================
// Stats DTOs
// ========================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DetailedStatsDto {
    pub activity_history: Vec<ActivityPointDto>,
    pub comprehension: ComprehensionDto,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActivityPointDto {
    pub date: String,
    pub count: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ComprehensionDto {
    pub memorization: f64,
    pub understanding: f64,
    pub context: f64,
}
