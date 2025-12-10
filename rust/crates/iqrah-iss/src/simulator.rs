//! Main simulation orchestrator.
//!
//! The Simulator runs virtual students through the real Iqrah scheduling pipeline.
//! ISS **orchestrates** the simulation; `iqrah-core` **decides** what to schedule.

use crate::baselines::{FixedSrsBaseline, GraphTopoBaseline, PageOrderBaseline, RandomBaseline};
use crate::debug_stats::{StudentDebugAccumulator, StudentDebugSummary};

use crate::{
    InMemoryUserRepository, Scenario, SchedulerVariant, SessionGenerator, SimulationConfig,
    SimulationMetrics, StudentBrain, StudentSanityData,
};

use crate::config::compute_almost_due_window;
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
    /// Graph-topo baseline (ISS v2.1)
    GraphTopo(GraphTopoBaseline),
    /// Oracle baseline (force perfect recall, use iqrah scheduler)
    OraclePerfect,
}

impl BaselineState {
    /// Create baseline state from scheduler variant.
    fn new(variant: SchedulerVariant, seed: u64) -> Self {
        match variant {
            SchedulerVariant::IqrahDefault => Self::None,
            SchedulerVariant::BaselinePageOrder => Self::PageOrder(PageOrderBaseline::new()),
            SchedulerVariant::BaselineFixedSrs => Self::FixedSrs(FixedSrsBaseline::new()),
            SchedulerVariant::BaselineRandom => Self::Random(RandomBaseline::new(seed)),
            SchedulerVariant::BaselineGraphTopo => Self::GraphTopo(GraphTopoBaseline::new(seed)),
            SchedulerVariant::BaselineOraclePerfect => Self::OraclePerfect,
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
            Self::GraphTopo(b) => {
                Some(b.generate_session(goal_items, memory_states, session_size, current_day, now))
            }
            Self::OraclePerfect => None, // Use Iqrah scheduler
        }
    }

    /// Record a review result for baselines that track reviews.
    fn record_review(&mut self, node_id: i64, current_day: u32, success: bool) {
        match self {
            Self::FixedSrs(b) => b.record_review(node_id, current_day, success),
            Self::GraphTopo(b) => b.record_review(node_id), // GraphTopo tracks for fairness
            Self::OraclePerfect => {}                       // No internal state tracking needed
            _ => {}
        }
    }
}

// ============================================================================
// ISS v2.6: Learning Cluster for Batch-Based Introduction
// ============================================================================

/// Represents a learning cluster for tracking consolidation progress.
///
/// A cluster is a set of recently introduced items that must stabilize
/// before introducing more. New items have higher weight in stability
/// calculation to prevent runaway expansion.
#[derive(Debug, Clone, Default)]
struct LearningCluster {
    /// Items in the cluster with their introduction timestamp: (node_id, introduced_on_day)
    items: Vec<(i64, u32)>,
}

impl LearningCluster {
    /// Create a new empty cluster.
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Check if cluster is empty (bootstrap phase).
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Number of items in the cluster.
    fn len(&self) -> usize {
        self.items.len()
    }

    /// Get a HashSet of all node IDs in the cluster for efficient membership checking.
    fn node_ids(&self) -> std::collections::HashSet<i64> {
        self.items.iter().map(|(node_id, _)| *node_id).collect()
    }

    /// Add a batch of items to the cluster (prevents duplicates).
    fn add_batch(&mut self, node_ids: &[i64], current_day: u32) {
        for &node_id in node_ids {
            // Skip if already in cluster (prevent duplicate entries)
            if !self.items.iter().any(|(id, _)| *id == node_id) {
                self.items.push((node_id, current_day));
            }
        }
    }

