// Review processing with FSRS + energy propagation

use iqrah_core::{ContentRepository, UserRepository, MemoryState, PropagationEvent, PropagationDetail};
use crate::types::ReviewGrade;
use anyhow::Result;
use chrono::{Utc, Duration};

/// Process a review: update FSRS state, calculate energy delta, propagate
pub async fn process_review(
    content_repo: &dyn ContentRepository,
    user_repo: &dyn UserRepository,
    user_id: &str,
    node_id: &str,
    grade: ReviewGrade,
) -> Result<()> {
    // 1. Get or create memory state
    let mut state = match user_repo.get_memory_state(user_id, node_id).await? {
        Some(s) => s,
        None => MemoryState::new_for_node(user_id.to_string(), node_id.to_string()),
    };

    // 2. Calculate days elapsed since last review
    let now = Utc::now();
    let days_elapsed = if state.review_count > 0 {
        let elapsed = now.signed_duration_since(state.last_reviewed);
        elapsed.num_days().max(0) as u32
    } else {
        0
    };

    // 3. Update FSRS state
    let fsrs = fsrs::FSRS::new(None)?;
    let current_memory = if state.review_count > 0 {
        Some(fsrs::MemoryState {
            stability: state.stability as f32,
            difficulty: state.difficulty as f32,
        })
    } else {
        None
    };

    // Use next_states to get all possible outcomes
    let next_states = fsrs.next_states(
        current_memory,
        0.9, // desired retention (90%)
        days_elapsed,
    )?;

    // Select the appropriate state based on grade
    let chosen_state = match grade {
        ReviewGrade::Again => &next_states.again,
        ReviewGrade::Hard => &next_states.hard,
        ReviewGrade::Good => &next_states.good,
        ReviewGrade::Easy => &next_states.easy,
    };

    // 4. Calculate energy delta
    let old_energy = state.energy;
    let energy_delta = match grade {
        ReviewGrade::Easy => 0.2,
        ReviewGrade::Good => 0.1,
        ReviewGrade::Hard => -0.05,
        ReviewGrade::Again => -0.15,
    };

    let new_energy = (old_energy + energy_delta).clamp(0.0, 1.0);

    // 5. Update state
    state.stability = chosen_state.memory.stability as f64;
    state.difficulty = chosen_state.memory.difficulty as f64;
    state.energy = new_energy;
    state.last_reviewed = now;
    state.due_at = now + Duration::days(chosen_state.interval as i64);
    state.review_count += 1;

    // 6. Save updated state
    user_repo.save_memory_state(&state).await?;

    // 7. Propagate energy if significant change
    if energy_delta.abs() > 0.01 {
        propagate_energy(content_repo, user_repo, user_id, node_id, energy_delta).await?;
    }

    // 8. Update stats
    let current_reviews = user_repo.get_stat("reviews_today").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    user_repo.set_stat("reviews_today", &(current_reviews + 1).to_string()).await?;

    Ok(())
}

/// Propagate energy through the knowledge graph
async fn propagate_energy(
    content_repo: &dyn ContentRepository,
    user_repo: &dyn UserRepository,
    user_id: &str,
    source_node_id: &str,
    energy_delta: f64,
) -> Result<()> {
    // Get outgoing edges
    let edges = content_repo.get_edges_from(source_node_id).await?;

    let mut details = Vec::new();

    for edge in edges {
        // Calculate propagated energy based on edge parameters
        let propagated = energy_delta * edge.param1; // Simplified propagation

        // Update target node energy if it has a memory state
        if let Some(target_state) = user_repo.get_memory_state(user_id, &edge.target_id).await? {
            let new_energy = (target_state.energy + propagated).clamp(0.0, 1.0);
            user_repo.update_energy(user_id, &edge.target_id, new_energy).await?;

            details.push(PropagationDetail {
                target_node_id: edge.target_id.clone(),
                energy_change: propagated,
                reason: "Propagated".to_string(),
            });
        }
    }

    // Log propagation event
    if !details.is_empty() {
        let event = PropagationEvent {
            id: None,
            source_node_id: source_node_id.to_string(),
            event_timestamp: Utc::now(),
            details,
        };

        user_repo.log_propagation(&event).await?;
    }

    Ok(())
}
