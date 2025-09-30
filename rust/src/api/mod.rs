pub mod simple;
pub mod types;

pub use simple::query_propagation_details;
pub use types::{PropagationDetailSummary, PropagationFilter};

use crate::{
    exercises::Exercise,
    repository::{
        DebugStats, ItemPreview, MemoryState, NodeData, ReviewGrade, WordInstanceContext,
    },
};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Once;

static LOG_INIT: Once = Once::new();

/// One-time setup: initializes DB, imports graph, and syncs the default user.
/// Should be called on first app launch.
pub async fn setup_database(db_path: Option<String>, kg_bytes: Vec<u8>) -> Result<String> {
    let db_path = if db_path.is_none() || db_path.as_ref().unwrap().is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(db_path.unwrap()))
    };

    let default_user = "default_user";

    // 1. Initialize the app/repo with the db_path
    crate::app::init_app(db_path)?;
    let service = &crate::app::app().service;

    let debug_stats = service.get_debug_stats(default_user).await?;
    if debug_stats.total_nodes_count == 0 {
        // 2. Import the graph from the asset file
        let import_stats = service.import_cbor_graph_from_bytes(kg_bytes).await?;

        // 3. Create the default user and sync their nodes
        service.sync_user_nodes(default_user).await?;

        Ok(format!(
            "Setup complete. Imported {} nodes and {} edges.",
            import_stats.nodes_imported, import_stats.edges_imported
        ))
    } else {
        Ok(format!(
            "Setup complete. Re-used existing DB (nodes={}, edges={})",
            debug_stats.total_nodes_count, debug_stats.total_edges_count,
        ))
    }
}

pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String> {
    setup_database(None, kg_bytes).await
}

pub async fn get_exercises(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
) -> Result<Vec<Exercise>> {
    let due_nodes = crate::app::app()
        .service
        .get_due_items(&user_id, limit * 2, surah_filter) // Get extra in case some fail to generate
        .await?;
    let word_instances = due_nodes
        .iter()
        .filter(|n| matches!(n.node_type, crate::cbor_import::NodeType::WordInstance))
        .count();
    let verses = due_nodes
        .iter()
        .filter(|n| matches!(n.node_type, crate::cbor_import::NodeType::Verse))
        .count();
    println!(
        "get_exercises: found {} nodes (word_instances={}, verses={})",
        due_nodes.len(),
        word_instances,
        verses
    );

    // Build exercises preferring MCQs for word instances when enough distractors exist
    let mut built: Vec<Exercise> = Vec::new();

    for node in due_nodes {
        if built.len() >= limit as usize {
            break;
        }
        match node.node_type {
            crate::cbor_import::NodeType::WordInstance => {
                if let Some(ex) = build_mcq_from_word_instance(&node).await? {
                    built.push(ex);
                } else if let Some(ex) = simple_map_to_exercise(node) {
                    built.push(ex);
                }
            }
            _ => {
                if let Some(ex) = simple_map_to_exercise(node) {
                    built.push(ex);
                }
            }
        }
    }

    let counts = (
        built
            .iter()
            .filter(|e| matches!(e, Exercise::Recall { .. }))
            .count(),
        built
            .iter()
            .filter(|e| matches!(e, Exercise::Cloze { .. }))
            .count(),
        built
            .iter()
            .filter(|e| matches!(e, Exercise::McqArToEn { .. }))
            .count(),
        built
            .iter()
            .filter(|e| matches!(e, Exercise::McqEnToAr { .. }))
            .count(),
    );
    println!(
        "get_exercises: built={} (recall={}, cloze={}, mcq_ar_en={}, mcq_en_ar={})",
        built.len(),
        counts.0,
        counts.1,
        counts.2,
        counts.3
    );
    Ok(built)
}

fn simple_map_to_exercise(node_data: NodeData) -> Option<Exercise> {
    match node_data.node_type {
        crate::cbor_import::NodeType::WordInstance => {
            let arabic = node_data.metadata.get("arabic")?.clone();
            let translation = node_data.metadata.get("translation")?.clone();
            Some(Exercise::Recall {
                node_id: node_data.id,
                arabic,
                translation,
            })
        }
        crate::cbor_import::NodeType::Verse => {
            let arabic = node_data.metadata.get("arabic")?.clone();
            let words: Vec<&str> = arabic.split_whitespace().collect();
            if words.len() < 2 {
                return None;
            }
            let mut cloze_words = words.clone();
            cloze_words[1] = "______";
            Some(Exercise::Cloze {
                node_id: node_data.id,
                question: cloze_words.join(" "),
                answer: arabic,
            })
        }
        _ => None,
    }
}

