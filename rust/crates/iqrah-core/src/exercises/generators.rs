// exercises/generators.rs
// Generator functions for creating ExerciseData instances
//
// Each generator:
// 1. Takes a node_id and ContentRepository reference
// 2. Queries database for minimal metadata (IDs, positions, keys)
// 3. Returns ExerciseData with ONLY keys/IDs (no full text)
// 4. Full text is fetched later during question generation

use super::exercise_data::ExerciseData;
use crate::domain::node_id::{self, PREFIX_CHAPTER, PREFIX_VERSE, PREFIX_WORD};
use crate::{ContentRepository, KnowledgeNode, Node};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::Rng;

async fn resolve_word_location(
    base_ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<(i32, i32, i32)> {
    let parts: Vec<&str> = base_ukey.split(':').collect();
    if parts.len() == 4 {
        let chapter: i32 = parts[1].parse()?;
        let verse: i32 = parts[2].parse()?;
        let position: i32 = parts[3].parse()?;
        return Ok((chapter, verse, position));
    }

    if base_ukey.starts_with(PREFIX_WORD) {
        let word_id = node_id::parse_word(base_ukey).map_err(|e| anyhow::anyhow!(e))?;
        let word = content_repo
            .get_word(word_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word not found: {}", base_ukey))?;
        let verse_parts: Vec<&str> = word.verse_key.split(':').collect();
        if verse_parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid verse key `{}` for word {}",
                word.verse_key,
                word_id
            ));
        }
        let chapter: i32 = verse_parts[0].parse()?;
        let verse: i32 = verse_parts[1].parse()?;
        return Ok((chapter, verse, word.position));
    }

    Err(anyhow::anyhow!("Invalid word ukey format: {}", base_ukey))
}

// ============================================================================
// Memorization Exercises
// ============================================================================

/// Generate a Memorization exercise
///
/// Stores only the node_id, fetches no text
pub async fn generate_memorization(
    node_id: i64,
    _ukey: &str,
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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Parse ukey to get base content
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let (chapter, verse, position) = resolve_word_location(&base_ukey, content_repo).await?;

    // Get other words from the same verse or nearby verses as distractors
    let verse_key = format!("{}:{}", chapter, verse);
    let words = content_repo.get_words_for_verse(&verse_key).await?;

    // Collect distractor node IDs (exclude the target word)
    let mut distractor_nodes: Vec<Node> = Vec::new();

    for word in words.iter().filter(|w| w.position != position) {
        let distractor_ukey =
            node_id::word_instance(chapter as u8, verse as u16, word.position as u8);
        if let Some(node) = content_repo.get_node_by_ukey(&distractor_ukey).await? {
            distractor_nodes.push(node);
        }
    }

    // If not enough distractors, get from adjacent verses
    if distractor_nodes.len() < 3 {
        if let Ok(next_words) = content_repo
            .get_words_for_verse(&format!("{}:{}", chapter, verse + 1))
            .await
        {
            for word in next_words.iter().take(3 - distractor_nodes.len()) {
                let distractor_ukey =
                    node_id::word_instance(chapter as u8, (verse + 1) as u16, word.position as u8);
                if let Some(node) = content_repo.get_node_by_ukey(&distractor_ukey).await? {
                    distractor_nodes.push(node);
                }
            }
        }
    }

    // Randomly select 3 distractors
    let mut rng = rand::thread_rng();
    distractor_nodes.shuffle(&mut rng);
    let distractor_node_ids: Vec<i64> =
        distractor_nodes.into_iter().map(|n| n.id).take(3).collect();

    Ok(ExerciseData::McqArToEn {
        node_id,
        distractor_node_ids,
    })
}

