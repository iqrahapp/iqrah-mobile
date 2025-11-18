// exercises/generators.rs
// Generator functions for creating ExerciseData instances
//
// Each generator:
// 1. Takes a node_id and ContentRepository reference
// 2. Queries database for minimal metadata (IDs, positions, keys)
// 3. Returns ExerciseData with ONLY keys/IDs (no full text)
// 4. Full text is fetched later during question generation

use super::exercise_data::ExerciseData;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::Rng;

// ============================================================================
// Memorization Exercises
// ============================================================================

/// Generate a Memorization exercise
///
/// Stores only the node_id, fetches no text
pub async fn generate_memorization(
    node_id: String,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // No need to fetch anything - just store the node_id
    // Text will be fetched during question generation
    Ok(ExerciseData::Memorization { node_id })
}

// ============================================================================
// MCQ Exercises
// ============================================================================

/// Generate MCQ Arabic to English exercise
///
/// Stores word node_id and distractor node_ids
pub async fn generate_mcq_ar_to_en(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Parse knowledge node to get base content
    let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node_id) {
        kn.base_node_id
    } else {
        node_id.clone()
    };

    // Parse node_id to extract verse location
    // Format: "WORD_INSTANCE:chapter:verse:position"
    let parts: Vec<&str> = base_node_id.split(':').collect();
    if parts.len() != 4 {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_node_id
        ));
    }

    let chapter: i32 = parts[1].parse()?;
    let verse: i32 = parts[2].parse()?;
    let position: i32 = parts[3].parse()?;

    // Get other words from the same verse or nearby verses as distractors
    let verse_key = format!("{}:{}", chapter, verse);
    let words = content_repo.get_words_for_verse(&verse_key).await?;

    // Collect distractor node IDs (exclude the target word)
    let mut distractor_node_ids: Vec<String> = words
        .iter()
        .filter(|w| w.position != position)
        .map(|w| format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, w.position))
        .collect();

    // If not enough distractors, get from adjacent verses
    if distractor_node_ids.len() < 3 {
        // Try next verse
        if let Ok(next_words) = content_repo
            .get_words_for_verse(&format!("{}:{}", chapter, verse + 1))
            .await
        {
            for word in next_words.iter().take(3 - distractor_node_ids.len()) {
                distractor_node_ids.push(format!(
                    "WORD_INSTANCE:{}:{}:{}",
                    chapter,
                    verse + 1,
                    word.position
                ));
            }
        }
    }

    // Randomly select 3 distractors
    let mut rng = rand::thread_rng();
    distractor_node_ids.shuffle(&mut rng);
    distractor_node_ids.truncate(3);

    Ok(ExerciseData::McqArToEn {
        node_id,
        distractor_node_ids,
    })
}

/// Generate MCQ English to Arabic exercise
pub async fn generate_mcq_en_to_ar(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Similar to ar_to_en, but question shows English, answer is Arabic
    // For now, use same distractor generation logic
    let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node_id) {
        kn.base_node_id
    } else {
        node_id.clone()
    };

    let parts: Vec<&str> = base_node_id.split(':').collect();
    if parts.len() != 4 {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_node_id
        ));
    }

    let chapter: i32 = parts[1].parse()?;
    let verse: i32 = parts[2].parse()?;
    let position: i32 = parts[3].parse()?;

    let verse_key = format!("{}:{}", chapter, verse);
    let words = content_repo.get_words_for_verse(&verse_key).await?;

    let mut distractor_node_ids: Vec<String> = words
        .iter()
        .filter(|w| w.position != position)
        .map(|w| format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, w.position))
        .collect();

    let mut rng = rand::thread_rng();
    distractor_node_ids.shuffle(&mut rng);
    distractor_node_ids.truncate(3);

    Ok(ExerciseData::McqEnToAr {
        node_id,
        distractor_node_ids,
    })
}

// ============================================================================
// Translation Exercises
// ============================================================================

/// Generate Translation exercise (word-level)
pub async fn generate_translation(
    node_id: String,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Store only node_id, fetch text during generation
    Ok(ExerciseData::Translation { node_id })
}

/// Generate Contextual Translation exercise (with verse context)
pub async fn generate_contextual_translation(
    node_id: String,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Parse to extract verse_key
    let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node_id) {
        kn.base_node_id
    } else {
        node_id.clone()
    };

    let parts: Vec<&str> = base_node_id.split(':').collect();
    if parts.len() != 4 {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_node_id
        ));
    }

    let verse_key = format!("{}:{}", parts[1], parts[2]);

    Ok(ExerciseData::ContextualTranslation { node_id, verse_key })
}

// ============================================================================
// Cloze/Fill-in Exercises
// ============================================================================

