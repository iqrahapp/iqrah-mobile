//! Main simulation orchestrator.
//!
//! The Simulator runs virtual students through the real Iqrah scheduling pipeline.
//! ISS **orchestrates** the simulation; `iqrah-core` **decides** what to schedule.

use crate::brain::{PriorKnowledgeConfig, StudentBrain};
use crate::config::{Scenario, SimulationConfig};
use crate::in_memory_repo::InMemoryUserRepository;
use crate::metrics::SimulationMetrics;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use iqrah_core::domain::MemoryState;
use iqrah_core::ports::{ContentRepository, UserRepository};
use iqrah_core::scheduler_v2::bandit::BanditOptimizer;
use iqrah_core::scheduler_v2::session_generator::{generate_session, SessionMode};
use iqrah_core::scheduler_v2::{CandidateNode, UserProfile};
use iqrah_core::services::LearningService;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Average time per exercise item in seconds.
const SECONDS_PER_ITEM: f64 = 30.0;

/// Convert a verse key (e.g., "1:1") to a node ID.
/// Uses a simple encoding: chapter * 1000 + verse
fn verse_key_to_node_id(key: &str) -> i64 {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() == 2 {
        let chapter: i64 = parts[0].parse().unwrap_or(0);
        let verse: i64 = parts[1].parse().unwrap_or(0);
        chapter * 1000 + verse
    } else {
        0
    }
}

/// Main simulator orchestrator.
pub struct Simulator {
    /// Real content repository (read-only access to content.db)
    content_repo: Arc<dyn ContentRepository>,

    /// Simulation configuration
    config: SimulationConfig,
}

impl Simulator {
    /// Create a new simulator with content repository and configuration.
    pub fn new(content_repo: Arc<dyn ContentRepository>, config: SimulationConfig) -> Self {
        Self {
            content_repo,
            config,
        }
    }

    /// Run simulation for a single student.
    ///
    /// # Arguments
    /// * `scenario` - The scenario to simulate
    /// * `student_index` - Index of the student (for seed derivation)
    ///
    /// # Returns
    /// Aggregated metrics for the student's simulation run
    #[instrument(skip(self, scenario), fields(scenario = %scenario.name, student = student_index))]
    pub async fn simulate_student(
        &self,
        scenario: &Scenario,
        student_index: usize,
    ) -> Result<SimulationMetrics> {
        info!("Starting simulation");

        // 1. Create student-specific RNG seed
        let student_seed = self.config.student_seed(student_index);
        let scheduler_seed = self.config.scheduler_seed();

        // 2. Create student brain
        let mut brain = StudentBrain::new(scenario.student_params.clone(), student_seed);

        // 3. Create in-memory user repository for this student
        let user_repo = Arc::new(InMemoryUserRepository::new());
        let user_id = format!("sim_student_{}", student_index);

        // 4. Initialize prior knowledge
        self.initialize_prior_knowledge(&mut brain, &user_id, &user_repo)
            .await
            .context("Failed to initialize prior knowledge")?;

        // 5. Create learning service using real iqrah-core
        let learning_service =
            LearningService::new(Arc::clone(&self.content_repo), Arc::clone(&user_repo) as _);

        // 6. Create scheduler RNG (separate from student RNG)
        let mut scheduler_rng = StdRng::seed_from_u64(scheduler_seed);

        // 7. Get goal items
        let goal_items = self.get_goal_items(&scenario.goal_id).await?;
        if goal_items.is_empty() {
            warn!("No goal items found for goal_id: {}", scenario.goal_id);
            return Ok(SimulationMetrics::default());
        }
        info!("Goal has {} items", goal_items.len());

        // 8. Track metrics
        let mut total_minutes = 0.0;
        let mut days_completed = 0u32;
        let mut introduction_order: HashMap<i64, usize> = HashMap::new();
        let mut intro_index = 1usize;

        // 9. Main simulation loop
        for day in 0..scenario.target_days {
            if brain.has_given_up() {
                info!("Student gave up at day {}", day);
                break;
            }

            brain.start_day();

            // Skip day?
            if brain.should_skip_day() {
                debug!("Day {} skipped", day);
                days_completed += 1;
                continue;
            }

            // Simulate this day
            let day_minutes = self
                .simulate_day(
                    &user_id,
                    scenario,
                    &mut brain,
                    &user_repo,
                    &learning_service,
                    &goal_items,
                    &mut scheduler_rng,
                    day,
                    &mut introduction_order,
                    &mut intro_index,
                )
                .await
                .context("Failed to simulate day")?;

            total_minutes += day_minutes;
            days_completed += 1;

            // Check daily minute budget
            if day_minutes >= scenario.daily_minutes {
                debug!("Day {} completed full budget", day);
            }
        }

        // 10. Compute final metrics
        let stabilities = user_repo.get_stabilities_for_user(&user_id);

        // Build plan priorities (for now, use goal order as priority)
        let plan_priorities: HashMap<i64, usize> = goal_items
            .iter()
            .enumerate()
            .map(|(i, &nid)| (nid, i + 1))
            .collect();

        let metrics = SimulationMetrics::compute(
            &stabilities,
            &goal_items,
            scenario.target_days as f64,
            total_minutes,
            days_completed,
            brain.has_given_up(),
            &plan_priorities,
            &introduction_order,
        );

        info!(
            "Simulation complete: {} days, {:.1} min, {}/{} mastered",
            metrics.total_days,
            metrics.total_minutes,
            metrics.items_mastered,
            metrics.goal_item_count
        );

        Ok(metrics)
    }

