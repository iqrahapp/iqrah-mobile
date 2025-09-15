// src/exercises.rs
use crate::{cbor_import::NodeType, repository::NodeData};
use anyhow::Result;

#[derive(Debug)]
pub struct Exercise {
    pub node_id: String,
    pub arabic: String,
    pub translation: String,
    pub exercise_type: String, // "recall" or "cloze"
}

pub fn create_exercise(node_data: NodeData) -> Result<Exercise> {
    match node_data.node_type {
        NodeType::WordInstance => {
            let arabic = node_data
                .metadata
                .get("arabic")
                .ok_or_else(|| anyhow::anyhow!("Missing arabic text"))?;
            let translation = node_data
                .metadata
                .get("translation")
                .ok_or_else(|| anyhow::anyhow!("Missing translation"))?;

            Ok(Exercise {
                node_id: node_data.id,
                arabic: arabic.clone(),
                translation: translation.clone(),
                exercise_type: "recall".to_string(),
            })
        }
        NodeType::Verse => {
            let arabic = node_data
                .metadata
                .get("arabic")
                .ok_or_else(|| anyhow::anyhow!("Missing arabic text for verse"))?;

            let words: Vec<&str> = arabic.split_whitespace().collect();
            if words.len() < 2 {
                return Err(anyhow::anyhow!("Verse too short for cloze"));
            }

            let mut cloze_words = words.clone();
            cloze_words[1] = "______";

            Ok(Exercise {
                node_id: node_data.id,
                arabic: cloze_words.join(" "),
                translation: arabic.to_string(),
                exercise_type: "cloze".to_string(),
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported node type for exercises")),
    }
}
