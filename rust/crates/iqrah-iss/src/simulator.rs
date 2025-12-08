//! Main simulation orchestrator.
//!
//! The Simulator runs virtual students through the real Iqrah scheduling pipeline.
//! ISS **orchestrates** the simulation; `iqrah-core` **decides** what to schedule.

use crate::baselines::{FixedSrsBaseline, PageOrderBaseline, RandomBaseline};
use crate::debug_stats::{StudentDebugAccumulator, StudentDebugSummary};
use crate::{
    InMemoryUserRepository, Scenario, SchedulerVariant, SessionGenerator, SimulationConfig,
    SimulationMetrics, StudentBrain,
};

use crate::config::{compute_almost_due_window, compute_min_new_for_plan};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use iqrah_core::domain::{MemoryState, ReviewGrade};
use iqrah_core::initial_placement::{
    ArabicLevel, InitialPlacementService, IntakeAnswers, SurahSelfReport,
};
use iqrah_core::ports::{ContentRepository, UserRepository};
use iqrah_core::scheduler_v2::bandit::BanditOptimizer;
use iqrah_core::scheduler_v2::session_generator::{generate_session, SessionMode};
use iqrah_core::scheduler_v2::{CandidateNode, SessionMixConfig, UserProfile};
use iqrah_core::services::LearningService;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, instrument, warn};

/// Estimate exercise time based on item properties.
///
/// Base times:
/// - New items (review_count == 0): ~50 seconds
/// - Review items: ~20 seconds
///
/// Modifiers:
/// - Difficulty: higher difficulty = more time
/// - Reading fluency: lower fluency = more time
fn estimate_exercise_time(
    stability: f64,
    difficulty: f64,
    is_new: bool,
    _reading_fluency: f64, // Placeholder for future use
) -> f64 {
    let base_secs = if is_new { 50.0 } else { 20.0 };

    // Difficulty modifier: D=1 -> 1.0x, D=10 -> 1.9x
    let diff_mult = 1.0 + (difficulty - 1.0).max(0.0) * 0.1;

    // Stability modifier: very low stability = uncertain, needs more time
    let stab_mult = if stability < 1.0 {
        1.2 // Low stability = review needed soon, takes more effort
    } else {
        1.0
    };

    (base_secs * diff_mult * stab_mult).clamp(10.0, 120.0)
}

/// Convert a verse key (e.g., "1:1") to a node ID.
/// Uses a simple encoding: chapter * 1000 + verse
fn verse_key_to_node_id(key: &str) -> i64 {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() != 2 {
        return 0;
    }
    let chapter: i64 = parts[0].parse().unwrap_or(0);
    let verse: i64 = parts[1].parse().unwrap_or(0);
    chapter * 1000 + verse
}

use crate::brain::StudentParams;

/// Convert StudentParams to IntakeAnswers for initializing prior knowledge.
///
/// Maps known_surah_ids to SurahSelfReports with 100% memorization (fully known)
/// and uses vocab_known_pct as understanding percentage.
fn create_intake_from_params(params: &StudentParams) -> IntakeAnswers {
    IntakeAnswers {
        surah_reports: params
            .known_surah_ids
            .iter()
            .map(|&id| SurahSelfReport {
                chapter_id: id,
                memorization_pct: 1.0, // Known surahs are fully memorized
                understanding_pct: params.vocab_known_pct.max(0.3), // At least 30% understanding
            })
            .collect(),
        global_vocab_estimate_pct: Some(params.vocab_known_pct),
        arabic_level: Some(ArabicLevel::BasicReading), // Default assumption for simulation
        ..Default::default()
    }
}

/// Baseline session state for non-Iqrah schedulers.
enum BaselineState {
    /// No baseline - use Iqrah scheduler
    None,
    /// Page-order baseline
    PageOrder(PageOrderBaseline),
    /// Fixed-SRS baseline
    FixedSrs(FixedSrsBaseline),
    /// Random baseline
    Random(RandomBaseline),
}

impl BaselineState {
    /// Create baseline state from scheduler variant.
    fn new(variant: SchedulerVariant, seed: u64) -> Self {
        match variant {
            SchedulerVariant::IqrahDefault => Self::None,
            SchedulerVariant::BaselinePageOrder => Self::PageOrder(PageOrderBaseline::new()),
            SchedulerVariant::BaselineFixedSrs => Self::FixedSrs(FixedSrsBaseline::new()),
            SchedulerVariant::BaselineRandom => Self::Random(RandomBaseline::new(seed)),
        }
    }