/// Generate Cloze Deletion exercise
pub async fn generate_cloze_deletion(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Parse verse_key from node_id
    let verse_key = node_id
        .strip_prefix("VERSE:")
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", node_id))?
        .to_string();

    // Get words to determine valid blank positions
    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 3 {
        return Err(anyhow::anyhow!(
            "Verse too short for cloze deletion (need at least 3 words)"
        ));
    }

    // Randomly select a word position to blank (avoid first and last)
    let mut rng = rand::thread_rng();
    let blank_position = if words.len() > 3 {
        rng.gen_range(2..words.len()) as i32
    } else {
        2 // For 3-word verses, always blank the middle
    };

    Ok(ExerciseData::ClozeDeletion {
        node_id,
        blank_position,
    })
}

/// Generate First Letter Hint exercise
pub async fn generate_first_letter_hint(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let verse_key = node_id
        .strip_prefix("VERSE:")
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", node_id))?
        .to_string();

    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.is_empty() {
        return Err(anyhow::anyhow!("No words found for verse: {}", verse_key));
    }

    // Randomly select a word position
    let mut rng = rand::thread_rng();
    let word_position = rng.gen_range(1..=words.len()) as i32;

    Ok(ExerciseData::FirstLetterHint {
        node_id,
        word_position,
    })
}

/// Generate Missing Word MCQ exercise
pub async fn generate_missing_word_mcq(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let verse_key = node_id
        .strip_prefix("VERSE:")
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", node_id))?
        .to_string();

    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 4 {
        return Err(anyhow::anyhow!(
            "Verse too short for missing word MCQ (need at least 4 words)"
        ));
    }

    let mut rng = rand::thread_rng();
    let blank_position = rng.gen_range(2..words.len()) as i32;

    // Get distractors from the same or nearby verses
    let parts: Vec<&str> = verse_key.split(':').collect();
    let chapter: i32 = parts[0].parse()?;
    let verse: i32 = parts[1].parse()?;

    let mut distractor_node_ids: Vec<String> = words
        .iter()
        .filter(|w| w.position != blank_position)
        .map(|w| format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, w.position))
        .collect();

    distractor_node_ids.shuffle(&mut rng);
    distractor_node_ids.truncate(3);

    Ok(ExerciseData::MissingWordMcq {
        node_id,
        blank_position,
        distractor_node_ids,
    })
}

/// Generate Next Word MCQ exercise
pub async fn generate_next_word_mcq(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let verse_key = node_id
        .strip_prefix("VERSE:")
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", node_id))?
        .to_string();

    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 3 {
        return Err(anyhow::anyhow!(
            "Verse too short for next word MCQ (need at least 3 words)"
        ));
    }

    // Context position is before the target word
    let mut rng = rand::thread_rng();
    let context_position = rng.gen_range(1..words.len()) as i32;

    // Get distractors
    let parts: Vec<&str> = verse_key.split(':').collect();
    let chapter: i32 = parts[0].parse()?;
    let verse: i32 = parts[1].parse()?;

    let mut distractor_node_ids: Vec<String> = words
        .iter()
        .filter(|w| w.position != context_position + 1)
        .map(|w| format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, w.position))
        .collect();

    distractor_node_ids.shuffle(&mut rng);
    distractor_node_ids.truncate(3);

    Ok(ExerciseData::NextWordMcq {
        node_id,
        context_position,
        distractor_node_ids,
    })
}

// ============================================================================
// Verse-Level Exercises
// ============================================================================

/// Generate Full Verse Input exercise
pub async fn generate_full_verse_input(
    node_id: String,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Store only node_id
    Ok(ExerciseData::FullVerseInput { node_id })
}

/// Generate Ayah Chain exercise
pub async fn generate_ayah_chain(
    chapter_node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Parse chapter number
    let chapter_num: i32 = chapter_node_id
        .strip_prefix("CHAPTER:")
        .ok_or_else(|| anyhow::anyhow!("Invalid chapter node ID: {}", chapter_node_id))?
        .parse()?;

    // Get all verses for the chapter (just keys, not full text)
    let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

    let verse_keys: Vec<String> = verses.iter().map(|v| v.key.clone()).collect();

    if verse_keys.is_empty() {
        return Err(anyhow::anyhow!(
            "No verses found for chapter {}",
            chapter_num
        ));
    }

    Ok(ExerciseData::AyahChain {
        node_id: chapter_node_id,
        verse_keys,
        current_index: 0,
        completed_count: 0,
    })
}

