// Exercise generation logic - simplified for clean break
// Full implementation can be added incrementally

use crate::types::Exercise;
use iqrah_core::{ContentRepository, UserRepository};
use std::collections::HashMap;

/// Build exercises from due items
/// Simplified version - just creates recall exercises for now
pub async fn build_exercises_from_due_items(
    _content_repo: &dyn ContentRepository,
    _user_repo: &dyn UserRepository,
    due_node_ids: Vec<String>,
    node_metadata: Vec<HashMap<String, String>>,
) -> Vec<Exercise> {
    let mut exercises = Vec::new();

    for (node_id, metadata) in due_node_ids.iter().zip(node_metadata.iter()) {
        // Simple recall exercise for now
        if let (Some(arabic), Some(translation)) = (metadata.get("arabic"), metadata.get("translation")) {
            exercises.push(Exercise::Recall {
                node_id: node_id.clone(),
                arabic: arabic.clone(),
                translation: translation.clone(),
            });
        }
    }

    exercises
}