    /// Generate session using baseline logic.
    fn generate_session(
        &mut self,
        goal_items: &[i64],
        memory_states: &HashMap<i64, MemoryState>,
        session_size: usize,
        current_day: u32,
    ) -> Option<Vec<i64>> {
        let now = chrono::Utc::now();
        match self {
            Self::None => None, // Use Iqrah scheduler
            Self::PageOrder(b) => {
                Some(b.generate_session(goal_items, memory_states, session_size, current_day, now))
            }
            Self::FixedSrs(b) => {
                Some(b.generate_session(goal_items, memory_states, session_size, current_day, now))
            }
            Self::Random(b) => {
                Some(b.generate_session(goal_items, memory_states, session_size, current_day, now))
            }
        }
    }

    /// Record a review result for FixedSRS baseline.
    fn record_review(&mut self, node_id: i64, current_day: u32, success: bool) {
        if let Self::FixedSrs(b) = self {
            b.record_review(node_id, current_day, success);
        }
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
    /// Run simulation for a single student (internal implementation).
    #[instrument(skip(self, scenario), fields(scenario = %scenario.name, student = student_index))]
    async fn simulate_student_internal(
        &self,
        scenario: &Scenario,
        student_index: usize,
    ) -> Result<(
        SimulationMetrics,
        Arc<InMemoryUserRepository>,
        Option<StudentDebugSummary>,
    )> {
        debug!("Starting simulation");

        // 1. Create student-specific RNG seed
        let student_seed = self.config.student_seed(student_index);
        let scheduler_seed = self.config.scheduler_seed();
        // Use deterministic start time for reproducible results (parallel-safe)
        // Epoch: 2024-01-01 00:00:00 UTC
        let start_time = chrono::DateTime::from_timestamp(1704067200, 0).unwrap_or_else(Utc::now);

        // 2. Create student brain with params (supports heterogeneous populations)
        let student_params = scenario.get_student_params(student_index, self.config.base_seed);
        let mut brain = StudentBrain::new(student_params, student_seed);

        // 3. Create in-memory user repository for this student
        let user_repo = Arc::new(InMemoryUserRepository::new());
        let user_id = format!("sim_student_{}", student_index);

        // 4. Initialize prior knowledge using IPS
        self.initialize_prior_knowledge(&brain, &user_id, &user_repo, student_seed)
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
            debug!("No goal items found for goal_id: {}", scenario.goal_id);
            return Ok((SimulationMetrics::default(), user_repo, None));
        }
        debug!("Goal has {} items", goal_items.len());

        // 7b. Compute dynamic almost_due_window for this plan
        // For large plans, widening the window keeps recently-introduced items in the candidate pool
        let dynamic_window = compute_almost_due_window(
            goal_items.len(),
            scenario.target_days,
            scenario.session_size,
        );
        debug!(
            "Dynamic almost_due_window: {} days (plan: {} items, {} days, session: {})",
            dynamic_window,
            goal_items.len(),
            scenario.target_days,
            scenario.session_size
        );

        // 8. Track metrics
        let mut total_minutes = 0.0;
        let mut days_completed = 0u32;
        let mut introduction_order: HashMap<i64, usize> = HashMap::new();
        let mut intro_index = 1usize;

        // 8b. Create baseline state based on scheduler variant
        let mut baseline_state = BaselineState::new(scenario.scheduler, scheduler_seed);

        let mut debug_accumulator = if self.config.debug_stats {
            Some(StudentDebugAccumulator::new(student_index))
        } else {
            None
        };

        // 9. Main simulation loop
        for day in 0..scenario.target_days {
            if brain.has_given_up() {
                debug!("Student gave up at day {}", day);
                if let Some(acc) = debug_accumulator.as_mut() {
                    acc.give_up = true;
                    acc.day_of_give_up = Some(day);
                }
                break;
            }

            brain.start_day();

            // Skip day?
            if brain.should_skip_day() {
                debug!("Day {} skipped", day);
                days_completed += 1;
                if let Some(acc) = debug_accumulator.as_mut() {
                    acc.skipped_days += 1;
                }
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
                    &mut baseline_state,
                    start_time,
                    &mut debug_accumulator,
                    dynamic_window,
                )
                .await
                .context("Failed to simulate day")?;

            total_minutes += day_minutes;
            days_completed += 1;

            if let Some(acc) = debug_accumulator.as_mut() {
                acc.record_day_end(
                    false, // skipped handled above
                    brain.frustration,
                    brain.weighted_failure_score,
                );
            }

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

        debug!(
            "Simulation complete: {} days, {:.1} min, {}/{} mastered",
            metrics.total_days,
            metrics.total_minutes,
            metrics.items_mastered,
            metrics.goal_item_count
        );

        let debug_summary = debug_accumulator.map(|a| a.finish());
        Ok((metrics, user_repo, debug_summary))
    }

    /// Run simulation for a single student.
    pub async fn simulate_student(
        &self,
        scenario: &Scenario,
        student_index: usize,
    ) -> Result<(SimulationMetrics, Option<StudentDebugSummary>)> {
        let (metrics, _, summary) = self
            .simulate_student_internal(scenario, student_index)
            .await?;
        Ok((metrics, summary))
    }

    /// Run simulation and return user repo for debugging.
    pub async fn simulate_student_debug(
        &self,
        scenario: &Scenario,
        student_index: usize,
    ) -> Result<(
        SimulationMetrics,
        Arc<InMemoryUserRepository>,
        Option<StudentDebugSummary>,
    )> {
        self.simulate_student_internal(scenario, student_index)
            .await
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
        baseline_state: &mut BaselineState,
        start_time: chrono::DateTime<Utc>,
        accumulator: &mut Option<StudentDebugAccumulator>,
        almost_due_window_days: u32,
    ) -> Result<f64> {
        let now = start_time + Duration::days(day as i64);
        let now_ts = now.timestamp_millis();

        // 1. Generate session - branch based on scheduler variant
        let session_items = if let Some(baseline_session) = {
            // Get memory states for baseline session generation
            let memory_states = user_repo.get_all_states_for_user(user_id);
            baseline_state.generate_session(goal_items, &memory_states, scenario.session_size, day)
        } {
            // Use baseline-generated session
            baseline_session
        } else {
            // Use real Iqrah scheduler

            // Get candidates for scheduling (sync batch operation)
            let candidates = self.get_candidates(
                user_id,
                goal_items,
                user_repo,
                now_ts,
                almost_due_window_days,
            )?;

            debug!("Got {} candidates for {}", candidates.len(), user_id);

            if candidates.is_empty() {
                debug!("No candidates - returning 0 minutes");
                return Ok(0.0);
            }

            // For ISS simulations, we skip the prerequisite gate to allow new students
            // to start learning immediately. In production, the scheduler would require
            // prerequisite mastery, but for simulation purposes we focus on verse-level
            // scheduling without the word-level prereq dependencies.
            let parent_map = HashMap::new();
            let parent_energies = HashMap::new();

            // Get user profile (via bandit if enabled)
            let profile = if scenario.enable_bandit {
                self.select_profile_via_bandit(user_id, &scenario.goal_id, user_repo, scheduler_rng)
                    .await?
            } else {
                UserProfile::balanced()
            };

            // Determine session mix configuration
            let mut mix_config = scenario
                .session_mix
                .unwrap_or_else(|| SessionMixConfig::default());

            // If not overridden using "session_mix" in YAML, compute min_new based on plan
            if scenario.session_mix.is_none() {
                mix_config.min_new_per_session = compute_min_new_for_plan(
                    goal_items.len(),
                    scenario.target_days,
                    scenario.session_size,
                );

                // For LARGE plans (>100 items), override percentages to heavily favor new items.
                // Default SessionMixConfig has pct_new=0.10 (10%), which starves coverage.
                // We need 95% new items to ensure adequate introduction rate.
                if goal_items.len() > 100 {
                    mix_config.pct_new = 0.95; // 95% new items
                    mix_config.pct_almost_mastered = 0.01;
                    mix_config.pct_almost_there = 0.01;
                    mix_config.pct_struggling = 0.01;
                    mix_config.pct_really_struggling = 0.02;
                }

                debug!(
                    "Computed min_new_per_session: {}, pct_new: {}",
                    mix_config.min_new_per_session, mix_config.pct_new
                );
            }

            // Generate session using REAL scheduler
            generate_session(
                candidates,
                parent_map,
                parent_energies,
                &profile,
                scenario.session_size,
                now_ts,
                SessionMode::MixedLearning,
                Some(&mix_config),
                None, // event_sink: use NullEventSink for ISS performance
            )
        };

        debug!(
            "Generated session with {} items for {}",
            session_items.len(),
            user_id
        );

        if session_items.is_empty() {
            debug!("Empty session - returning 0 minutes");
            return Ok(0.0);
        }

        // 2. Process each item in the session
        let mut minutes_spent = 0.0;
        let session_len = session_items.len();

        for node_id in session_items {
            // Track first introduction
            introduction_order.entry(node_id).or_insert_with(|| {
                let idx = *intro_index;
                *intro_index += 1;
                idx
            });

            // Get current memory state (sync - no async overhead)
            let state = user_repo
                .get_memory_state_sync(user_id, node_id)
                .unwrap_or_else(|| MemoryState::new_for_node(user_id.to_string(), node_id));

            // Compute elapsed days since last review
            let elapsed_days = if state.review_count > 0 {
                (now.timestamp_millis() - state.last_reviewed.timestamp_millis()) as f64
                    / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                0.0 // First time seeing this item
            };

            // Student attempts recall
            // For new items (stability=0), use a reasonable initial stability
            // so students have a fair chance of recalling on subsequent days
            let effective_stability = if state.stability < 0.1 {
                1.0
            } else {
                state.stability
            };
            let recall_result = brain.attempt_recall(
                effective_stability,
                state.difficulty,
                elapsed_days,
                state.review_count,
            );
            let grade = brain.determine_grade(recall_result);

            if let Some(acc) = accumulator {
                acc.record_review(
                    grade,
                    recall_result.retrievability,
                    elapsed_days as u32,
                    node_id,
                );
            }

            // Process review via REAL learning service
            let review_time = now + Duration::seconds((minutes_spent * 60.0) as i64);
            learning_service
                .process_review_at(user_id, node_id, grade, review_time)
                .await
                .context("Failed to process review")?;

            // Record stability for debug instrumentation (sync)
            if let Some(acc) = accumulator {
                // Get updated state after the review
                if let Some(new_state) = user_repo.get_memory_state_sync(user_id, node_id) {
                    acc.record_stability(new_state.stability, new_state.review_count);
                }
            }

            // Update baseline tracking (for FixedSRS, etc)
            let success = matches!(grade, ReviewGrade::Easy | ReviewGrade::Good);
            baseline_state.record_review(node_id, day, success);

            // Update time tracking using estimate_exercise_time
            let exercise_secs = estimate_exercise_time(
                state.stability,
                state.difficulty,
                state.review_count == 0,
                1.0, // Default reading fluency
            );
            minutes_spent += exercise_secs / 60.0;

            // Check early quit
            if brain.should_quit_early(minutes_spent) {
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

        if let Some(acc) = accumulator {
            acc.record_session(session_len, minutes_spent);
        }

        Ok(minutes_spent)
    }

    /// Initialize prior knowledge for a student using InitialPlacementService.
    ///
    /// Converts StudentParams to IntakeAnswers and uses the IPS for deterministic
    /// initialization based on known_surah_ids and vocab_known_pct.
    async fn initialize_prior_knowledge(
        &self,
        brain: &StudentBrain,
        user_id: &str,
        user_repo: &Arc<InMemoryUserRepository>,
        seed: u64,
    ) -> Result<()> {
        // Convert StudentParams to IntakeAnswers
        let intake = create_intake_from_params(&brain.params);

        // Create the IPS with concrete repository types
        let ips =
            InitialPlacementService::new(Arc::clone(user_repo), Arc::clone(&self.content_repo));

        // Apply the intake - this creates memory states for known verses and vocab
        let summary = ips.apply_intake(user_id, intake, seed).await?;

        debug!(
            "Initialized prior knowledge via IPS: {} verses, {} vocab nodes",
            summary.verses_initialized, summary.vocab_nodes_initialized
        );

        Ok(())
    }

    /// Get goal items (node IDs) for a goal.
    async fn get_goal_items(&self, goal_id: &str) -> Result<Vec<i64>> {
        // Try to get from real content repository via node_goals table
        let items = self.content_repo.get_nodes_for_goal(goal_id).await?;

        if !items.is_empty() {
            return Ok(items);
        }

        // Fallback: parse goal_id and get real node IDs from nodes table
        // Format: "surah:N" or "juz:N"
        if let Some(surah_num) = goal_id.strip_prefix("surah:") {
            if let Ok(num) = surah_num.parse::<i32>() {
                return self.get_verses_as_nodes(num, num).await;
            }
        }

        // Handle juz:N - Juz 30 is surahs 78-114
        if let Some(juz_num) = goal_id.strip_prefix("juz:") {
            if let Ok(num) = juz_num.parse::<i32>() {
                // Map juz to surah ranges (simplified - only juz 30 for now)
                let (start_surah, end_surah) = match num {
                    30 => (78, 114), // Juz 'Amma
                    29 => (67, 77),  // Juz Tabarak
                    28 => (58, 66),  // Juz Qad Sami'a
                    _ => {
                        warn!("Unsupported juz number: {}", num);
                        return Ok(Vec::new());
                    }
                };
                return self.get_verses_as_nodes(start_surah, end_surah).await;
            }
        }

        // Return empty if we can't resolve
        warn!("Could not resolve goal_id: {}", goal_id);
        Ok(Vec::new())
    }

    /// Helper to get verses for a range of surahs as node IDs
    async fn get_verses_as_nodes(&self, start_surah: i32, end_surah: i32) -> Result<Vec<i64>> {
        let mut node_ids = Vec::new();

        for surah_num in start_surah..=end_surah {
            let verses = self.content_repo.get_verses_for_chapter(surah_num).await?;

            for verse in verses {
                if let Ok(Some(node)) = self.content_repo.get_node_by_ukey(&verse.key).await {
                    node_ids.push(node.id);
                } else {
                    warn!("Node not found for ukey: {}, using synthetic ID", verse.key);
                    node_ids.push(verse_key_to_node_id(&verse.key));
                }
            }
        }

        Ok(node_ids)
    }

    /// Get candidate nodes for scheduling.
    ///
    /// Candidate selection uses a **unified pipeline** for all plan sizes:
    /// 1. New items (never reviewed): always eligible
    /// 2. Due or overdue: always eligible
    /// 3. Almost due (within window): eligible for proactive review
    ///
    /// FSRS due status affects candidate eligibility here; urgency scoring
    /// happens downstream in `generate_session()`.
    ///
    /// OPTIMIZATION: Uses synchronous batch lookup to avoid O(n) async calls.
    fn get_candidates(
        &self,
        user_id: &str,
        goal_items: &[i64],
        user_repo: &Arc<InMemoryUserRepository>,
        now_ts: i64,
        almost_due_window_days: u32,
    ) -> Result<Vec<CandidateNode>> {
        let mut candidates = Vec::with_capacity(goal_items.len());

        // OPTIMIZATION: Single batch fetch instead of 564 individual async calls
        let memory_basics = user_repo.get_memory_basics_sync(user_id, goal_items);
        let all_states = user_repo.get_memory_states_batch_sync(user_id, goal_items);

        // Calculate almost-due window in milliseconds
        let almost_due_window_ms = almost_due_window_days as i64 * 24 * 60 * 60 * 1000;

        // Diagnostic counters
        let mut count_new = 0;
        let mut count_due = 0;
        let mut count_almost_due = 0;

        for &node_id in goal_items {
            let (energy, next_due_ts) = memory_basics
                .get(&node_id)
                .map(|m| (m.energy, m.next_due_ts))
                .unwrap_or((0.0, 0)); // New items have 0 energy

            // ISS OVERRIDE: ALL goal items are candidates for simulation.
            // This ensures fair evaluation of scheduler quality across the entire goal set.
            // Per spec §4, FSRS status must NOT be the sole gate for candidate eligibility.

            // Get memory state from batch (no async call!)
            let (review_count, predicted_recall) = if let Some(state) = all_states.get(&node_id) {
                // Calculate FSRS retrievability: R = e^(-t/S) where t = days since last review
                let days_since_review = if state.review_count > 0 {
                    (now_ts - state.last_reviewed.timestamp_millis()) as f64 / (86400.0 * 1000.0)
                } else {
                    0.0
                };
                let predicted_recall = if state.stability > 0.01 {
                    (-days_since_review / state.stability).exp() as f32
                } else {
                    0.0
                };
                (state.review_count, predicted_recall.clamp(0.0, 1.0))
            } else {
                (0, 0.0) // New item
            };

            // Track what type of candidate this is (for diagnostics)
            let is_new = energy == 0.0 || next_due_ts == 0;
            let is_due_or_overdue = next_due_ts > 0 && next_due_ts <= now_ts;
            let is_almost_due = almost_due_window_ms > 0
                && next_due_ts > now_ts
                && next_due_ts <= now_ts + almost_due_window_ms;

            if is_new {
                count_new += 1;
            } else if is_due_or_overdue {
                count_due += 1;
            } else if is_almost_due {
                count_almost_due += 1;
            }

            let candidate = CandidateNode {
                id: node_id,
                foundational_score: 0.5, // Default (ISS simplification)
                influence_score: 0.5,    // Default (ISS simplification)
                difficulty_score: 0.3,   // Default (ISS simplification)
                energy,
                next_due_ts,
                quran_order: node_id, // Use node_id as tie-breaker
                review_count,
                predicted_recall,
            };

            candidates.push(candidate);
        }

        debug!(
            "Candidates: {} total ({} new, {} due, {} almost-due) | window={}d",
            candidates.len(),
            count_new,
            count_due,
            count_almost_due,
            almost_due_window_days
        );

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