/// Generate Ayah Chain for a specific range
pub async fn generate_ayah_chain_range(
    chapter_num: i32,
    start_verse: i32,
    end_verse: i32,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let all_verses = content_repo.get_verses_for_chapter(chapter_num).await?;

    let verse_keys: Vec<String> = all_verses
        .into_iter()
        .filter(|v| v.verse_number >= start_verse && v.verse_number <= end_verse)
        .map(|v| v.key)
        .collect();

    if verse_keys.is_empty() {
        return Err(anyhow::anyhow!(
            "No verses found for chapter {} range {}:{}",
            chapter_num,
            start_verse,
            end_verse
        ));
    }

    let node_id = format!("CHAPTER:{}:{}:{}", chapter_num, start_verse, end_verse);

    Ok(ExerciseData::AyahChain {
        node_id,
        verse_keys,
        current_index: 0,
        completed_count: 0,
    })
}

// ============================================================================
// Advanced Exercises
// ============================================================================

/// Generate Find the Mistake exercise
pub async fn generate_find_mistake(
    verse_node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let verse_key = verse_node_id
        .strip_prefix("VERSE:")
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", verse_node_id))?
        .to_string();

    let parts: Vec<&str> = verse_key.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid verse key format: {}", verse_key));
    }
    let chapter_num: i32 = parts[0].parse()?;
    let verse_num: i32 = parts[1].parse()?;

    // Get words for the verse
    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 3 {
        return Err(anyhow::anyhow!(
            "Verse too short for Find the Mistake (need at least 3 words)"
        ));
    }

    // Select random position (avoid first and last)
    let mut rng = rand::thread_rng();
    let mistake_position: i32 = if words.len() > 3 {
        (rng.gen_range(1..words.len() - 1) + 1) as i32
    } else {
        2
    };

    // Get correct word node ID
    let correct_word = words
        .iter()
        .find(|w| w.position == mistake_position)
        .ok_or_else(|| anyhow::anyhow!("Word not found at position {}", mistake_position))?;

    let correct_word_node_id = format!(
        "WORD_INSTANCE:{}:{}:{}",
        chapter_num, verse_num, correct_word.position
    );

    // Get a replacement word from another verse
    let all_verses = content_repo.get_verses_for_chapter(chapter_num).await?;
    let mut candidate_words = Vec::new();

    for verse in &all_verses {
        if verse.key != verse_key {
            let other_words = content_repo.get_words_for_verse(&verse.key).await?;
            for word in other_words {
                if word.text_uthmani != correct_word.text_uthmani {
                    let parts: Vec<&str> = verse.key.split(':').collect();
                    let word_node_id =
                        format!("WORD_INSTANCE:{}:{}:{}", parts[0], parts[1], word.position);
                    candidate_words.push(word_node_id);
                }
            }
        }
    }

    if candidate_words.is_empty() {
        return Err(anyhow::anyhow!(
            "No suitable replacement words found in chapter {}",
            chapter_num
        ));
    }

    let incorrect_word_node_id = candidate_words
        .choose(&mut rng)
        .ok_or_else(|| anyhow::anyhow!("Failed to select random word"))?
        .clone();

    Ok(ExerciseData::FindMistake {
        node_id: verse_node_id,
        mistake_position,
        correct_word_node_id,
        incorrect_word_node_id,
    })
}

/// Generate Ayah Sequence exercise
pub async fn generate_ayah_sequence(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Can be chapter-level or verse-level (word sequence)
    if node_id.starts_with("CHAPTER:") {
        // Sequence of verses
        let chapter_num: i32 = node_id
            .strip_prefix("CHAPTER:")
            .ok_or_else(|| anyhow::anyhow!("Invalid chapter node ID"))?
            .parse()?;

        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // For now, take first 5 verses
        let correct_sequence: Vec<String> = verses
            .iter()
            .take(5)
            .map(|v| format!("VERSE:{}", v.key))
            .collect();

        if correct_sequence.len() < 2 {
            return Err(anyhow::anyhow!("Not enough verses for sequence exercise"));
        }

        Ok(ExerciseData::AyahSequence {
            node_id,
            correct_sequence,
        })
    } else if node_id.starts_with("VERSE:") {
        // Sequence of words within a verse
        let verse_key = node_id
            .strip_prefix("VERSE:")
            .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID"))?;

        let words = content_repo.get_words_for_verse(verse_key).await?;

        let parts: Vec<&str> = verse_key.split(':').collect();
        let chapter: i32 = parts[0].parse()?;
        let verse: i32 = parts[1].parse()?;

        let correct_sequence: Vec<String> = words
            .iter()
            .map(|w| format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, w.position))
            .collect();

        if correct_sequence.len() < 3 {
            return Err(anyhow::anyhow!(
                "Verse too short for word sequence exercise"
            ));
        }

        Ok(ExerciseData::AyahSequence {
            node_id,
            correct_sequence,
        })
    } else {
        Err(anyhow::anyhow!(
            "Invalid node ID format for sequence exercise: {}",
            node_id
        ))
    }
}

