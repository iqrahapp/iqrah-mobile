//! Initial placement service implementation.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use anyhow::Result;
use chrono::{Duration, Utc};
use rand::prelude::*;
use rand::rngs::StdRng;
use tracing::{debug, info, instrument};

use crate::domain::MemoryState;
use crate::ports::{ContentRepository, UserRepository};

use super::config::InitialPlacementConfig;
use super::summary::{InitialPlacementSummary, SurahPlacementResult};
use super::types::IntakeAnswers;

/// Service for applying initial knowledge placement based on intake answers.
///
/// Maps user self-reports to FSRS memory states and KG node energies.
pub struct InitialPlacementService<R, C>
where
    R: UserRepository + ?Sized,
    C: ContentRepository + ?Sized,
{
    user_repo: Arc<R>,
    content_repo: Arc<C>,
    config: InitialPlacementConfig,
}

impl<R, C> InitialPlacementService<R, C>
where
    R: UserRepository + ?Sized,
    C: ContentRepository + ?Sized,
{
    /// Create a new initial placement service.
    pub fn new(user_repo: Arc<R>, content_repo: Arc<C>) -> Self {
        Self {
            user_repo,
            content_repo,
            config: InitialPlacementConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(
        user_repo: Arc<R>,
        content_repo: Arc<C>,
        config: InitialPlacementConfig,
    ) -> Self {
        Self {
            user_repo,
            content_repo,
            config,
        }
    }

    /// Apply initial placement based on intake answers.
    ///
    /// This initializes the user's memory states for verses and vocabulary
    /// based on their self-reported knowledge levels.
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `answers` - Intake questionnaire answers
    /// * `seed` - Global seed for deterministic sampling
    ///
    /// # Returns
    /// Summary of initialized nodes
    #[instrument(skip(self, answers), fields(user_id = user_id, seed = seed))]
    pub async fn apply_intake(
        &self,
        user_id: &str,
        answers: IntakeAnswers,
        seed: u64,
    ) -> Result<InitialPlacementSummary> {
        info!("Applying initial placement");

        let mut summary = InitialPlacementSummary::new();
        let reading_fluency = answers.effective_reading_fluency();
        summary.reading_fluency_used = reading_fluency;

        // Collect all states to batch-save
        let mut all_states: Vec<MemoryState> = Vec::new();

        // Process each surah report
        for report in &answers.surah_reports {
            if report.memorization_pct <= 0.0 && report.understanding_pct <= 0.0 {
                continue; // Skip surahs with no reported knowledge
            }

            let surah_result = self
                .process_surah_report(
                    user_id,
                    report.chapter_id,
                    report.memorization_pct,
                    report.understanding_pct,
                    reading_fluency,
                    seed,
                    &mut all_states,
                )
                .await?;

            summary.add_surah_result(surah_result);
        }

        // Apply global vocab modifier if provided
        if let Some(global_vocab_pct) = answers.global_vocab_estimate_pct {
            if global_vocab_pct > 0.0 {
                // This would require sampling from all vocab in database
                // For v1, we only initialize vocab for reported surahs
                debug!("Global vocab estimate: {:.1}%", global_vocab_pct * 100.0);
            }
        }

        summary.global_modifiers_applied = reading_fluency > 0.0;

        // Batch save all states
        if !all_states.is_empty() {
            info!("Saving {} memory states", all_states.len());
            self.user_repo.save_memory_states_batch(&all_states).await?;
        }

        info!(
            "Initial placement complete: {} verses, {} vocab nodes",
            summary.verses_initialized, summary.vocab_nodes_initialized
        );

        Ok(summary)
    }

    /// Process a single surah report.
    async fn process_surah_report(
        &self,
        user_id: &str,
        chapter_id: i32,
        memorization_pct: f64,
        understanding_pct: f64,
        reading_fluency: f64,
        seed: u64,
        states: &mut Vec<MemoryState>,
    ) -> Result<SurahPlacementResult> {
        debug!(
            "Processing surah {}: mem={:.1}%, understand={:.1}%",
            chapter_id,
            memorization_pct * 100.0,
            understanding_pct * 100.0
        );

        // Get all verses for this chapter
        let verses = self.content_repo.get_verses_for_chapter(chapter_id).await?;
        let verses_total = verses.len();

        if verses_total == 0 {
            return Ok(SurahPlacementResult {
                chapter_id,
                verses_known: 0,
                verses_partial: 0,
                verses_total: 0,
                vocab_initialized: 0,
            });
        }

        // Create deterministic RNG for this (user, chapter, seed) combination
        let mut rng = make_rng_for(user_id, chapter_id, seed);

        // Determine which verses to mark as known vs partial
        let n = verses_total;
        let n_known = ((memorization_pct * n as f64) as usize).min(n);
        let n_partial = if memorization_pct >= self.config.partial_threshold {
            (n - n_known).min(((memorization_pct * 0.5) * n as f64) as usize)
        } else {
            0
        };

        // Create shuffled indices
        let mut indices: Vec<usize> = (0..n).collect();
        indices.shuffle(&mut rng);

        let now = Utc::now();
        let mut verses_known = 0;
        let mut verses_partial = 0;
        let mut vocab_initialized = 0;

        // Process verses
        for (i, &verse_idx) in indices.iter().enumerate() {
            let verse = &verses[verse_idx];

            // Get node ID for this verse
            let node_id = if let Some(node) = self.content_repo.get_node_by_ukey(&verse.key).await?
            {
                node.id
            } else {
                debug!("No node found for verse {}", verse.key);
                continue;
            };

            if i < n_known {
                // Fully known verse
                let state = self.create_verse_state(
                    user_id,
                    node_id,
                    memorization_pct,
                    reading_fluency,
                    now,
                    true,
                );
                states.push(state);
                verses_known += 1;

                // Initialize vocab for known verses
                if understanding_pct > 0.0 {
                    let vocab_count = self
                        .initialize_vocab_for_verse(
                            user_id,
                            &verse.key,
                            understanding_pct,
                            reading_fluency,
                            seed,
                            states,
                        )
                        .await?;
                    vocab_initialized += vocab_count;
                }
            } else if i < n_known + n_partial {
                // Partially known verse
                let state = self.create_verse_state(
                    user_id,
                    node_id,
                    memorization_pct * 0.5, // Reduced stability
                    reading_fluency,
                    now,
                    false,
                );
                states.push(state);
                verses_partial += 1;
            }
            // Remaining verses: not initialized (treated as new)
        }

        Ok(SurahPlacementResult {
            chapter_id,
            verses_known,
            verses_partial,
            verses_total,
            vocab_initialized,
        })
    }

    /// Create a memory state for a verse.
    fn create_verse_state(
        &self,
        user_id: &str,
        node_id: i64,
        memorization_pct: f64,
        reading_fluency: f64,
        now: chrono::DateTime<Utc>,
        fully_known: bool,
    ) -> MemoryState {
        let stability = self.config.verse_stability(memorization_pct);
        let difficulty = self.config.verse_difficulty(reading_fluency);
        let review_count = self.config.verse_review_count(memorization_pct);
        let energy = if fully_known {
            self.config.verse_known_energy
        } else {
            self.config.verse_partial_energy
        };

        // Calculate due date based on stability
        let due_at = now + Duration::days(stability as i64);

        // Last reviewed is backdated to make the item "due" in the right timeframe
        let last_reviewed = now - Duration::days((stability * 0.5) as i64);

        MemoryState {
            user_id: user_id.to_string(),
            node_id,
            stability,
            difficulty,
            energy,
            last_reviewed,
            due_at,
            review_count,
        }
    }

    /// Initialize vocabulary nodes for a verse based on understanding percentage.
    async fn initialize_vocab_for_verse(
        &self,
        user_id: &str,
        verse_key: &str,
        understanding_pct: f64,
        reading_fluency: f64,
        seed: u64,
        states: &mut Vec<MemoryState>,
    ) -> Result<usize> {
        let words = self.content_repo.get_words_for_verse(verse_key).await?;
        if words.is_empty() {
            return Ok(0);
        }

        // Create RNG for vocab selection
        let mut rng = make_rng_for_vocab(user_id, verse_key, seed);

        let n = words.len();
        let n_known = ((understanding_pct * n as f64) as usize).min(n);

        // Shuffle and select
        let mut indices: Vec<usize> = (0..n).collect();
        indices.shuffle(&mut rng);

        let now = Utc::now();
        let mut count = 0;

        for i in 0..n_known {
            let word = &words[indices[i]];
            let state =
                self.create_vocab_state(user_id, word.id, understanding_pct, reading_fluency, now);
            states.push(state);
            count += 1;
        }

        Ok(count)
    }

    /// Create a memory state for a vocabulary node.
    fn create_vocab_state(
        &self,
        user_id: &str,
        node_id: i64,
        understanding_pct: f64,
        reading_fluency: f64,
        now: chrono::DateTime<Utc>,
    ) -> MemoryState {
        let stability = self.config.vocab_stability(understanding_pct);
        let difficulty = self.config.vocab_base_difficulty
            * (1.0 - self.config.fluency_difficulty_reduction * reading_fluency);
        let review_count = self.config.vocab_review_count(understanding_pct);
        let energy = self.config.vocab_known_energy;

        let due_at = now + Duration::days(stability as i64);
        let last_reviewed = now - Duration::days((stability * 0.5) as i64);

        MemoryState {
            user_id: user_id.to_string(),
            node_id,
            stability,
            difficulty,
            energy,
            last_reviewed,
            due_at,
            review_count,
        }
    }
}

/// Create a deterministic RNG for (user_id, chapter_id, seed).
///
/// Same inputs always produce the same RNG state.
pub fn make_rng_for(user_id: &str, chapter_id: i32, seed: u64) -> StdRng {
    let mut hasher = DefaultHasher::new();
    user_id.hash(&mut hasher);
    chapter_id.hash(&mut hasher);
    seed.hash(&mut hasher);
    StdRng::seed_from_u64(hasher.finish())
}

/// Create a deterministic RNG for vocab selection within a verse.
fn make_rng_for_vocab(user_id: &str, verse_key: &str, seed: u64) -> StdRng {
    let mut hasher = DefaultHasher::new();
    user_id.hash(&mut hasher);
    verse_key.hash(&mut hasher);
    "vocab".hash(&mut hasher); // Differentiate from verse RNG
    seed.hash(&mut hasher);
    StdRng::seed_from_u64(hasher.finish())
}