    /// Simulate a single day of learning.
    #[allow(clippy::too_many_arguments)]
    async fn simulate_day(
        &self,
        user_id: &str,
        scenario: &Scenario,
        brain: &mut StudentBrain,
        user_repo: &Arc<InMemoryUserRepository>,
        learning_service: &LearningService,
        goal_items: &[i64],
        scheduler_rng: &mut StdRng,
        day: u32,
        introduction_order: &mut HashMap<i64, usize>,
        intro_index: &mut usize,
    ) -> Result<f64> {
        let now = Utc::now() + Duration::days(day as i64);
        let now_ts = now.timestamp_millis();

        // 1. Get candidates for scheduling
        let candidates = self
            .get_candidates(user_id, goal_items, user_repo, now_ts)
            .await?;

        if candidates.is_empty() {
            return Ok(0.0);
        }

        // 2. Get parent energies (for prerequisite gate)
        let all_parent_ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();
        let parent_energies = user_repo
            .get_parent_energies(user_id, &all_parent_ids)
            .await?;

        // 3. Get parent map (prerequisites)
        let parent_map = self
            .content_repo
            .get_prerequisite_parents(&all_parent_ids)
            .await?;

        // 4. Get user profile (via bandit if enabled)
        let profile = if scenario.enable_bandit {
            self.select_profile_via_bandit(user_id, &scenario.goal_id, user_repo, scheduler_rng)
                .await?
        } else {
            UserProfile::balanced()
        };

        // 5. Generate session using REAL scheduler
        let session_items = generate_session(
            candidates,
            parent_map,
            parent_energies,
            &profile,
            scenario.session_size,
            now_ts,
            SessionMode::MixedLearning,
        );

        // 6. Process each item in the session
        let mut minutes_spent = 0.0;
        let mut last_difficulty = 1.0f64;

        for node_id in session_items {
            // Track first introduction
            introduction_order.entry(node_id).or_insert_with(|| {
                let idx = *intro_index;
                *intro_index += 1;
                idx
            });

            // Get current memory state
            let state = user_repo
                .get_memory_state(user_id, node_id)
                .await?
                .unwrap_or_else(|| MemoryState::new_for_node(user_id.to_string(), node_id));

            // Compute elapsed days since last review
            let elapsed_days = if state.review_count > 0 {
                (now.timestamp_millis() - state.last_reviewed.timestamp_millis()) as f64
                    / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                0.0 // First time seeing this item
            };

            // Student attempts recall
            let recall_result =
                brain.attempt_recall(state.stability, state.difficulty, elapsed_days);
            let grade = brain.determine_grade(recall_result);

            // Process review via REAL learning service
            learning_service
                .process_review(user_id, node_id, grade)
                .await
                .context("Failed to process review")?;

            // Update time tracking
            minutes_spent += SECONDS_PER_ITEM / 60.0;
            last_difficulty = state.difficulty;

            // Check early quit
            if brain.should_quit_early(minutes_spent, last_difficulty) {
                debug!("Quitting early after {:.1} minutes", minutes_spent);
                break;
            }

            // Check daily budget
            if minutes_spent >= scenario.daily_minutes {
                break;
            }

            // Check if student gave up
            if brain.has_given_up() {
                break;
            }
        }

        Ok(minutes_spent)
    }

    /// Initialize prior knowledge for a student.
    async fn initialize_prior_knowledge(
        &self,
        brain: &mut StudentBrain,
        user_id: &str,
        user_repo: &Arc<InMemoryUserRepository>,
    ) -> Result<()> {
        let config = PriorKnowledgeConfig::default();
        let now = Utc::now();

        // Initialize known surahs
        for surah_id in &brain.params.known_surah_ids {
            let verses = self
                .content_repo
                .get_verses_for_chapter(*surah_id)
                .await
                .unwrap_or_default();

            for verse in verses {
                // Derive node_id from verse key (e.g. "1:1" -> node ID)
                // For now, use a simple hash of the verse key as node_id
                // TODO: Use proper node ID encoding from content schema
                let node_id = verse_key_to_node_id(&verse.key);
                if node_id == 0 {
                    continue;
                }

                let state = MemoryState {
                    user_id: user_id.to_string(),
                    node_id,
                    stability: config.stability,
                    difficulty: config.difficulty,
                    energy: config.energy,
                    last_reviewed: now,
                    due_at: now + Duration::days(30),
                    review_count: config.review_count,
                };

                user_repo.initialize_state(state);
            }
        }

        // TODO: Implement vocab_known_pct sampling
        // This would require access to vocabulary nodes and sampling logic

        Ok(())
    }