/// Generate Identify Root exercise
pub async fn generate_identify_root(
    word_node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_node_id = if let Some(kn) = KnowledgeNode::parse(&word_node_id) {
        kn.base_node_id
    } else {
        word_node_id.clone()
    };

    // Parse to get verse_key and position
    let parts: Vec<&str> = base_node_id.split(':').collect();
    if parts.len() != 4 {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_node_id
        ));
    }

    let verse_key = format!("{}:{}", parts[1], parts[2]);
    let position: i32 = parts[3].parse()?;

    // Get word from database
    let words = content_repo.get_words_for_verse(&verse_key).await?;
    let word = words
        .iter()
        .find(|w| w.position == position)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Word not found at position {} in verse {}",
                position,
                verse_key
            )
        })?;

    // Get morphology to extract root_id
    let morphology = content_repo.get_morphology_for_word(word.id).await?;

    let root_id = morphology
        .iter()
        .find_map(|seg| seg.root_id.clone())
        .ok_or_else(|| anyhow::anyhow!("No root_id found for word ID {}", word.id))?;

    // Fetch the Root object to get the actual root string
    let root_obj = content_repo
        .get_root_by_id(&root_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Root not found: {}", root_id))?;

    let root = root_obj.root_id;

    Ok(ExerciseData::IdentifyRoot {
        node_id: word_node_id,
        root,
    })
}

/// Generate Reverse Cloze exercise
pub async fn generate_reverse_cloze(
    node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let verse_key = node_id
        .strip_prefix("VERSE:")
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", node_id))?
        .to_string();

    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 3 {
        return Err(anyhow::anyhow!(
            "Verse too short for reverse cloze (need at least 3 words)"
        ));
    }

    let mut rng = rand::thread_rng();
    let blank_position = rng.gen_range(2..=words.len()) as i32;

    Ok(ExerciseData::ReverseCloze {
        node_id,
        blank_position,
    })
}

/// Generate Translate Phrase exercise
pub async fn generate_translate_phrase(
    node_id: String,
    translator_id: i32,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Store node_id and translator_id only
    Ok(ExerciseData::TranslatePhrase {
        node_id,
        translator_id,
    })
}

/// Generate POS Tagging exercise
pub async fn generate_pos_tagging(
    word_node_id: String,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_node_id = if let Some(kn) = KnowledgeNode::parse(&word_node_id) {
        kn.base_node_id
    } else {
        word_node_id.clone()
    };

    let parts: Vec<&str> = base_node_id.split(':').collect();
    if parts.len() != 4 {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_node_id
        ));
    }

    let verse_key = format!("{}:{}", parts[1], parts[2]);
    let position: i32 = parts[3].parse()?;

    let words = content_repo.get_words_for_verse(&verse_key).await?;
    let word = words
        .iter()
        .find(|w| w.position == position)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Word not found at position {} in verse {}",
                position,
                verse_key
            )
        })?;

    // Get morphology for POS tag
    let morphology = content_repo.get_morphology_for_word(word.id).await?;

    let correct_pos = morphology
        .iter()
        .find_map(|seg| seg.pos_tag.clone())
        .ok_or_else(|| anyhow::anyhow!("No POS tag found for word ID {}", word.id))?;

    // Generate options
    let all_pos = vec![
        "noun".to_string(),
        "verb".to_string(),
        "particle".to_string(),
        "pronoun".to_string(),
    ];

    let mut options = vec![correct_pos.clone()];
    for pos in &all_pos {
        if pos.to_lowercase() != correct_pos.to_lowercase() && options.len() < 4 {
            options.push(pos.clone());
        }
    }

    let mut rng = rand::thread_rng();
    options.shuffle(&mut rng);

    Ok(ExerciseData::PosTagging {
        node_id: word_node_id,
        correct_pos,
        options,
    })
}

/// Generate Cross-Verse Connection exercise
pub async fn generate_cross_verse_connection(
    node_id: String,
    related_verse_ids: Vec<String>,
    connection_theme: String,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Store IDs only
    Ok(ExerciseData::CrossVerseConnection {
        node_id,
        related_verse_ids,
        connection_theme,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_memorization() {
        // Synchronous test - no async needed
        let node_id = "WORD_INSTANCE:1:1:1".to_string();
        let exercise = ExerciseData::Memorization {
            node_id: node_id.clone(),
        };
        assert_eq!(exercise.node_id(), &node_id);
    }

    #[test]
    fn test_generate_translation() {
        let node_id = "WORD_INSTANCE:1:1:1".to_string();
        let exercise = ExerciseData::Translation {
            node_id: node_id.clone(),
        };
        assert_eq!(exercise.node_id(), &node_id);
    }

    #[test]
    fn test_generate_full_verse_input() {
        let node_id = "VERSE:1:1".to_string();
        let exercise = ExerciseData::FullVerseInput {
            node_id: node_id.clone(),
        };
        assert_eq!(exercise.node_id(), &node_id);
    }
}