/// Generate MCQ English to Arabic exercise
pub async fn generate_mcq_en_to_ar(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Similar to ar_to_en, but question shows English, answer is Arabic
    // For now, use same distractor generation logic
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let (chapter, verse, position) = resolve_word_location(&base_ukey, content_repo).await?;

    let verse_key = format!("{}:{}", chapter, verse);
    let words = content_repo.get_words_for_verse(&verse_key).await?;

    let mut distractor_nodes: Vec<Node> = Vec::new();
    for word in words.iter().filter(|w| w.position != position) {
        let distractor_ukey =
            node_id::word_instance(chapter as u8, verse as u16, word.position as u8);
        if let Some(node) = content_repo.get_node_by_ukey(&distractor_ukey).await? {
            distractor_nodes.push(node);
        }
    }

    let mut rng = rand::thread_rng();
    distractor_nodes.shuffle(&mut rng);
    let distractor_node_ids: Vec<i64> =
        distractor_nodes.into_iter().map(|n| n.id).take(3).collect();

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
    node_id: i64,
    _ukey: &str,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Store only node_id, fetch text during generation
    Ok(ExerciseData::Translation { node_id })
}

/// Generate Contextual Translation exercise (with verse context)
pub async fn generate_contextual_translation(
    node_id: i64,
    ukey: &str,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };
    let (chapter, verse, _) = resolve_word_location(&base_ukey, _content_repo).await?;
    let verse_key = format!("{}:{}", chapter, verse);

    Ok(ExerciseData::ContextualTranslation { node_id, verse_key })
}

// ============================================================================
// Cloze/Fill-in Exercises
// ============================================================================

/// Generate Cloze Deletion exercise
pub async fn generate_cloze_deletion(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = if let Some(stripped) = base_ukey.strip_prefix(PREFIX_VERSE) {
        stripped.to_string()
    } else if let Some((chapter, verse)) = node_id::decode_verse(node_id) {
        format!("{}:{}", chapter, verse)
    } else {
        return Err(anyhow::anyhow!("Invalid verse node ID: {}", base_ukey));
    };

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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = if let Some(stripped) = base_ukey.strip_prefix(PREFIX_VERSE) {
        stripped.to_string()
    } else if let Some((chapter, verse)) = node_id::decode_verse(node_id) {
        format!("{}:{}", chapter, verse)
    } else {
        return Err(anyhow::anyhow!("Invalid verse node ID: {}", base_ukey));
    };

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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = if let Some(stripped) = base_ukey.strip_prefix(PREFIX_VERSE) {
        stripped.to_string()
    } else if let Some((chapter, verse)) = node_id::decode_verse(node_id) {
        format!("{}:{}", chapter, verse)
    } else {
        return Err(anyhow::anyhow!("Invalid verse node ID: {}", base_ukey));
    };

    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 4 {
        return Err(anyhow::anyhow!(
            "Verse too short for missing word MCQ (need at least 4 words)"
        ));
    }

    let blank_position = {
        let mut rng = rand::thread_rng();
        rng.gen_range(2..words.len()) as i32
    };

    // Get distractors from the same or nearby verses
    let parts: Vec<&str> = verse_key.split(':').collect();
    let chapter: i32 = parts[0].parse()?;
    let verse: i32 = parts[1].parse()?;

    let mut distractor_nodes: Vec<Node> = Vec::new();
    for w in words.iter().filter(|w| w.position != blank_position) {
        let ukey = node_id::word_instance(chapter as u8, verse as u16, w.position as u8);
        if let Some(node) = content_repo.get_node_by_ukey(&ukey).await? {
            distractor_nodes.push(node);
        }
    }

    let mut distractor_node_ids: Vec<i64> = distractor_nodes.into_iter().map(|n| n.id).collect();
    let mut rng = rand::thread_rng();
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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = if let Some(stripped) = base_ukey.strip_prefix(PREFIX_VERSE) {
        stripped.to_string()
    } else if let Some((chapter, verse)) = node_id::decode_verse(node_id) {
        format!("{}:{}", chapter, verse)
    } else {
        return Err(anyhow::anyhow!("Invalid verse node ID: {}", base_ukey));
    };

    let words = content_repo.get_words_for_verse(&verse_key).await?;

    if words.len() < 3 {
        return Err(anyhow::anyhow!(
            "Verse too short for next word MCQ (need at least 3 words)"
        ));
    }

    // Context position is before the target word
    let context_position = {
        let mut rng = rand::thread_rng();
        rng.gen_range(1..words.len()) as i32
    };

    // Get distractors
    let parts: Vec<&str> = verse_key.split(':').collect();
    let chapter: i32 = parts[0].parse()?;
    let verse: i32 = parts[1].parse()?;

    let mut distractor_nodes: Vec<Node> = Vec::new();
    for w in words.iter().filter(|w| w.position != context_position + 1) {
        let ukey = node_id::word_instance(chapter as u8, verse as u16, w.position as u8);
        if let Some(node) = content_repo.get_node_by_ukey(&ukey).await? {
            distractor_nodes.push(node);
        }
    }

    let mut distractor_node_ids: Vec<i64> = distractor_nodes.into_iter().map(|n| n.id).collect();
    let mut rng = rand::thread_rng();
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
    node_id: i64,
    _ukey: &str,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Store only node_id
    Ok(ExerciseData::FullVerseInput { node_id })
}

