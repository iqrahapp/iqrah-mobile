// exercises/graph.rs
// Graph-based exercises leveraging the Knowledge Graph

use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

// ============================================================================
// Exercise 19: Cross-Verse Connection (Graph-based!)
// ============================================================================

/// Exercise for finding thematically connected verses using the Knowledge Graph
/// Leverages graph edges to find verses with semantic or grammatical connections
pub struct CrossVerseConnectionExercise {
    node_id: String,
    verse_text: String,
    #[allow(dead_code)]
    // Used for potential future features (e.g., showing source verse reference)
    verse_key: String,
    correct_verse_key: String,
    correct_verse_text: String,
    options: Vec<(String, String)>, // (verse_key, verse_text) pairs
}

impl CrossVerseConnectionExercise {
    /// Create a new cross-verse connection exercise
    ///
    /// Queries the Knowledge Graph to find:
    /// - Connected verses via graph edges
    /// - Disconnected verses for distractors
    pub async fn new(verse_node_id: String, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Parse knowledge node
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&verse_node_id) {
            kn.base_node_id
        } else {
            verse_node_id.clone()
        };

        // Extract verse key from node ID
        // Format: "VERSE:chapter:verse"
        let verse_key = base_node_id
            .strip_prefix("VERSE:")
            .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", base_node_id))?
            .to_string();

        // Get source verse text
        let verse_text = content_repo
            .get_quran_text(&base_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", base_node_id))?;

        // Find connected verses via Knowledge Graph edges
        let edges = content_repo.get_edges_from(&base_node_id).await?;

        // Filter for verse connections
        let mut connected_verse_keys = Vec::new();
        for edge in &edges {
            // Check if target is a VERSE node
            if let Some(target_verse_key) = edge.target_id.strip_prefix("VERSE:") {
                connected_verse_keys.push(target_verse_key.to_string());
            }
        }

        // If no direct verse connections, try to find verses sharing words/roots
        if connected_verse_keys.is_empty() {
            connected_verse_keys =
                Self::find_verses_sharing_roots(&verse_key, content_repo).await?;
        }

        // Select a connected verse as the correct answer
        let correct_verse_key = connected_verse_keys
            .first()
            .ok_or_else(|| anyhow::anyhow!("No connected verses found for: {}", verse_key))?
            .clone();

        let correct_verse_node = format!("VERSE:{}", correct_verse_key);
        let correct_verse_text = content_repo
            .get_quran_text(&correct_verse_node)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Connected verse text not found: {}", correct_verse_node)
            })?;

        // Generate distractors (verses NOT connected)
        let distractors =
            Self::generate_distractors(&verse_key, &connected_verse_keys, content_repo).await?;

        // Combine correct answer with distractors
        let mut options = vec![(correct_verse_key.clone(), correct_verse_text.clone())];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        Ok(Self {
            node_id: verse_node_id,
            verse_text,
            verse_key,
            correct_verse_key,
            correct_verse_text,
            options,
        })
    }

    /// Find verses that share roots/lemmas with the source verse
    async fn find_verses_sharing_roots(
        source_verse_key: &str,
        content_repo: &dyn ContentRepository,
    ) -> Result<Vec<String>> {
        let mut connected_verses = Vec::new();

        // Get words from source verse
        let source_words = content_repo.get_words_for_verse(source_verse_key).await?;

        // For each word, get its morphology and find other verses with same roots
        for word in source_words.iter().take(3) {
            // Limit to first 3 words for performance
            let morphology = content_repo.get_morphology_for_word(word.id).await?;

            for segment in morphology {
                if let Some(root_id) = segment.root_id {
                    // This is a simplified approach - in production, you'd query
                    // morphology_segments to find other words with same root,
                    // then get their verses
                    // For now, we'll use a fallback approach
                    let _ = root_id; // Suppress unused warning
                }
            }
        }

        // Fallback: Use verses from same chapter if no root connections found
        if connected_verses.is_empty() {
            let parts: Vec<&str> = source_verse_key.split(':').collect();
            if let Ok(chapter_num) = parts[0].parse::<i32>() {
                let verses = content_repo.get_verses_for_chapter(chapter_num).await?;
                for verse in verses.iter().take(10) {
                    if verse.key != source_verse_key {
                        connected_verses.push(verse.key.clone());
                        if !connected_verses.is_empty() {
                            break;
                        }
                    }
                }
            }
        }

        Ok(connected_verses)
    }

    /// Generate distractor verses (not connected to source)
    async fn generate_distractors(
        source_verse_key: &str,
        exclude_verse_keys: &[String],
        content_repo: &dyn ContentRepository,
    ) -> Result<Vec<(String, String)>> {
        let mut distractors = Vec::new();

        // Parse source chapter
        let parts: Vec<&str> = source_verse_key.split(':').collect();
        let source_chapter: i32 = parts[0].parse()?;

        // Get verses from different chapters (to ensure disconnection)
        for chapter_num in [1, 2, 3, 112, 113, 114] {
            if chapter_num == source_chapter {
                continue;
            }

            let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

            for verse in verses.iter().take(3) {
                if !exclude_verse_keys.contains(&verse.key) {
                    let verse_node_id = format!("VERSE:{}", verse.key);
                    if let Some(text) = content_repo.get_quran_text(&verse_node_id).await? {
                        distractors.push((verse.key.clone(), text));
                        if distractors.len() >= 3 {
                            return Ok(distractors);
                        }
                    }
                }
            }
        }

        // If still not enough distractors, add more from same chapter but distant verses
        if distractors.len() < 3 {
            let verses = content_repo.get_verses_for_chapter(source_chapter).await?;
            for verse in verses.iter().rev().take(5) {
                // Take from end
                if verse.key != source_verse_key && !exclude_verse_keys.contains(&verse.key) {
                    let verse_node_id = format!("VERSE:{}", verse.key);
                    if let Some(text) = content_repo.get_quran_text(&verse_node_id).await? {
                        distractors.push((verse.key.clone(), text));
                        if distractors.len() >= 3 {
                            break;
                        }
                    }
                }
            }
        }

        Ok(distractors.into_iter().take(3).collect())
    }

    /// Get the MCQ options (verse key, text pairs)
    pub fn get_options(&self) -> &[(String, String)] {
        &self.options
    }

    /// Get the correct answer verse key
    pub fn get_correct_verse_key(&self) -> &str {
        &self.correct_verse_key
    }
}

impl Exercise for CrossVerseConnectionExercise {
    fn generate_question(&self) -> String {
        format!(
            "This verse:\n\n{}\n\nWhich verse is thematically connected to this one?",
            self.verse_text
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Normalize for comparison - accept either verse key or verse text
        let normalized_answer = answer.trim();

        // Check if answer matches verse key
        if normalized_answer == self.correct_verse_key {
            return true;
        }

        // Check if answer matches verse text (for UI that displays options)
        if normalized_answer == self.correct_verse_text {
            return true;
        }

        false
    }

    fn get_hint(&self) -> Option<String> {
        // Show the chapter number of the connected verse
        let parts: Vec<&str> = self.correct_verse_key.split(':').collect();
        if let Some(chapter) = parts.first() {
            return Some(format!("The connected verse is from Surah {}", chapter));
        }
        None
    }

    fn get_node_id(&self) -> &str {
        &self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "cross_verse_connection"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Graph tests require full database setup
        // See graph_tests.rs for comprehensive tests
    }
}

// Include comprehensive integration tests
#[cfg(test)]
#[path = "graph_tests.rs"]
mod graph_tests;
