// exercises/full_verse_input.rs
// Exercise 9: Full Verse Input - Given verse number, type entire verse

use super::memorization::MemorizationExercise;
use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;

// ============================================================================
// Exercise 9: Full Verse Input
// ============================================================================

/// Exercise for testing complete verse memorization
/// Given a verse reference (e.g., "Al-Fatihah verse 1"), user must type the entire verse
/// Tests fluent production and full verse recall
#[derive(Debug)]
pub struct FullVerseInputExercise {
    node_id: i64,
    verse_key: String,
    chapter_name: String,
    verse_number: i32,
    correct_verse_text: String,
}

impl FullVerseInputExercise {
    /// Create a new Full Verse Input exercise
    ///
    /// Queries the database for:
    /// - Verse text (correct answer)
    /// - Chapter name (for question prompt)
    pub async fn new(verse_node_id: i64, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Get the node to access its ukey
        let node = content_repo
            .get_node(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", verse_node_id))?;

        // Parse knowledge node
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node.ukey) {
            kn.base_node_id
        } else {
            node.ukey.clone()
        };

        // Parse node_id to get verse_key
        // Format: "VERSE:chapter:verse"
        let parts: Vec<&str> = base_node_id.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!(
                "Invalid verse node ID format: {}",
                base_node_id
            ));
        }

        let chapter_num: i32 = parts[1].parse()?;
        let verse_num: i32 = parts[2].parse()?;
        let verse_key = format!("{}:{}", chapter_num, verse_num);

        // Get verse text
        let correct_verse_text = content_repo
            .get_quran_text(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", verse_node_id))?;

        // Get chapter name
        let chapter_ukey = format!("CHAPTER:{}", chapter_num);
        let chapter_node = content_repo
            .get_node_by_ukey(&chapter_ukey)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Chapter node not found: {}", chapter_ukey))?;

        let chapter_name = content_repo
            .get_quran_text(chapter_node.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Chapter name not found: {}", chapter_ukey))?;

        Ok(Self {
            node_id: verse_node_id,
            verse_key,
            chapter_name,
            verse_number: verse_num,
            correct_verse_text,
        })
    }

    /// Get the correct verse text (for testing)
    pub fn get_correct_verse(&self) -> &str {
        &self.correct_verse_text
    }

    /// Get verse reference (for testing)
    pub fn get_verse_key(&self) -> &str {
        &self.verse_key
    }
}

impl Exercise for FullVerseInputExercise {
    fn generate_question(&self) -> String {
        format!(
            "Type the complete verse:\n\n{}, verse {}",
            self.chapter_name, self.verse_number
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Use Arabic normalization (same as MemorizationExercise)
        let normalized_answer = MemorizationExercise::normalize_arabic(answer);
        let normalized_correct = MemorizationExercise::normalize_arabic(&self.correct_verse_text);

        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Show first word of the verse as a hint
        let normalized = MemorizationExercise::normalize_arabic(&self.correct_verse_text);
        let first_word = normalized.split_whitespace().next()?;
        Some(format!("Starts with: {}", first_word))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "full_verse_input"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Full verse input tests require database setup
    }
}