    /// Compute weighted cluster energy.
    /// New items (recently introduced) have weight 1.0
    /// Old items (introduced long ago) have decaying weight
    fn compute_weighted_energy(
        &self,
        memory_states: &HashMap<i64, MemoryState>,
        current_day: u32,
    ) -> f64 {
        if self.items.is_empty() {
            return f64::INFINITY; // No constraint when empty (bootstrap)
        }

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (node_id, introduced_day) in &self.items {
            // Find corresponding memory state
            let energy = memory_states
                .get(node_id)
                .map(|ms| ms.energy as f64)
                .unwrap_or(0.0);

            // Compute weight based on age (exponential decay)
            let age = current_day.saturating_sub(*introduced_day);
            let weight = compute_maturity_weight(age);

            weighted_sum += energy * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Prune mastered items from cluster (optional).
    fn prune_mastered(
        &mut self,
        memory_states: &HashMap<i64, MemoryState>,
        current_day: u32,
        energy_threshold: f64,
        days_threshold: u32,
    ) {
        self.items.retain(|(node_id, introduced_day)| {
            if let Some(ms) = memory_states.get(node_id) {
                let age = current_day.saturating_sub(*introduced_day);
                let is_mastered = ms.energy as f64 >= energy_threshold && age >= days_threshold;
                !is_mastered // Keep if NOT mastered
            } else {
                true // Keep if no state found
            }
        });
    }
}

/// Compute weight for item based on age (days since introduction).
///
/// New items have weight 1.0 (high impact on cluster stability)
/// Old items have exponentially decaying weight (low impact)
fn compute_maturity_weight(age_days: u32) -> f64 {
    // Exponential decay with half-life of 20 days
    // age=0  → weight=1.00 (brand new, full impact)
    // age=10 → weight=0.61
    // age=20 → weight=0.37 (half-life)
    // age=40 → weight=0.14
    let half_life = 20.0;
    (-(age_days as f64) * 0.693 / half_life).exp()
}

use crate::events::{
    event_channel, EnergyHistogram as EventEnergyHistogram, EventSender, SessionMixSummary,
    SimulationEvent, TransitionCause,
};

/// Main simulator orchestrator.
pub struct Simulator {
    /// Real content repository (read-only access to content.db)
    content_repo: Arc<dyn ContentRepository>,

    /// Simulation configuration
    config: SimulationConfig,

    /// Event sender for diagnostics (enabled via config.event_log_enabled)
    event_sender: EventSender,
}

impl Simulator {
    /// Create a new simulator with content repository and configuration.
    pub fn new(content_repo: Arc<dyn ContentRepository>, config: SimulationConfig) -> Self {
        let (event_sender, _receiver) = event_channel(config.event_log_enabled);
        Self {
            content_repo,
            config,
            event_sender,
        }
    }

    /// Create a simulator with an external event sender.
    /// Use this when you want to collect events via the corresponding EventReceiver.
    pub fn with_event_sender(
        content_repo: Arc<dyn ContentRepository>,
        config: SimulationConfig,
        event_sender: EventSender,
    ) -> Self {
        Self {
            content_repo,
            config,
            event_sender,
        }
    }

    /// Get the event sender for this simulator.
    pub fn event_sender(&self) -> &EventSender {
        &self.event_sender
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
        Option<StudentSanityData>,
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
            debug!("No goal items found for goal_id: {}", scenario.goal_id);
            return Ok((SimulationMetrics::default(), user_repo, None, None));
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

        // Create sanity data collector (always collected for comparisons)
        let mut sanity_data = StudentSanityData::default();

        // ISS v2.6: Create learning cluster for batch-based introduction
        // Seed cluster with items that are already active from initial placement
        let mut learning_cluster = LearningCluster::new();
        {
            let initial_states = user_repo.get_memory_states_batch_sync(&user_id, &goal_items);
            let initial_active: Vec<i64> = initial_states
                .iter()
                .filter(|(_, s)| s.review_count > 0)
                .map(|(&id, _)| id)
                .collect();

            if !initial_active.is_empty() {
                // Add with day=0 (treated as already established)
                learning_cluster.add_batch(&initial_active, 0);
            }
        }

        // 9. Main simulation loop
        for day in 0..scenario.target_days {
            if brain.has_given_up() {
                debug!("Student gave up at day {}", day);
                if let Some(acc) = debug_accumulator.as_mut() {
                    acc.give_up = true;
                    acc.day_of_give_up = Some(day);
                }

                // Record GaveUp event
                self.event_sender.record(SimulationEvent::GaveUp {
                    day,
                    frustration: brain.frustration as f32,
                    trigger: "frustration_threshold".to_string(),
                });

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

            // Dynamic session size from student model (ISS v2.1)
            let session_size = brain.sample_daily_reviews();

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
                    &mut sanity_data,
                    session_size,
                    dynamic_window,
                    &mut learning_cluster,
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

        // Finalize debug summary if accumulator exists
        let debug_summary = debug_accumulator.map(|acc| acc.finish());

        // Populate Sanity Data summaries
        sanity_data.unique_items_seen = sanity_data.per_item_reviews.len();
        sanity_data.unique_items_mastered = metrics.items_mastered;

        // Populate energy and retrievability histograms
        let relevant_states = user_repo.get_memory_states_batch_sync(&user_id, &goal_items);
        let end_time = start_time + Duration::days(scenario.target_days as i64);

        for (_, state) in relevant_states {
            sanity_data.energy_histogram.add(state.energy);

            // Calculate current retrievability
            let elapsed_days = (end_time.timestamp_millis()
                - state.last_reviewed.timestamp_millis()) as f64
                / (24.0 * 60.0 * 60.0 * 1000.0);

            // Re-use logic from simulate_day for effective stability
            let effective_stability = if state.stability < 0.1 {
                1.0
            } else {
                state.stability
            };

            let recall_prob = brain.compute_recall_probability(
                effective_stability,
                elapsed_days,
                0.0, // progress_in_session - assume start of session context
            );
            sanity_data.retrievability_histogram.add(recall_prob);
        }

        Ok((metrics, user_repo, debug_summary, Some(sanity_data)))
    }

    /// Run simulation for a single student.
    pub async fn simulate_student(
        &self,
        scenario: &Scenario,
        student_index: usize,
    ) -> Result<(
        SimulationMetrics,
        Option<StudentDebugSummary>,
        Option<StudentSanityData>,
    )> {
        let (metrics, _, summary, sanity) = self
            .simulate_student_internal(scenario, student_index)
            .await?;
        Ok((metrics, summary, sanity))
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
        Option<StudentSanityData>,
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
        sanity_data: &mut StudentSanityData,
        session_size: usize,
        almost_due_window_days: u32,
        learning_cluster: &mut LearningCluster,
    ) -> Result<f64> {
        let now = start_time + Duration::days(day as i64);
        let now_ts = now.timestamp_millis();

        // Configure Brain for Oracle mode if needed
        if matches!(baseline_state, BaselineState::OraclePerfect) {
            brain.force_perfect_recall = true;
        } else {
            brain.force_perfect_recall = false;
        }

        // 1. Generate session - branch based on scheduler variant
        let session_items = if let Some(baseline_session) = {
            // Get memory states for baseline session generation
            let memory_states = user_repo.get_all_states_for_user(user_id);
            // Use dynamic session size
            baseline_state.generate_session(goal_items, &memory_states, session_size, day)
        } {
            // Use baseline-generated session
            baseline_session
        } else {
            // Use real Iqrah scheduler

            // === ISS v2.6: Compute cluster gate BEFORE candidate selection ===
            // This determines which items are eligible for scheduling

            // Build memory state map for cluster energy computation
            let states_map: HashMap<i64, MemoryState> = goal_items
                .iter()
                .filter_map(|id| {
                    user_repo
                        .get_memory_state_sync(user_id, *id)
                        .map(|s| (*id, s))
                })
                .collect();

            // Count active items (already introduced) for working set limit
            let active_count = states_map.values().filter(|s| s.review_count > 0).count();

            // ISS v2.6: Batch-based cluster gating
            // Step 1: Compute weighted cluster energy (INFINITY if empty)
            let cluster_energy = learning_cluster.compute_weighted_energy(&states_map, day);

            // Step 2: Check if cluster is ready for expansion
            let can_expand = if learning_cluster.is_empty() {
                true // Bootstrap: Always allow first batch
            } else {
                cluster_energy >= brain.params.cluster_stability_threshold
            };

            // Step 3: Determine how many new items to allow (used for filtering)
            let new_items_limit = if can_expand {
                // Check working set limit first
                if active_count >= brain.params.max_working_set {
                    0 // At working set limit - consolidate
                } else {
                    // Within limit - allow batch
                    let remaining_capacity = brain.params.max_working_set - active_count;
                    let batch_size = brain.params.cluster_expansion_batch_size;
                    batch_size.min(remaining_capacity)
                }
            } else {
                0 // Cluster not stable - consolidate
            };

            // Get cluster node IDs for membership filtering
            let cluster_node_ids = learning_cluster.node_ids();

            debug!(
                "ClusterGate day={}: cluster_size={}, cluster_energy={:.3}, can_expand={}, active={}, new_limit={}",
                day,
                learning_cluster.len(),
                cluster_energy,
                can_expand,
                active_count,
                new_items_limit
            );

            // Get ALL candidates for scheduling (sync batch operation)
            let all_candidates = self.get_candidates(
                user_id,
                goal_items,
                user_repo,
                now_ts,
                almost_due_window_days,
                &brain.params,
            )?;

            // === ISS v2.6: Filter candidates by cluster membership ===
            // Only include:
            // 1. Items already in cluster (being learned)
            // 2. OR new items (review_count=0) if gate is open, up to limit
            let mut new_items_allowed = new_items_limit;
            let candidates: Vec<_> = all_candidates
                .into_iter()
                .filter(|c| {
                    // Item is in cluster - always include
                    if cluster_node_ids.contains(&c.id) {
                        return true;
                    }
                    // Item is new (review_count=0) and gate allows more
                    if c.review_count == 0 && new_items_allowed > 0 {
                        new_items_allowed -= 1;
                        return true;
                    }
                    // Otherwise exclude - not in cluster and not eligible for introduction
                    false
                })
                .collect();

            debug!(
                "Filtered candidates: {} from {} (cluster: {}, new_allowed: {})",
                candidates.len(),
                goal_items.len(),
                cluster_node_ids.len(),
                new_items_limit
            );

            if candidates.is_empty() {
                debug!("No candidates - returning 0 minutes");
                return Ok(0.0);
            }

            // === Event tracking: Capture candidate info before scheduling ===
            let candidate_info: Vec<(i64, f32)> =
                candidates.iter().map(|c| (c.id, c.energy)).collect();

            // ISS v2.6: Enable prerequisite gate using actual graph topology
            // Fetch parent_map and parent_energies from content repository
            let parent_map = self
                .content_repo
                .get_prerequisite_parents(goal_items)
                .await
                .unwrap_or_default();

            // Build parent_energies from current memory states
            let all_parent_ids: Vec<i64> = parent_map.values().flatten().copied().collect();
            let parent_energies: HashMap<i64, f32> = if all_parent_ids.is_empty() {
                HashMap::new()
            } else {
                user_repo
                    .get_parent_energies(user_id, &all_parent_ids)
                    .await
                    .unwrap_or_default()
            };

            // Get user profile (via bandit if enabled)
            let profile = if scenario.enable_bandit {
                self.select_profile_via_bandit(user_id, &scenario.goal_id, user_repo, scheduler_rng)
                    .await?
            } else {
                UserProfile::balanced()
            };

            // Determine session mix configuration
            // Note: Cluster gate already filtered candidates, so we use default mix config
            // The candidate filtering controls introduction, not band composition percentages
            let mix_config = scenario
                .session_mix
                .unwrap_or_else(|| SessionMixConfig::default());

            // Generate session using REAL scheduler
            let session = generate_session(
                candidates,
                parent_map,
                parent_energies,
                &profile,
                session_size, // Use dynamic session_size
                now_ts,
                SessionMode::MixedLearning,
                Some(&mix_config),
                None, // event_sink: use NullEventSink for ISS performance
            );

            // === Event tracking: Record ItemSkipped for candidates not in session ===
            if self.event_sender.is_enabled() {
                use std::collections::HashSet;
                let session_set: HashSet<i64> = session.iter().cloned().collect();

                for (item_id, energy) in &candidate_info {
                    if !session_set.contains(item_id) {
                        // Determine skip reason based on energy
                        let reason = if energy == &0.0 {
                            crate::events::SkipReason::MixCapReached // New item hit mix cap
                        } else if session.len() >= session_size {
                            crate::events::SkipReason::SessionFull
                        } else {
                            crate::events::SkipReason::LowPriority
                        };

                        self.event_sender.record(SimulationEvent::ItemSkipped {
                            day,
                            item_id: *item_id,
                            urgency_score: 0.0, // Would need scheduler to expose this
                            energy: *energy,
                            reason,
                        });
                    }
                }
            }

            session
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

        // === Event tracking: Capture frustration before session ===
        let frustration_before = brain.frustration;

        for (idx, node_id) in session_items.iter().enumerate() {
            let node_id = *node_id;

            // Get current memory state (sync - no async overhead)
            let state = user_repo
                .get_memory_state_sync(user_id, node_id)
                .unwrap_or_else(|| MemoryState::new_for_node(user_id.to_string(), node_id));

            // Track first introduction
            let is_new_item = !introduction_order.contains_key(&node_id);
            introduction_order.entry(node_id).or_insert_with(|| {
                let idx = *intro_index;
                *intro_index += 1;
                idx
            });

            // === Event: ItemIntroduced (first time seeing this item) ===
            if is_new_item {
                self.event_sender.record(SimulationEvent::ItemIntroduced {
                    day,
                    item_id: node_id,
                    session_idx: idx as u32,
                });
            }

            // === Event: ItemScheduled ===
            self.event_sender.record(SimulationEvent::ItemScheduled {
                day,
                item_id: node_id,
                urgency_score: 0.0, // TODO: capture from scheduler
                energy: state.energy as f32,
                recall: 0.0, // Will compute below
                category: if is_new_item {
                    crate::events::SessionCategory::New
                } else if state.energy < 0.3 {
                    crate::events::SessionCategory::Struggling
                } else {
                    crate::events::SessionCategory::Due
                },
            });

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

            // Calculate progress in session (0.0 to 1.0)
            let progress = if session_len > 1 {
                idx as f64 / (session_len - 1) as f64
            } else {
                0.0
            };

            let recall_result = brain.attempt_recall(
                effective_stability,
                state.difficulty,
                elapsed_days,
                state.review_count,
                state.energy, // ISS v2.4: Pass energy for blended recall
                progress,
            );
            // No manual override needed - brain handles it
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

            // Get updated state for event tracking
            let new_state = user_repo
                .get_memory_state_sync(user_id, node_id)
                .unwrap_or_else(|| state.clone());

            // === Event: ReviewOutcome ===
            let success = matches!(grade, ReviewGrade::Easy | ReviewGrade::Good);
            self.event_sender.record(SimulationEvent::ReviewOutcome {
                day,
                item_id: node_id,
                success,
                recall_before: recall_result.retrievability as f32,
                recall_after: recall_result.retrievability as f32, // Same since R depends on S
                energy_before: state.energy as f32,
                energy_after: new_state.energy as f32,
            });

            // === Event: EnergyTransition (for review outcomes) ===
            self.event_sender.record_energy_transition(
                day,
                node_id,
                state.energy as f32,
                new_state.energy as f32,
                if success {
                    TransitionCause::ReviewSuccess
                } else {
                    TransitionCause::ReviewFail
                },
            );

            // Record stability for debug instrumentation (sync)
            if let Some(acc) = accumulator {
                acc.record_stability(new_state.stability, new_state.review_count);
            }

            // Update Sanity Data (ISS v2.1)
            *sanity_data.per_item_reviews.entry(node_id).or_insert(0) += 1;

            // Update baseline tracking (for FixedSRS, etc)
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

        // Record daily session count for sanity histogram
        // session_len is the number of reviews performed
        sanity_data.record_day(session_len);
        sanity_data.days_active += 1; // Mark this day as active

        // === Event: FrustrationSpike (if frustration increased significantly) ===
        let frustration_delta = brain.frustration - frustration_before;
        if frustration_delta > 0.1 {
            self.event_sender.record(SimulationEvent::FrustrationSpike {
                day,
                frustration: brain.frustration as f32,
                delta: frustration_delta as f32,
                cause: format!("Session failures: {} fail reviews", session_len),
            });
        }

        if let Some(acc) = accumulator {
            acc.record_session(session_len, minutes_spent);
            // We should record day end stats here or in caller? Caller does it.
            // But caller needs frustration/failure from brain.
            // Actually simulate_student_internal loop handles record_day_end on debug_accumulator.
        }

        // === ISS v2.3: Apply mastery-dependent daily energy drift ===
        // This ensures energy tracks FSRS decay between reviews, giving the
        // scheduler accurate urgency signals for items that are weakening.
        // Higher mastery items decay slower (spacing effect).
        self.apply_daily_energy_drift(
            user_id,
            goal_items,
            user_repo,
            now.timestamp_millis(),
            &brain.params,
            day,
        )?;

        // === ISS v2.6: Track cluster state (unconditionally) ===
        // This must happen regardless of event logging to maintain cluster state
        {
            let states = user_repo.get_memory_states_batch_sync(user_id, goal_items);

            // Track newly introduced items in cluster (review_count == 1)
            let newly_introduced: Vec<i64> = states
                .iter()
                .filter(|(_, s)| s.review_count == 1)
                .map(|(&id, _)| id)
                .collect();

            if !newly_introduced.is_empty() {
                learning_cluster.add_batch(&newly_introduced, day);
            }

            // Optional auto-pruning of mastered items
            if brain.params.cluster_auto_prune_enabled {
                learning_cluster.prune_mastered(
                    &states,
                    day,
                    brain.params.cluster_prune_energy_threshold,
                    brain.params.cluster_prune_days_threshold,
                );
            }
        }

        // === Event Tracking: DaySnapshot ===
        if self.event_sender.is_enabled() {
            let states = user_repo.get_memory_states_batch_sync(user_id, goal_items);
            let energies: Vec<f32> = states.values().map(|s| s.energy as f32).collect();
            let energy_dist = EventEnergyHistogram::from_energies(&energies);

            // Count new items introduced today (review_count == 1 means first review)
            let introduced_today = states.values().filter(|s| s.review_count == 1).count() as u32;

            // Coverage: mean energy for reviewed items
            let reviewed_energies: Vec<f32> = states
                .values()
                .filter(|s| s.review_count > 0)
                .map(|s| s.energy as f32)
                .collect();
            let coverage_mean = if reviewed_energies.is_empty() {
                0.0
            } else {
                reviewed_energies.iter().sum::<f32>() / reviewed_energies.len() as f32
            };
            // ISS v2.3: Compute capacity metrics
            let active_items = states.values().filter(|s| s.review_count > 0).count();
            // ISS v2.4 fix: use FSRS avg interval
            let all_states: Vec<_> = states.values().cloned().collect();
            let avg_interval = Self::compute_actual_avg_interval(&all_states) as f32;
            let maintenance_burden = active_items as f32 / avg_interval.max(1.0);
            let capacity_util = maintenance_burden / brain.params.session_capacity as f32;
            let intro_rate = Self::compute_sustainable_intro_rate(&all_states, &brain.params);

            // Note: add_batch and prune_mastered are called unconditionally above
            // (lines 1010-1034), so we don't duplicate them here

            // ISS v2.6: Compute cluster metrics using LearningCluster
            let cluster_energy_val = learning_cluster.compute_weighted_energy(&states, day);
            let cluster_energy_gated = cluster_energy_val
                < brain.params.cluster_stability_threshold
                && !learning_cluster.is_empty();
            let working_set_gated = active_items >= brain.params.max_working_set;

            // Compute actual session size (scheduled today)
            let actual_session_size = session_len as f32;

            // Predicted maintenance
            let predicted_maintenance = maintenance_burden;

            self.event_sender.record(SimulationEvent::DaySnapshot {
                day,
                energy_distribution: energy_dist,
                coverage_mean_r: coverage_mean,
                introduced_count: introduced_today,
                reviewed_count: session_len as u32,
                urgent_backlog: 0, // TODO: track from scheduling
                session_mix: SessionMixSummary {
                    new_count: introduced_today as u32,
                    review_count: (session_len - introduced_today as usize) as u32,
                    total_sessions: 1,
                    // Note: session_len includes both.
                },
                active_items,
                capacity_utilization: capacity_util,
                sustainable_intro_rate: intro_rate as f32,
                // ISS v2.4: Capacity diagnostics
                avg_review_interval: avg_interval,
                predicted_maintenance,
                actual_session_size,
                // ISS v2.6: Cluster stability metrics
                cluster_size: learning_cluster.len(),
                cluster_energy: if cluster_energy_val.is_finite() {
                    cluster_energy_val as f32
                } else {
                    1.0 // Infinity means empty cluster, display as 1.0
                },
                cluster_threshold: brain.params.cluster_stability_threshold as f32,
                intro_gated_by_cluster: cluster_energy_gated,
                intro_gated_by_working_set: working_set_gated,
            });
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

    /// Sigmoid damping function for smooth capacity approach (ISS v2.3).
    ///
    /// When x approaches 0 (capacity full), output smoothly drops to 0.
    /// When x is large (capacity available), output stays near 1.
    fn sigmoid_damping(x: f64, k: f64) -> f64 {
        1.0 / (1.0 + (-k * (x - 0.5)).exp())
    }

    /// Compute actual average review interval from FSRS stability (ISS v2.4 fix).
    fn compute_actual_avg_interval(items: &[MemoryState]) -> f64 {
        let active: Vec<_> = items.iter().filter(|i| i.review_count > 0).collect();

        if active.is_empty() {
            return 7.0; // Bootstrap value
        }

        // MemoryState has stability directly on it (flattened FSRS)
        let total_stability: f64 = active.iter().map(|i| i.stability).sum();
        let mean_stability = total_stability / active.len() as f64;

        // Clamp to avoid extreme values
        mean_stability.clamp(3.0, 60.0)
    }

    /// Adaptive sigmoid damping based on coverage (ISS v2.4 fix).
    fn adaptive_sigmoid_damping(capacity_margin: f64, current_coverage: f64) -> f64 {
        if current_coverage < 0.20 {
            // Low coverage: minimal damping (be aggressive)
            1.0
        } else if current_coverage < 0.50 {
            // Medium coverage: light damping
            Self::sigmoid_damping(capacity_margin, 5.0)
        } else {
            // High coverage: full damping
            Self::sigmoid_damping(capacity_margin, 10.0)
        }
    }

    /// Compute sustainable introduction rate based on current capacity (ISS v2.3).
    ///
    /// Returns the number of new items that can be sustainably introduced
    /// per day given current maintenance burden and available session capacity.
    ///
    /// Formula:
    /// ```text
    /// available_capacity = (1 - capacity_used) × (1 - headroom)
    /// sustainable_rate = σ(available) × capacity / burden_per_new
    /// ```
    fn compute_sustainable_intro_rate(
        items: &[MemoryState],
        student_params: &crate::brain::StudentParams,
    ) -> f64 {
        let active_count = items.iter().filter(|i| i.review_count > 0).count();

        if active_count == 0 {
            // No maintenance burden yet, introduce freely but conservatively
            return student_params.session_capacity * 0.8;
        }

        // =========================================================================
        // ISS v2.6: Working Set Limit Check
        // =========================================================================
        // If at or above the working set limit, stop introducing and consolidate
        if active_count >= student_params.max_working_set {
            return 0.0; // At limit - consolidate existing items
        }

        // Compute working set factor: 1.0 when plenty of room, 0.0 when at limit
        let working_set_factor =
            Self::compute_working_set_factor(active_count, student_params.max_working_set);

        // =========================================================================
        // ISS v2.6: Cluster Stability Gate
        // =========================================================================
        // Compute mean energy of active items (the "cluster")
        let cluster_energy = Self::compute_cluster_energy(items);

        // Bootstrap exception: Skip cluster gate when fewer than 10 active items
        // This allows initial items to be introduced before the cluster can stabilize
        const BOOTSTRAP_THRESHOLD: usize = 10;
        let skip_cluster_gate = active_count < BOOTSTRAP_THRESHOLD;

        // If cluster energy is below threshold and not in bootstrap phase, consolidate
        if !skip_cluster_gate && cluster_energy < student_params.cluster_stability_threshold {
            return 0.0; // Cluster weak - consolidate
        }

        // =========================================================================
        // Original capacity-based logic (ISS v2.3/v2.4)
        // =========================================================================
        let active_count_f64 = active_count as f64;

        // ISS v2.4: Use actual FSRS stability
        let avg_review_interval = Self::compute_actual_avg_interval(items);

        // Compute maintenance burden (reviews/day needed for existing items)
        let maintenance_burden = active_count_f64 / avg_review_interval;

        // Compute capacity utilization
        let capacity_used = maintenance_burden / student_params.session_capacity;

        // If heavily over capacity (>110%), return minimum.
        if capacity_used >= 1.1 {
            return 1.0;
        }

        // Apply headroom reserve (keep buffer for urgent items)
        let available_capacity = (1.0 - capacity_used) * (1.0 - student_params.headroom_reserve);

        // Compute review burden per new item
        let review_burden_per_new =
            student_params.reviews_to_stability / student_params.days_to_stability;

        let raw_intro_rate = available_capacity / review_burden_per_new;

        // Calculate current coverage for adaptive damping (mean Retrievability)
        // Only consider introduced items (review_count > 0)
        let introduced_items: Vec<_> = items.iter().filter(|i| i.review_count > 0).collect();
        let current_coverage = if !introduced_items.is_empty() {
            // Use energy as proxy for retrievability (or stability->R conversion)
            introduced_items.iter().map(|i| i.energy).sum::<f64>() / introduced_items.len() as f64
        } else {
            0.0
        };

        // Apply adaptive sigmoid damping
        let capacity_margin = 1.0 - capacity_used;
        let damping = Self::adaptive_sigmoid_damping(capacity_margin, current_coverage);

        // Apply both damping and working set factor
        let rate = raw_intro_rate * damping * student_params.session_capacity * working_set_factor;

        // Ensure at least 1 if we have capacity and damping didn't kill it
        if rate < 1.0 && capacity_margin > 0.05 && working_set_factor > 0.0 {
            return 1.0;
        }

        rate.max(1.0)
    }

    /// Compute working set factor: 1.0 when plenty of room, 0.0 when at limit (ISS v2.6)
    ///
    /// # Behavior
    /// - Utilization < 70%: Full introduction rate (1.0)
    /// - Utilization 70-100%: Linear decay from 1.0 to 0.0
    /// - Utilization >= 100%: No introduction (0.0)
    fn compute_working_set_factor(active_count: usize, max_working_set: usize) -> f64 {
        if max_working_set == 0 {
            return 1.0; // No limit configured
        }
        if active_count >= max_working_set {
            return 0.0; // At or over limit
        }
        let utilization = active_count as f64 / max_working_set as f64;
        if utilization < 0.7 {
            1.0 // Plenty of room, no restriction
        } else {
            // Linear decay from 0.7 to 1.0 utilization
            // 0.7 → 1.0, 1.0 → 0.0
            (1.0 - utilization) / 0.3
        }
    }

    /// Compute mean energy of active items (ISS v2.6)
    ///
    /// Active items = items with review_count > 0
    /// Returns 1.0 if no active items (allows bootstrap)
    fn compute_cluster_energy(items: &[MemoryState]) -> f64 {
        let active_items: Vec<_> = items.iter().filter(|i| i.review_count > 0).collect();
        if active_items.is_empty() {
            return 1.0; // Empty cluster is "stable" (allows bootstrap)
        }
        active_items.iter().map(|i| i.energy).sum::<f64>() / active_items.len() as f64
    }
    ///
    /// This couples the cognitive model (energy) to the forgetting model (FSRS)
    /// by making energy reflect current predicted recall rather than just review history.
    ///
    /// # Formula
    /// ```text
    /// For each goal item with review_count > 0:
    ///   elapsed_days = days_since_last_review
    ///   R_now = (1 + elapsed_days/(9*S))^(-1)  [FSRS formula]
    ///   target = R_now^gamma                    [emphasis transform]
    ///   E_new = (1 - alpha) * E_old + alpha * target  [EMA]
    /// Apply mastery-dependent daily energy drift (ISS v2.3 + v2.4).
    ///
    /// ISS v2.3: drift_rate(E) = α_max × (1 - E^k) + α_min × E^k
    /// ISS v2.4: Additional protection for young items (review_count < 5)
    ///
    /// This gives high-energy (mastered) items and young items lower decay rates,
    /// preventing the death spiral where items decay before they can be reinforced.
    ///
    /// # Parameters
    /// * `student_params` - Contains drift_alpha_max, drift_alpha_min, drift_mastery_exponent, drift_energy_floor
    ///
    /// # Performance
    /// Uses batch fetching for O(1) lock overhead instead of O(n) async calls.
    fn apply_daily_energy_drift(
        &self,
        user_id: &str,
        goal_items: &[i64],
        user_repo: &Arc<InMemoryUserRepository>,
        _now_ts: i64,
        student_params: &crate::brain::StudentParams,
        day: u32,
    ) -> Result<()> {
        // Batch fetch for performance (single lock instead of n locks)
        let states = user_repo.get_memory_states_batch_sync(user_id, goal_items);

        for (&node_id, state) in &states {
            // CRITICAL: Only drift items that have been reviewed at least once
            // Unseen items should remain at energy = 0.0
            if state.review_count == 0 {
                continue;
            }

            let old_energy = state.energy;

            // === ISS v2.4: Mastery AND maturity-dependent drift rate ===
            // Uses compute_drift_rate_v2 which applies:
            // 1. Energy protection: high energy → lower drift (spacing effect)
            // 2. Maturity protection: young items (0-5 reviews) → 30% slower drift
            let drift_rate = student_params.compute_drift_rate_v2(old_energy, state.review_count);

            // Apply simple decay: E_new = E_old × (1 - drift_rate)
            let mut new_energy = old_energy * (1.0 - drift_rate);

            // Apply floor to prevent complete collapse
            new_energy = new_energy.max(student_params.drift_energy_floor);

            // Clamp to valid range
            new_energy = new_energy.clamp(0.0, 1.0);

            // Only update if changed significantly (avoid unnecessary writes)
            if (new_energy - old_energy).abs() > 0.001 {
                let mut updated_state = state.clone();
                updated_state.energy = new_energy;
                user_repo.save_memory_state_sync(&updated_state);

                // Record energy transition event if bucket changed
                self.event_sender.record_energy_transition(
                    day,
                    node_id,
                    old_energy as f32,
                    new_energy as f32,
                    TransitionCause::Decay,
                );
            }
        }

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
    /// ISS v2.4 Fix 2: Boosts urgency for items where safe_days < fsrs_due_days * 0.8
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
        student_params: &crate::brain::StudentParams,
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

            // === ISS v2.4 Fix 2: Urgency boost for at-risk items ===
            // If energy will decay below critical threshold before FSRS triggers review,
            // boost the effective energy to prioritize this item in session selection.
            let boosted_energy = if review_count > 0 && energy > 0.0 && next_due_ts > now_ts {
                // Compute days until FSRS says item is due
                let fsrs_due_days = (next_due_ts - now_ts) as f64 / (86400.0 * 1000.0);

                // Compute days until energy drops below critical threshold
                let safe_days = student_params.compute_safe_interval(energy as f64, review_count);

                // If safe interval is less than 80% of FSRS interval, boost urgency
                if safe_days < fsrs_due_days * 0.8 {
                    // Boost energy by 1.5x to increase priority (capped at 1.0)
                    (energy * 1.5).min(1.0)
                } else {
                    energy
                }
            } else {
                energy
            };

            let candidate = CandidateNode {
                id: node_id,
                foundational_score: 0.5, // Default (ISS simplification)
                influence_score: 0.5,    // Default (ISS simplification)
                difficulty_score: 0.3,   // Default (ISS simplification)
                energy: boosted_energy,
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

    // =========================================================================
    // ISS v2.2: Energy Drift Tests (matches spec exactly)
    // =========================================================================

    mod drift_tests {
        use super::*;
        use crate::metrics::retrievability;
        use chrono::{Duration, Utc};
        use iqrah_core::domain::MemoryState;

        /// Helper to create a memory state for testing drift
        fn make_state(
            user_id: &str,
            node_id: i64,
            stability: f64,
            energy: f64,
            review_count: u32,
            last_reviewed: chrono::DateTime<Utc>,
        ) -> MemoryState {
            MemoryState {
                user_id: user_id.to_string(),
                node_id,
                stability,
                difficulty: 0.3,
                energy,
                review_count,
                last_reviewed,
                due_at: last_reviewed + Duration::days(7),
            }
        }

        /// Create a minimal Simulator for testing
        fn make_test_simulator() -> Simulator {
            use crate::SimulationConfig;
            use iqrah_core::testing::MockContentRepository;

            let content_repo = Arc::new(MockContentRepository::new());
            let config = SimulationConfig::default();
            Simulator::new(content_repo, config)
        }

        /// Create test StudentParams for drift testing
        fn make_test_params() -> crate::brain::StudentParams {
            crate::brain::StudentParams {
                drift_alpha_max: 0.20,
                drift_alpha_min: 0.02,
                drift_mastery_exponent: 2.0,
                drift_energy_floor: 0.05,
                ..Default::default()
            }
        }

        #[test]
        fn test_energy_drift_tracks_fsrs_decay() {
            // Setup: Item with S=3 days, reviewed 14 days ago → significant decay
            // R = (1 + 14/(9*3))^(-1) = (1 + 14/27)^(-1) ≈ 0.66 (not < 0.5)
            // Use S=2, t=14: R = (1 + 14/18)^(-1) ≈ 0.56
            // Use S=1.5, t=14: R = (1 + 14/13.5)^(-1) ≈ 0.49
            let repo = Arc::new(InMemoryUserRepository::new());
            let user_id = "test_user";
            let node_id = 123;

            // Initialize state: S=1.5, last reviewed 14 days ago → significant decay
            let start_time = Utc::now();
            let last_reviewed = start_time - Duration::days(14);

            let stability = 1.5;
            let initial_energy = 0.8;
            let state = make_state(
                user_id,
                node_id,
                stability,
                initial_energy,
                3,
                last_reviewed,
            );
            repo.initialize_state(state);

            // Expected R after 14 days with S=1.5
            let elapsed = 14.0;
            let expected_r = retrievability(stability, elapsed);
            // R = (1 + 14/(9*1.5))^(-1) = (1 + 14/13.5)^(-1) ≈ 0.49
            assert!(
                expected_r < 0.55,
                "R should have decayed significantly: {}",
                expected_r
            );

            // Apply drift with default params
            let simulator = make_test_simulator();

            let test_params = make_test_params();
            simulator
                .apply_daily_energy_drift(
                    user_id,
                    &[node_id],
                    &repo,
                    start_time.timestamp_millis(),
                    &test_params,
                    0, // day
                )
                .unwrap();

            // Check that energy decreased toward R
            let updated = repo.get_memory_state_sync(user_id, node_id).unwrap();
            assert!(
                updated.energy < initial_energy,
                "Energy should decrease: {}",
                updated.energy
            );
            assert!(
                updated.energy > 0.05,
                "Energy should respect floor: {}",
                updated.energy
            );

            // Energy should be between old_energy and target
            // E_new = (1-α)*E_old + α*target where target = R^γ
            let target = expected_r.powf(1.2);
            let expected_new = (1.0 - 0.15) * initial_energy + 0.15 * target;
            assert!(
                (updated.energy - expected_new).abs() < 0.02,
                "Energy: {}, Expected: {}",
                updated.energy,
                expected_new
            );
        }

        #[test]
        fn test_energy_drift_respects_floor() {
            // Very weak item: low stability, long time since review
            let repo = Arc::new(InMemoryUserRepository::new());
            let user_id = "test_user";
            let node_id = 456;

            let start_time = Utc::now();
            let last_reviewed = start_time - Duration::days(30);

            // S=2, reviewed 30 days ago → R is very low
            let state = make_state(user_id, node_id, 2.0, 0.1, 1, last_reviewed);
            repo.initialize_state(state);

            let simulator = make_test_simulator();

            let test_params = crate::brain::StudentParams {
                drift_energy_floor: 0.08,
                ..make_test_params()
            };
            simulator
                .apply_daily_energy_drift(
                    user_id,
                    &[node_id],
                    &repo,
                    start_time.timestamp_millis(),
                    &test_params,
                    0, // day
                )
                .unwrap();

            let updated = repo.get_memory_state_sync(user_id, node_id).unwrap();
            assert!(
                updated.energy >= 0.08,
                "Energy should respect floor 0.08: {}",
                updated.energy
            );
        }

        #[test]
        fn test_energy_drift_skips_unseen_items() {
            // Unseen item: review_count = 0
            let repo = Arc::new(InMemoryUserRepository::new());
            let user_id = "test_user";
            let node_id = 789;

            let state = make_state(user_id, node_id, 0.0, 0.0, 0, Utc::now());
            repo.initialize_state(state);

            let simulator = make_test_simulator();

            let test_params = make_test_params();
            simulator
                .apply_daily_energy_drift(
                    user_id,
                    &[node_id],
                    &repo,
                    Utc::now().timestamp_millis(),
                    &test_params,
                    0, // day
                )
                .unwrap();

            // Energy should remain at 0.0 for unseen items
            let updated = repo.get_memory_state_sync(user_id, node_id).unwrap();
            assert_eq!(updated.energy, 0.0, "Unseen items should not drift");
        }

        #[test]
        fn test_energy_drift_gentle_for_stable_items() {
            // Test case from spec: Item with S=30 days, 7 days idle
            // Expected R ≈ 0.75-0.80
            // Expected E drop: 5-10% from initial
            let repo = Arc::new(InMemoryUserRepository::new());
            let user_id = "test_user";
            let node_id = 111;

            let start_time = Utc::now();
            let last_reviewed = start_time - Duration::days(7);

            let initial_energy = 0.9;
            let state = make_state(user_id, node_id, 30.0, initial_energy, 5, last_reviewed);
            repo.initialize_state(state);

            // Expected R: (1 + 7/(9*30))^-1 = (1 + 7/270)^-1 ≈ 0.97
            let expected_r = retrievability(30.0, 7.0);
            assert!(
                expected_r > 0.9,
                "R should be high for stable item: {}",
                expected_r
            );

            let simulator = make_test_simulator();

            let test_params = make_test_params();
            simulator
                .apply_daily_energy_drift(
                    user_id,
                    &[node_id],
                    &repo,
                    start_time.timestamp_millis(),
                    &test_params,
                    0, // day
                )
                .unwrap();

            let updated = repo.get_memory_state_sync(user_id, node_id).unwrap();
            let energy_drop = initial_energy - updated.energy;

            // Drift should be gentle: 5-15% drop max
            assert!(
                energy_drop.abs() < 0.15,
                "Stable items should have gentle drift. Drop: {}",
                energy_drop
            );
        }

        #[test]
        fn test_energy_drift_aggressive_for_weak_items() {
            // Test case: Item with initial E=0.8, review_count=2
            // ISS v2.4 mastery AND maturity-dependent formula:
            // 1. Base drift from energy: drift_rate(E) = α_max × (1 - E^k) + α_min × E^k
            // 2. Young item protection: young_protection = 1 - 0.4 × (1 - maturity)
            //
            // With α_max=0.20, α_min=0.02, k=2, review_count=2:
            // protection = 0.8^2 = 0.64
            // base_drift = 0.20 × (1-0.64) + 0.02 × 0.64 = 0.0848
            // maturity = 2/5 = 0.4
            // young_protection = 1 - 0.4 × (1-0.4) = 1 - 0.24 = 0.76
            // final_drift = 0.0848 × 0.76 ≈ 0.0644
            // new_energy = 0.8 × (1 - 0.0644) ≈ 0.7485
            let repo = Arc::new(InMemoryUserRepository::new());
            let user_id = "test_user";
            let node_id = 222;

            let start_time = Utc::now();
            let last_reviewed = start_time - Duration::days(7);

            let initial_energy = 0.8;
            let review_count = 2;
            let state = make_state(
                user_id,
                node_id,
                5.0,
                initial_energy,
                review_count,
                last_reviewed,
            );
            repo.initialize_state(state);

            let simulator = make_test_simulator();

            let test_params = make_test_params();
            simulator
                .apply_daily_energy_drift(
                    user_id,
                    &[node_id],
                    &repo,
                    start_time.timestamp_millis(),
                    &test_params,
                    0, // day
                )
                .unwrap();

            let updated = repo.get_memory_state_sync(user_id, node_id).unwrap();

            // Compute expected using ISS v2.4 maturity-dependent formula
            // This uses compute_drift_rate_v2 under the hood
            let drift_rate = test_params.compute_drift_rate_v2(initial_energy, review_count);
            let expected_new = initial_energy * (1.0 - drift_rate);

            assert!(
                (updated.energy - expected_new).abs() < 0.001,
                "Energy should follow maturity-dependent decay. Got: {}, Expected: {} (drift: {})",
                updated.energy,
                expected_new,
                drift_rate
            );
        }
    }
}