/// Generate Sequence Recall exercise
///
/// Creates a simple continuation sequence based on the next verse.
pub async fn generate_sequence_recall(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let (chapter, verse) = if let Some(cv) = node_id::decode_verse(node_id) {
        cv
    } else {
        node_id::parse_verse(&base_ukey).map_err(|e| anyhow::anyhow!(e.to_string()))?
    };

    let verses = content_repo.get_verses_for_chapter(chapter as i32).await?;
    if verses.is_empty() {
        return Err(anyhow::anyhow!("No verses found for chapter {}", chapter));
    }

    let max_verse = verses
        .iter()
        .map(|v| v.verse_number)
        .max()
        .unwrap_or(verse as i32);

    let current = verse as i32;
    let correct_num = if current < max_verse {
        current + 1
    } else if current > 1 {
        current - 1
    } else {
        return Err(anyhow::anyhow!(
            "Cannot determine sequence continuation for {}",
            base_ukey
        ));
    };

    let mut option_nums = vec![correct_num];

    for offset in 1..=3 {
        let candidate = correct_num + offset;
        if candidate <= max_verse {
            option_nums.push(candidate);
        }
    }

    for offset in 1..=3 {
        let candidate = correct_num - offset;
        if candidate >= 1 {
            option_nums.push(candidate);
        }
    }

    option_nums.sort();
    option_nums.dedup();
    option_nums.truncate(4);

    let correct_sequence = vec![node_id::encode_verse(chapter, correct_num as u16)];
    let options = option_nums
        .iter()
        .map(|num| vec![node_id::encode_verse(chapter, *num as u16)])
        .collect();

    Ok(ExerciseData::SequenceRecall {
        node_id,
        correct_sequence,
        options,
    })
}

/// Generate First Word Recall exercise
pub async fn generate_first_word_recall(
    node_id: i64,
    ukey: &str,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let (chapter, verse) = if let Some(cv) = node_id::decode_verse(node_id) {
        cv
    } else {
        node_id::parse_verse(&base_ukey).map_err(|e| anyhow::anyhow!(e.to_string()))?
    };

    let verse_key = format!("{}:{}", chapter, verse);

    Ok(ExerciseData::FirstWordRecall { node_id, verse_key })
}

/// Generate Echo Recall exercise
///
/// Stores only the ayah node IDs. The UI will fetch words on demand.
pub async fn generate_echo_recall(
    verse_node_id: i64,
    _ukey: &str,
    _content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    Ok(ExerciseData::EchoRecall {
        ayah_node_ids: vec![verse_node_id],
    })
}