async fn build_mcq_from_word_instance(node_data: &NodeData) -> Result<Option<Exercise>> {
    let ctx: WordInstanceContext = match crate::app::app()
        .service
        .get_word_instance_context(&node_data.id)
        .await
    {
        Ok(c) => c,
        Err(err) => {
            tracing::debug!(node_id = %node_data.id, error = %err, "MCQ skip: failed to load context");
            return Ok(None);
        }
    };

    if ctx.arabic.trim().is_empty() || ctx.translation.trim().is_empty() {
        tracing::debug!(node_id = %ctx.node_id, "MCQ skip: missing arabic or translation metadata");
        return Ok(None);
    }

    // Collect candidate distractors from the same verse, excluding identicals/empties
    let mut en_candidates: Vec<String> = ctx
        .verse_word_en_list
        .iter()
        .filter(|t| !t.is_empty() && **t != ctx.translation)
        .cloned()
        .collect();
    en_candidates.sort();
    en_candidates.dedup();

    let mut ar_candidates: Vec<String> = ctx
        .verse_word_ar_list
        .iter()
        .filter(|t| !t.is_empty() && **t != ctx.arabic)
        .cloned()
        .collect();
    ar_candidates.sort();
    ar_candidates.dedup();

    tracing::debug!(node_id = %ctx.node_id, en_candidates = en_candidates.len(), ar_candidates = ar_candidates.len(), "MCQ candidates collected");

    // Need at least 3 distractors to form 4 choices
    if en_candidates.len() >= 3 {
        let mut rng = rng();
        let mut choices = en_candidates;
        choices.shuffle(&mut rng);
        if choices.len() > 3 {
            choices.truncate(3);
        }
        // Insert correct answer at random index
        let correct_idx: usize = rng.random_range(0..=3);
        // Ensure capacity: if less than 3 after dedup, bail
        if choices.len() < 3 {
            tracing::debug!(node_id = %ctx.node_id, "MCQ skip: not enough unique English distractors after shuffle");
            return Ok(None);
        }
        let mut final_choices = choices;
        final_choices.insert(correct_idx, ctx.translation.clone());
        if final_choices.len() != 4 {
            tracing::debug!(node_id = %ctx.node_id, choices = final_choices.len(), "MCQ skip: incorrect final choice count for Ar→En");
            return Ok(None);
        }
        return Ok(Some(Exercise::McqArToEn {
            node_id: ctx.node_id,
            arabic: ctx.arabic,
            verse_arabic: ctx.verse_arabic,
            surah_number: ctx.surah_number,
            ayah_number: ctx.ayah_number,
            word_index: ctx.word_index,
            choices_en: final_choices,
            correct_index: correct_idx as i32,
        }));
    }

    if ar_candidates.len() >= 3 {
        let mut rng = rng();
        let mut choices = ar_candidates;
        choices.shuffle(&mut rng);
        if choices.len() > 3 {
            choices.truncate(3);
        }
        if choices.len() < 3 {
            tracing::debug!(node_id = %ctx.node_id, "MCQ skip: not enough unique Arabic distractors after shuffle");
            return Ok(None);
        }
        let correct_idx: usize = rng.random_range(0..=3);
        let mut final_choices = choices;
        final_choices.insert(correct_idx, ctx.arabic.clone());
        if final_choices.len() != 4 {
            tracing::debug!(node_id = %ctx.node_id, choices = final_choices.len(), "MCQ skip: incorrect final choice count for En→Ar");
            return Ok(None);
        }
        return Ok(Some(Exercise::McqEnToAr {
            node_id: ctx.node_id,
            english: ctx.translation,
            verse_arabic: ctx.verse_arabic,
            surah_number: ctx.surah_number,
            ayah_number: ctx.ayah_number,
            word_index: ctx.word_index,
            choices_ar: final_choices,
            correct_index: correct_idx as i32,
        }));
    }

    tracing::debug!(node_id = %ctx.node_id, "MCQ skip: insufficient distractors in either language");
    Ok(None)
}

pub async fn process_review(
    user_id: String,
    node_id: String,
    grade: ReviewGrade,
) -> Result<MemoryState> {
    crate::app::app()
        .service
        .process_review(&user_id, &node_id, grade)
        .await
}

pub async fn get_debug_stats(user_id: String) -> Result<DebugStats> {
    crate::app::app().service.get_debug_stats(&user_id).await
}

pub async fn reseed_database() -> Result<String> {
    // Call the repo's seed method which deletes tables and reseeds
    crate::app::app()
        .service
        .reset_user_progress("default_user")
        .await?;
    Ok("User progress reset successfully".to_string())
}

pub async fn refresh_priority_scores(user_id: String) -> Result<String> {
    crate::app::app()
        .service
        .refresh_all_priority_scores(&user_id)
        .await?;
    Ok("Priority scores refreshed".to_string())
}

pub async fn get_session_preview(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
) -> Result<Vec<ItemPreview>> {
    crate::app::app()
        .service
        .get_session_preview(&user_id, limit, surah_filter)
        .await
}

/// Search node IDs by prefix (used for sandbox suggestions)
pub async fn search_nodes(query: String, limit: u32) -> Result<Vec<NodeData>> {
    crate::app::app().service.search_nodes(&query, limit).await
}

/// Fetch a single node with its metadata by ID
pub async fn fetch_node_with_metadata(node_id: String) -> Result<Option<NodeData>> {
    crate::app::app()
        .service
        .get_node_with_metadata(&node_id)
        .await
}

// Build exercises for a specific node id (sandbox)
pub async fn get_exercises_for_node(node_id: String) -> Result<Vec<Exercise>> {
    let node_opt = crate::app::app()
        .service
        .get_node_with_metadata(&node_id)
        .await?;
    let Some(node) = node_opt else {
        return Ok(vec![]);
    };
    let mut out: Vec<Exercise> = Vec::new();
    match node.node_type {
        crate::cbor_import::NodeType::WordInstance => {
            if let Ok(Some(mcq)) = build_mcq_from_word_instance(&node).await {
                out.push(mcq);
            }
            if let Some(rec) = simple_map_to_exercise(node) {
                out.push(rec);
            }
        }
        crate::cbor_import::NodeType::Verse => {
            if let Some(cloze) = simple_map_to_exercise(node) {
                out.push(cloze);
            }
        }
        _ => {}
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name: String,
}

pub async fn get_available_surahs() -> Result<Vec<SurahInfo>> {
    let surahs = crate::app::app().service.get_available_surahs().await?;

    Ok(surahs
        .into_iter()
        .map(|(number, name)| SurahInfo { number, name })
        .collect())
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    LOG_INIT.call_once(|| {
        if tracing_subscriber::fmt::try_init().is_err() {
            tracing::debug!("tracing subscriber already initialized");
        }
    });
    flutter_rust_bridge::setup_default_user_utils();
}
