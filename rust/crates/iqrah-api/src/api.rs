use anyhow::Result;
// Re-exported for frb_generated access
use iqrah_core::domain::node_id as nid;
pub use iqrah_core::exercises::{ExerciseData, ExerciseService};
use iqrah_core::{import_cbor_graph_from_bytes, ReviewGrade};
pub use iqrah_core::{ContentRepository, LearningService, SessionService, UserRepository};
use iqrah_storage::{
    create_content_repository, init_content_db, init_user_db, SqliteUserRepository,
};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub learning_service: Arc<LearningService>,
    pub session_service: Arc<SessionService>,
    pub exercise_service: Arc<ExerciseService>,
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

    // Clone pool for debug builds before consuming it
    #[cfg(debug_assertions)]
    let debug_content_pool = content_pool.clone();
    #[cfg(debug_assertions)]
    let debug_user_pool = user_pool.clone();

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(create_content_repository(content_pool));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool));

    // Check if data already imported
    let all_nodes = content_repo.get_all_nodes().await?;

    if all_nodes.is_empty() && !kg_bytes.is_empty() {
        tracing::info!("Importing knowledge graph...");
        let cursor = std::io::Cursor::new(kg_bytes);
        let stats = import_cbor_graph_from_bytes(Arc::clone(&content_repo), cursor).await?;
        tracing::info!(
            "Import complete: {} nodes, {} edges",
            stats.nodes_imported,
            stats.edges_imported
        );
    } else {
        tracing::info!(
            "Database already contains {} nodes, skipping import",
            all_nodes.len()
        );
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
    _surah_filter: Option<i32>,
    is_high_yield: bool,
) -> Result<Vec<ExerciseDataDto>> {
    let app = app();

    // Get due items from learning service
    // Note: surah_filter is not yet supported in V2 session service
    let due_items = app
        .session_service
        .get_due_items(&user_id, limit, is_high_yield, None)
        .await?;

    let mut exercises = Vec::new();
    for item in due_items {
        let nid_val = item.node.id;
        // We need the ukey for the exercise service
        let ukey = nid::to_ukey(nid_val).unwrap_or_default();

        // Generate V2 exercise
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

/// Get exercises for a specific node (Sandbox/Preview)
pub async fn get_exercises_for_node(node_id: String) -> Result<Vec<ExerciseDataDto>> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;
    let ex = app
        .exercise_service
        .generate_exercise_v2(nid_val, &node_id)
        .await?;
    Ok(vec![ex.into()])
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
        .get_due_items(&user_id, 1000, false, None)
        .await?;

    Ok(DashboardStatsDto {
        reviews_today,
        streak_days: streak,
        due_count: due_items.len() as u32,
    })
}

/// Get debug stats
pub async fn get_debug_stats(user_id: String) -> Result<DebugStatsDto> {
    let app = app();

    let all_nodes = app.content_repo.get_all_nodes().await?;
    let due_items = app
        .session_service
        .get_due_items(&user_id, 1000, false, None)
        .await?;

    Ok(DebugStatsDto {
        total_nodes_count: all_nodes.len() as u32,
        total_edges_count: 0, // TODO: add edge count method
        due_count: due_items.len() as u32,
    })
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
        .get_due_items(&user_id, limit, is_high_yield, None)
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

    let all_nodes = app.content_repo.get_all_nodes().await?;

    // Simple prefix search
    let results: Vec<_> = all_nodes
        .into_iter()
        .filter(|n| {
            nid::to_ukey(n.id)
                .map(|s| s.starts_with(&query))
                .unwrap_or(false)
        })
        .take(limit as usize)
        .collect();

    let mut dtos = Vec::new();
    for node in results {
        let arabic = app
            .content_repo
            .get_quran_text(node.id)
            .await?
            .unwrap_or_default();
        dtos.push(NodeSearchDto {
            node_id: nid::to_ukey(node.id).unwrap_or_default(),
            node_type: format!("{:?}", node.node_type),
            preview: arabic.chars().take(100).collect(),
        });
    }

    Ok(dtos)
}

/// Get available surahs
pub async fn get_available_surahs() -> Result<Vec<SurahInfo>> {
    // TODO: Implement surah listing from database
    // For now return empty - needs to query chapters from content.db
    Ok(Vec::new())
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

// ========================================================================
// Debug Infrastructure DTOs (Phase 2)
// ========================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EnergySnapshotDto {
    pub node_id: String,
    pub energy: f64,
    pub neighbors: Vec<NodeEnergyDto>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeEnergyDto {
    pub node_id: String,
    pub energy: f64,
    pub edge_weight: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PropagationResultDto {
    pub before: Vec<NodeEnergyDto>,
    pub after: Vec<NodeEnergyDto>,
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
pub struct NodeSearchDto {
    pub node_id: String,
    pub node_type: String,
    pub preview: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name: String,
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

// ========================================================================
// Debug Infrastructure API (Phase 2)
// ========================================================================

/// Get energy snapshot for a node including neighbor energies
pub async fn get_energy_snapshot(user_id: String, node_id: String) -> Result<EnergySnapshotDto> {
    let app = app();
    let nid_val = nid::from_ukey(&node_id).ok_or_else(|| anyhow::anyhow!("Invalid node ID"))?;

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

    // Get edges from this node
    let edges = app.content_repo.get_edges_from(nid_val).await?;

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

    Ok(PropagationResultDto { before, after })
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

        results.push(NodeSearchDto {
            node_id: nid::to_ukey(node.id).unwrap_or_default(),
            node_type: format!("{:?}", node.node_type),
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
        return Err(anyhow::anyhow!(
            "Only SELECT queries allowed in debug mode"
        ));
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