/// Generate Ayah Chain exercise
pub async fn generate_ayah_chain(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    // Parse chapter number
    let chapter_num: i32 = ukey
        .strip_prefix(PREFIX_CHAPTER)
        .ok_or_else(|| anyhow::anyhow!("Invalid chapter node ID: {}", ukey))?
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
        node_id,
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
        .iter()
        .filter(|v| v.verse_number >= start_verse && v.verse_number <= end_verse)
        .map(|v| v.key.clone())
        .collect();

    if verse_keys.is_empty() {
        return Err(anyhow::anyhow!(
            "No verses found for chapter {} range {}:{}",
            chapter_num,
            start_verse,
            end_verse
        ));
    }

    let ukey = node_id::chapter_range(chapter_num as u8, start_verse as u16, end_verse as u16);
    let node = content_repo
        .get_node_by_ukey(&ukey)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Node not found for ukey: {}", ukey))?;

    Ok(ExerciseData::AyahChain {
        node_id: node.id,
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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = base_ukey
        .strip_prefix(PREFIX_VERSE)
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", base_ukey))?
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
    let mistake_position: i32 = {
        let mut rng = rand::thread_rng();
        if words.len() > 3 {
            (rng.gen_range(1..words.len() - 1) + 1) as i32
        } else {
            2
        }
    };

    // Get correct word node ID
    let correct_word = words
        .iter()
        .find(|w| w.position == mistake_position)
        .ok_or_else(|| anyhow::anyhow!("Word not found at position {}", mistake_position))?;

    let correct_word_ukey = node_id::word_instance(
        chapter_num as u8,
        verse_num as u16,
        correct_word.position as u8,
    );
    let correct_word_node = content_repo
        .get_node_by_ukey(&correct_word_ukey)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Node not found for ukey: {}", correct_word_ukey))?;
    let correct_word_node_id = correct_word_node.id;

    // Get a replacement word from another verse
    let all_verses = content_repo.get_verses_for_chapter(chapter_num).await?;
    let mut candidate_nodes = Vec::new();

    for verse in &all_verses {
        if verse.key != verse_key {
            let other_words = content_repo.get_words_for_verse(&verse.key).await?;
            for word in other_words {
                if word.text_uthmani != correct_word.text_uthmani {
                    let parts: Vec<&str> = verse.key.split(':').collect();
                    let ukey = node_id::word_instance(
                        parts[0].parse().unwrap_or(1),
                        parts[1].parse().unwrap_or(1),
                        word.position as u8,
                    );
                    if let Some(node) = content_repo.get_node_by_ukey(&ukey).await? {
                        candidate_nodes.push(node);
                    }
                }
            }
        }
    }

    if candidate_nodes.is_empty() {
        return Err(anyhow::anyhow!(
            "No suitable replacement words found in chapter {}",
            chapter_num
        ));
    }

    let incorrect_word_node = {
        let mut rng = rand::thread_rng();
        candidate_nodes
            .choose(&mut rng)
            .ok_or_else(|| anyhow::anyhow!("Failed to select random word"))?
    };

    Ok(ExerciseData::FindMistake {
        node_id,
        mistake_position,
        correct_word_node_id,
        incorrect_word_node_id: incorrect_word_node.id,
    })
}

/// Generate Ayah Sequence exercise
pub async fn generate_ayah_sequence(
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    // Can be chapter-level or verse-level (word sequence)
    if base_ukey.starts_with(PREFIX_CHAPTER) {
        // Sequence of verses
        let chapter_num: i32 = base_ukey
            .strip_prefix(PREFIX_CHAPTER)
            .ok_or_else(|| anyhow::anyhow!("Invalid chapter node ID"))?
            .parse()?;

        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // For now, take first 5 verses
        let mut correct_sequence: Vec<i64> = Vec::new();
        for v in verses.iter().take(5) {
            let verse_ukey = node_id::verse_from_key(&v.key);
            let verse_id = if let Some(node) = content_repo.get_node_by_ukey(&verse_ukey).await? {
                node.id
            } else {
                node_id::encode_verse(v.chapter_number as u8, v.verse_number as u16)
            };
            correct_sequence.push(verse_id);
        }

        if correct_sequence.len() < 2 {
            return Err(anyhow::anyhow!("Not enough verses for sequence exercise"));
        }

        Ok(ExerciseData::AyahSequence {
            node_id,
            correct_sequence,
        })
    } else if base_ukey.starts_with(PREFIX_VERSE) {
        // Sequence of words within a verse
        let verse_key = base_ukey
            .strip_prefix(PREFIX_VERSE)
            .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID"))?;

        let words = content_repo.get_words_for_verse(verse_key).await?;

        let parts: Vec<&str> = verse_key.split(':').collect();
        let chapter: i32 = parts[0].parse()?;
        let verse: i32 = parts[1].parse()?;

        let mut correct_sequence: Vec<i64> = Vec::new();
        for w in words {
            let word_ukey = node_id::word_instance(chapter as u8, verse as u16, w.position as u8);
            let word_id = if let Some(node) = content_repo.get_node_by_ukey(&word_ukey).await? {
                node.id
            } else {
                node_id::encode_word_instance(chapter as u8, verse as u16, w.position as u8)
            };
            correct_sequence.push(word_id);
        }

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
            base_ukey
        ))
    }
}

