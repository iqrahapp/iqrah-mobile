use anyhow::Result;
// Re-exported for frb_generated access
pub use iqrah_core::exercises::{ExerciseData, ExerciseService};
use iqrah_core::{import_cbor_graph_from_bytes, ReviewGrade};
pub use iqrah_core::{ContentRepository, LearningService, SessionService, UserRepository};
use iqrah_storage::{init_content_db, init_user_db, SqliteContentRepository, SqliteUserRepository};
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

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));
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
        let node_id = &item.node.id;

        // Generate V2 exercise
        match app.exercise_service.generate_exercise_v2(node_id).await {
            Ok(ex) => exercises.push(ex.into()),
            Err(e) => tracing::error!("Failed to generate exercise for {}: {}", node_id, e),
        }
    }

    Ok(exercises)
}

/// Get exercises for a specific node (Sandbox/Preview)
pub async fn get_exercises_for_node(node_id: String) -> Result<Vec<ExerciseDataDto>> {
    let app = app();
    let ex = app.exercise_service.generate_exercise_v2(&node_id).await?;
    Ok(vec![ex.into()])
}

/// Fetch node with metadata for Sandbox
pub async fn fetch_node_with_metadata(node_id: String) -> Result<Option<NodeData>> {
    let app = app();
    let node = app.content_repo.get_node(&node_id).await?;

    if let Some(node) = node {
        let mut metadata = HashMap::new();
        if let Some(text) = app.content_repo.get_quran_text(&node_id).await? {
            metadata.insert("text".to_string(), text);
        }

        Ok(Some(NodeData {
            id: node.id,
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
    let data = app.exercise_service.generate_exercise_v2(&node_id).await?;
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
    let word = app.content_repo.get_word(word_id).await?;
    Ok(word.map(|w| w.into()))
}

/// Get all words for a verse
pub async fn get_words_for_verse(verse_key: String) -> Result<Vec<WordDto>> {
    let app = app();
    let words = app.content_repo.get_words_for_verse(&verse_key).await?;
    Ok(words.into_iter().map(|w| w.into()).collect())
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
        .get_word_translation(word_id, translator_id)
        .await
}

/// Process a review
pub async fn process_review(user_id: String, node_id: String, grade: u8) -> Result<String> {
    let review_grade = ReviewGrade::from(grade);
    let app = app();

    app.learning_service
        .process_review(&user_id, &node_id, review_grade)
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
            .get_quran_text(&item.node.id)
            .await?
            .unwrap_or_default();

        preview.push(SessionPreviewDto {
            node_id: item.node.id,
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
        .filter(|n| n.id.starts_with(&query))
        .take(limit as usize)
        .collect();

    let mut dtos = Vec::new();
    for node in results {
        let arabic = app
            .content_repo
            .get_quran_text(&node.id)
            .await?
            .unwrap_or_default();
        dtos.push(NodeSearchDto {
            node_id: node.id,
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
            Memorization { node_id } => ExerciseDataDto::Memorization { node_id },
            McqArToEn {
                node_id,
                distractor_node_ids,
            } => ExerciseDataDto::McqArToEn {
                node_id,
                distractor_node_ids,
            },
            McqEnToAr {
                node_id,
                distractor_node_ids,
            } => ExerciseDataDto::McqEnToAr {
                node_id,
                distractor_node_ids,
            },
            Translation { node_id } => ExerciseDataDto::Translation { node_id },
            ContextualTranslation { node_id, verse_key } => {
                ExerciseDataDto::ContextualTranslation { node_id, verse_key }
            }
            ClozeDeletion {
                node_id,
                blank_position,
            } => ExerciseDataDto::ClozeDeletion {
                node_id,
                blank_position,
            },
            FirstLetterHint {
                node_id,
                word_position,
            } => ExerciseDataDto::FirstLetterHint {
                node_id,
                word_position,
            },
            MissingWordMcq {
                node_id,
                blank_position,
                distractor_node_ids,
            } => ExerciseDataDto::MissingWordMcq {
                node_id,
                blank_position,
                distractor_node_ids,
            },
            NextWordMcq {
                node_id,
                context_position,
                distractor_node_ids,
            } => ExerciseDataDto::NextWordMcq {
                node_id,
                context_position,
                distractor_node_ids,
            },
            FullVerseInput { node_id } => ExerciseDataDto::FullVerseInput { node_id },
            AyahChain {
                node_id,
                verse_keys,
                current_index,
                completed_count,
            } => ExerciseDataDto::AyahChain {
                node_id,
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
                node_id,
                mistake_position,
                correct_word_node_id,
                incorrect_word_node_id,
            },
            AyahSequence {
                node_id,
                correct_sequence,
            } => ExerciseDataDto::AyahSequence {
                node_id,
                correct_sequence,
            },
            IdentifyRoot { node_id, root } => ExerciseDataDto::IdentifyRoot { node_id, root },
            ReverseCloze {
                node_id,
                blank_position,
            } => ExerciseDataDto::ReverseCloze {
                node_id,
                blank_position,
            },
            TranslatePhrase {
                node_id,
                translator_id,
            } => ExerciseDataDto::TranslatePhrase {
                node_id,
                translator_id,
            },
            PosTagging {
                node_id,
                correct_pos,
                options,
            } => ExerciseDataDto::PosTagging {
                node_id,
                correct_pos,
                options,
            },
            CrossVerseConnection {
                node_id,
                related_verse_ids,
                connection_theme,
            } => ExerciseDataDto::CrossVerseConnection {
                node_id,
                related_verse_ids,
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
        Self {
            id: word.id,
            text_uthmani: word.text_uthmani,
            verse_key: word.verse_key,
            position: word.position,
        }
    }
}