    /// Get goal items (node IDs) for a goal.
    async fn get_goal_items(&self, goal_id: &str) -> Result<Vec<i64>> {
        // Try to get from real content repository
        let items = self.content_repo.get_nodes_for_goal(goal_id).await?;

        if !items.is_empty() {
            return Ok(items);
        }

        // Fallback: parse goal_id and generate items
        // Format: "surah:N" or "juz:N"
        if let Some(surah_num) = goal_id.strip_prefix("surah:") {
            if let Ok(num) = surah_num.parse::<i32>() {
                let verses = self.content_repo.get_verses_for_chapter(num).await?;
                return Ok(verses
                    .into_iter()
                    .map(|v| verse_key_to_node_id(&v.key))
                    .collect());
            }
        }

        // Return empty if we can't resolve
        warn!("Could not resolve goal_id: {}", goal_id);
        Ok(Vec::new())
    }

    /// Get candidate nodes for scheduling.
    async fn get_candidates(
        &self,
        user_id: &str,
        goal_items: &[i64],
        user_repo: &Arc<InMemoryUserRepository>,
        now_ts: i64,
    ) -> Result<Vec<CandidateNode>> {
        let mut candidates = Vec::new();

        // Get memory basics for all goal items
        let memory_basics = user_repo.get_memory_basics(user_id, goal_items).await?;

        for &node_id in goal_items {
            let (energy, next_due_ts) = memory_basics
                .get(&node_id)
                .map(|m| (m.energy, m.next_due_ts))
                .unwrap_or((0.0, 0)); // New items have 0 energy and are always due

            // Only include items that are due or new
            if next_due_ts > now_ts && energy > 0.0 {
                continue; // Not due yet and not new
            }

            // Try to get metadata from content repo
            // TODO: Use get_scheduler_candidates for full metadata
            let candidate = CandidateNode {
                id: node_id,
                foundational_score: 0.5, // Default
                influence_score: 0.5,    // Default
                difficulty_score: 0.3,   // Default
                energy,
                next_due_ts,
                quran_order: node_id, // Use node_id as tie-breaker
            };

            candidates.push(candidate);
        }

        Ok(candidates)
    }

    /// Select user profile via bandit (Thompson Sampling).
    async fn select_profile_via_bandit(
        &self,
        user_id: &str,
        goal_group: &str,
        user_repo: &Arc<InMemoryUserRepository>,
        rng: &mut StdRng,
    ) -> Result<UserProfile> {
        // Get current bandit state
        let arms = user_repo.get_bandit_arms(user_id, goal_group).await?;

        if arms.is_empty() {
            // Initialize arms if not present
            let initial_arms = BanditOptimizer::<StdRng>::initialize_arms();
            for arm in &initial_arms {
                user_repo
                    .update_bandit_arm(
                        user_id,
                        goal_group,
                        arm.profile_name.as_str(),
                        arm.successes,
                        arm.failures,
                    )
                    .await?;
            }
            return Ok(UserProfile::balanced());
        }

        // Use Thompson Sampling to select profile
        let mut bandit = BanditOptimizer::new(rng.clone());
        let chosen = bandit.choose_arm(&arms);

        // Blend with safe default
        Ok(iqrah_core::scheduler_v2::bandit::blend_profile(chosen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verse_key_to_node_id() {
        // Test basic encoding: chapter * 1000 + verse
        assert_eq!(verse_key_to_node_id("1:1"), 1001);
        assert_eq!(verse_key_to_node_id("1:7"), 1007);
        assert_eq!(verse_key_to_node_id("2:255"), 2255);
        assert_eq!(verse_key_to_node_id("114:6"), 114006);
    }

    #[test]
    fn test_verse_key_to_node_id_invalid() {
        // Invalid keys should return 0
        assert_eq!(verse_key_to_node_id("invalid"), 0);
        assert_eq!(verse_key_to_node_id(""), 0);
        assert_eq!(verse_key_to_node_id("1:2:3"), 0);
    }

    // Integration tests that require a real ContentRepository should be
    // placed in tests/integration_tests.rs and run against a test database.
    // For now, we test the individual components in their respective modules.
}