/// Generate Identify Root exercise
pub async fn generate_identify_root(
    word_node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let word = if base_ukey.starts_with(node_id::PREFIX_WORD_INSTANCE) {
        let parts: Vec<&str> = base_ukey.split(':').collect();
        if parts.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_ukey
            ));
        }

        let verse_key = format!("{}:{}", parts[1], parts[2]);
        let position: i32 = parts[3].parse()?;

        let words = content_repo.get_words_for_verse(&verse_key).await?;
        words
            .into_iter()
            .find(|w| w.position == position)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Word not found at position {} in verse {}",
                    position,
                    verse_key
                )
            })?
    } else if base_ukey.starts_with(node_id::PREFIX_WORD) {
        let word_id = node_id::parse_word(&base_ukey).map_err(|e| anyhow::anyhow!(e))?;
        content_repo
            .get_word(word_id as i64)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word not found: {}", base_ukey))?
    } else {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_ukey
        ));
    };

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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = base_ukey
        .strip_prefix(PREFIX_VERSE)
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", base_ukey))?
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
    node_id: i64,
    _ukey: &str,
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
    word_node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let word = if base_ukey.starts_with(node_id::PREFIX_WORD_INSTANCE) {
        let parts: Vec<&str> = base_ukey.split(':').collect();
        if parts.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_ukey
            ));
        }

        let verse_key = format!("{}:{}", parts[1], parts[2]);
        let position: i32 = parts[3].parse()?;

        let words = content_repo.get_words_for_verse(&verse_key).await?;
        words
            .into_iter()
            .find(|w| w.position == position)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Word not found at position {} in verse {}",
                    position,
                    verse_key
                )
            })?
    } else if base_ukey.starts_with(node_id::PREFIX_WORD) {
        let word_id = node_id::parse_word(&base_ukey).map_err(|e| anyhow::anyhow!(e))?;
        content_repo
            .get_word(word_id as i64)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word not found: {}", base_ukey))?
    } else {
        return Err(anyhow::anyhow!(
            "Invalid word node ID format: {}",
            base_ukey
        ));
    };

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
    node_id: i64,
    ukey: &str,
    content_repo: &dyn ContentRepository,
) -> Result<ExerciseData> {
    let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
        kn.base_node_id
    } else {
        ukey.to_string()
    };

    let verse_key = base_ukey
        .strip_prefix(PREFIX_VERSE)
        .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", base_ukey))?
        .to_string();

    let parts: Vec<&str> = verse_key.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid verse key format: {}", verse_key));
    }

    let chapter_num: i32 = parts[0].parse()?;
    let verse_num: i32 = parts[1].parse()?;

    let mut related_verse_ids = Vec::new();
    let mut connection_theme = None;

    let edges = content_repo
        .get_edges_from(node_id)
        .await
        .unwrap_or_default();
    for edge in edges {
        let target = content_repo.get_node(edge.target_id).await?;
        let Some(target) = target else { continue };

        let target_base = KnowledgeNode::parse(&target.ukey)
            .map(|kn| kn.base_node_id)
            .unwrap_or_else(|| target.ukey.clone());

        if target_base.starts_with(PREFIX_VERSE) && target.id != node_id {
            related_verse_ids.push(target.id);
            if connection_theme.is_none() {
                connection_theme = Some(match edge.edge_type {
                    crate::EdgeType::Knowledge => "Knowledge link".to_string(),
                    crate::EdgeType::Dependency => "Dependency link".to_string(),
                });
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    related_verse_ids.retain(|id| seen.insert(*id));

    if related_verse_ids.is_empty() {
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;
        let verse_numbers: std::collections::HashSet<i32> =
            verses.iter().map(|v| v.verse_number).collect();

        let mut candidates = Vec::new();
        for candidate in [verse_num + 1, verse_num + 2, verse_num - 1, verse_num - 2] {
            if candidate >= 1 && verse_numbers.contains(&candidate) {
                candidates.push(candidate);
            }
        }

        candidates.sort();
        candidates.dedup();

        for num in candidates {
            let key = format!("{}:{}", chapter_num, num);
            let verse_ukey = node_id::verse_from_key(&key);
            let verse_id = if let Some(node) = content_repo.get_node_by_ukey(&verse_ukey).await? {
                node.id
            } else {
                node_id::encode_verse(chapter_num as u8, num as u16)
            };
            related_verse_ids.push(verse_id);
        }

        connection_theme = Some("Adjacent verses".to_string());
    }

    if related_verse_ids.is_empty() {
        return Err(anyhow::anyhow!(
            "Not enough related verses for connection exercise"
        ));
    }

    let correct_id = related_verse_ids[0];
    let mut option_ids = vec![correct_id];

    let mut distractors = Vec::new();
    for chapter in [1, 2, 3, 112, 113, 114] {
        if chapter == chapter_num {
            continue;
        }
        let verses = content_repo.get_verses_for_chapter(chapter).await?;
        for verse in verses.iter().take(3) {
            let verse_id =
                node_id::encode_verse(verse.chapter_number as u8, verse.verse_number as u16);
            if verse_id != correct_id {
                distractors.push(verse_id);
                if distractors.len() >= 3 {
                    break;
                }
            }
        }
        if distractors.len() >= 3 {
            break;
        }
    }

    if distractors.len() < 3 {
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;
        for verse in verses.iter().rev().take(5) {
            let verse_id =
                node_id::encode_verse(verse.chapter_number as u8, verse.verse_number as u16);
            if verse_id != correct_id && !distractors.contains(&verse_id) {
                distractors.push(verse_id);
                if distractors.len() >= 3 {
                    break;
                }
            }
        }
    }

    for id in distractors.into_iter().take(3) {
        option_ids.push(id);
    }

    Ok(ExerciseData::CrossVerseConnection {
        node_id,
        related_verse_ids: option_ids,
        connection_theme: connection_theme.unwrap_or_else(|| "Graph connection".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_memorization() {
        // Synchronous test - no async needed
        let node_id = 1;
        let exercise = ExerciseData::Memorization { node_id };
        assert_eq!(exercise.node_id(), node_id);
    }

    #[test]
    fn test_generate_translation() {
        let node_id = 1;
        let exercise = ExerciseData::Translation { node_id };
        assert_eq!(exercise.node_id(), node_id);
    }

    #[test]
    fn test_generate_full_verse_input() {
        let node_id = 1;
        let exercise = ExerciseData::FullVerseInput { node_id };
        assert_eq!(exercise.node_id(), node_id);
    }
}
