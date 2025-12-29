//! Main simulation orchestrator.
//!
//! The Simulator runs virtual students through the real Iqrah scheduling pipeline.
//! ISS **orchestrates** the simulation; `iqrah-core` **decides** what to schedule.

use crate::baselines::{FixedSrsBaseline, GraphTopoBaseline, PageOrderBaseline, RandomBaseline};
use crate::config::{GateReason, GateTraceRow};
use crate::debug_stats::{StudentDebugAccumulator, StudentDebugSummary};
use crate::gate_trace::GateTraceCollector;
use crate::memory_health_trace::{
    compute_mean, compute_p10, compute_p50, compute_p90, MemoryHealthRow,
    MemoryHealthTraceCollector,
};

use crate::{
    InMemoryUserRepository, Scenario, SchedulerVariant, SessionGenerator, SimulationConfig,
    SimulationMetrics, StudentBrain, StudentSanityData,
};

use crate::config::compute_almost_due_window;
use crate::introduction_policy::{compute_allowance, IntroductionAllowance};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use iqrah_core::domain::{MemoryState, ReviewGrade};
use iqrah_core::initial_placement::{
    ArabicLevel, InitialPlacementService, IntakeAnswers, SurahSelfReport,
};
use iqrah_core::ports::{ContentRepository, UserRepository};
use iqrah_core::scheduler_v2::bandit::BanditOptimizer;
// M2.2: generate_session replaced by budget-enforced selection
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

        // M2.4: Per-student hysteresis state for gate (start true = allow expansion initially)
        let mut gate_expand_mode = true;
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

        // ISS v2.8: Load exercises from scenario config
        let (exercises, mut exercise_schedules) =
            crate::exercises::load_exercises(&scenario.exercises);
        if !exercises.is_empty() {
            debug!(
                "Loaded {} exercises for scenario {}",
                exercises.len(),
                scenario.name
            );
        }

        // M1.2: Create gate trace collector if enabled
        let mut gate_trace_collector = if self.config.debug_trace.enabled {
            Some(GateTraceCollector::new(
                &self.config.debug_trace,
                &scenario.name,
                &scenario.scheduler.name(),
            ))
        } else {
            None
        };

        // M3: Create memory health trace collector if enabled
        let mut memory_health_collector = if self.config.debug_trace.enabled {
            Some(MemoryHealthTraceCollector::new(
                &self.config.debug_trace,
                &scenario.name,
                &scenario.scheduler.name(),
            ))
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
                    gate_trace_collector.as_mut(),
                    &mut gate_expand_mode,
                    memory_health_collector.as_mut(),
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

            // ISS v2.8: Run scheduled exercises at end of day
            if !exercises.is_empty() {
                // Collect current memory states for exercise evaluation
                let memory_states: HashMap<i64, MemoryState> = goal_items
                    .iter()
                    .filter_map(|id| {
                        user_repo
                            .get_memory_state_sync(&user_id, *id)
                            .map(|s| (*id, s))
                    })
                    .collect();

                // Debug: Log mismatch between goal items and memory states
                if memory_states.len() != goal_items.len() {
                    debug!(
                        "Exercise memory states mismatch: {} goal items, {} states found",
                        goal_items.len(),
                        memory_states.len()
                    );
                    if !goal_items.is_empty() {
                        debug!("First goal item: {}", goal_items[0]);
                    }
                }

                self.run_scheduled_exercises(
                    scenario,
                    &mut exercise_schedules,
                    &exercises,
                    &memory_states,
                    &goal_items,
                    &brain,
                    day,
                );
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

        // M1.2: Write trace output if enabled
        if let Some(collector) = gate_trace_collector {
            if let Err(e) = collector.write_output() {
                warn!("Failed to write gate trace output: {}", e);
            }
        }

        // M3: Write memory health trace output if enabled
        if let Some(collector) = memory_health_collector {
            if let Err(e) = collector.write_output() {
                warn!("Failed to write memory health trace output: {}", e);
            }
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
        mut gate_trace: Option<&mut GateTraceCollector>,
        gate_expand_mode: &mut bool,
        memory_health: Option<&mut MemoryHealthTraceCollector>,
    ) -> Result<f64> {
        let now = start_time + Duration::days(day as i64);
        let _now_ts = now.timestamp_millis();

        // Configure Brain for Oracle mode if needed
        if matches!(baseline_state, BaselineState::OraclePerfect) {
            brain.force_perfect_recall = true;
        } else {
            brain.force_perfect_recall = false;
        }

        // 1. Generate session - branch based on scheduler variant
        let session_result = if let Some(baseline_session) = {
            // Get memory states for baseline session generation
            let memory_states = user_repo.get_all_states_for_user(user_id);
            // Use dynamic session size
            baseline_state.generate_session(goal_items, &memory_states, session_size, day)
        } {
            // Use baseline-generated session with default allowance (baseline doesn't use policy)
            let default_allowance = IntroductionAllowance {
                allowance_raw: 0,
                allowance_after_capacity: 0,
                allowance_after_workingset: 0,
                allowance_after_gate: 0,
                allowance_after_floor: 0,
                allowance_final: 0,
                gate_expand_mode: false,
                threshold_low: 0.0,
                threshold_high: 0.0,
            };
            (baseline_session, default_allowance)
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

            // M2.4: Compute introduction allowance using explicit clamp stages
            // compute_allowance() internally calls update_expand_mode() with hysteresis
            // Order: capacity → working-set (HARD) → gate → floor
            let capacity_used = 0.5; // TODO: Compute actual capacity utilization

            // M2.5: Compute effective working-set from ratio-of-goal
            let effective_max_ws_from_ratio = brain
                .params
                .compute_effective_max_working_set(goal_items.len());

            // M2.6: Compute p90 due age for backlog-aware decisions
            let now_ts = (start_time + Duration::days(day as i64)).timestamp_millis();
            let active_due_ages: Vec<f64> = states_map
                .values()
                .filter(|s| s.review_count > 0)
                .map(|s| {
                    (now_ts - s.last_reviewed.timestamp_millis()) as f64
                        / (24.0 * 60.0 * 60.0 * 1000.0)
                })
                .collect();
            let p90_due_age = if active_due_ages.is_empty() {
                0.0
            } else {
                crate::memory_health_trace::compute_p90(&active_due_ages)
            };

            // M2.6: Compute budgeted working set (caps based on review capacity)
            let max_ws_budget = brain
                .params
                .compute_budgeted_working_set(session_size, goal_items.len());
            let effective_max_ws = match max_ws_budget {
                Some(budget) => effective_max_ws_from_ratio.min(budget),
                None => effective_max_ws_from_ratio,
            };

            // M2.6: Compute backlog-aware intro floor
            let intro_floor_effective = brain.params.compute_effective_intro_floor(p90_due_age);

            let allowance = compute_allowance(
                &brain.params,
                active_count,
                effective_max_ws, // M2.5 + M2.6: min of ratio-derived and budget-derived
                cluster_energy,
                *gate_expand_mode, // current mode - will be updated internally
                capacity_used,
                intro_floor_effective, // M2.6: may be 0 if backlog severe
            );
            // Update gate_expand_mode from policy result (hysteresis applied internally)
            *gate_expand_mode = allowance.gate_expand_mode;
            let new_items_limit = allowance.allowance_final;

            // Get cluster node IDs for membership filtering
            let cluster_node_ids = learning_cluster.node_ids();

            // M2.4: Log policy decision for diagnosis
            tracing::info!(
                "M2.4 Policy day={}: expand_mode={}, raw={}, cap={}, ws={}, gate={}, final={} (active={}, ceiling={})",
                day,
                *gate_expand_mode,
                allowance.allowance_raw,
                allowance.allowance_after_capacity,
                allowance.allowance_after_workingset,
                allowance.allowance_after_gate,
                allowance.allowance_final,
                active_count,
                brain.params.max_working_set
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
            // 2. OR new items (review_count=0) - M2.2: let budget selection handle limiting
            // (removed new_items_allowed counter - budget-enforced selection does this)

            // M2.3: Count new items BEFORE cluster filter
            let _new_from_get_candidates = all_candidates
                .iter()
                .filter(|c| c.review_count == 0)
                .count();

            let candidates: Vec<_> = all_candidates
                .into_iter()
                .filter(|c| {
                    // Item is in cluster - always include
                    if cluster_node_ids.contains(&c.id) {
                        return true;
                    }
                    // Item is new (review_count=0) - include all, budget selection will limit
                    if c.review_count == 0 {
                        return true;
                    }
                    // Otherwise exclude - not in cluster and not a new item
                    false
                })
                .collect();

            // M2.3: Count new items AFTER cluster filter (should be same for new items)
            let _new_pass_cluster_filter =
                candidates.iter().filter(|c| c.review_count == 0).count();

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
            let _parent_energies: HashMap<i64, f32> = if all_parent_ids.is_empty() {
                HashMap::new()
            } else {
                user_repo
                    .get_parent_energies(user_id, &all_parent_ids)
                    .await
                    .unwrap_or_default()
            };

            // Get user profile (via bandit if enabled)
            let _profile = if scenario.enable_bandit {
                self.select_profile_via_bandit(user_id, &scenario.goal_id, user_repo, scheduler_rng)
                    .await?
            } else {
                UserProfile::balanced()
            };

            // Determine session mix configuration
            // Note: Cluster gate already filtered candidates, so we use default mix config
            // The candidate filtering controls introduction, not band composition percentages
            let _mix_config = scenario
                .session_mix
                .unwrap_or_else(|| SessionMixConfig::default());

            // === M2.2: BUDGET-ENFORCED SESSION COMPOSITION ===
            // Split candidates into due (review_count > 0) and new (review_count == 0)
            let (new_candidates, mut due_candidates): (Vec<_>, Vec<_>) =
                candidates.into_iter().partition(|c| c.review_count == 0);

            // M2.7: Sort due candidates by due_age DESC (most overdue first)
            // M2.8: Added deterministic tie-break (item_id ASC)
            // This prevents starvation where items with large due_age never get selected.
            // Due age = now - next_due_ts (negative means not yet due)
            due_candidates.sort_by(|a, b| {
                // Higher due_age (more overdue) = higher priority
                // if next_due_ts < now_ts, item is overdue -> positive due_age
                // Sort DESC: larger due_age comes first
                let due_age_a = now_ts - a.next_due_ts;
                let due_age_b = now_ts - b.next_due_ts;
                // Primary: due_age DESC, Secondary: id ASC (deterministic)
                due_age_b.cmp(&due_age_a).then(a.id.cmp(&b.id))
            });

            let due_candidates_available = due_candidates.len();
            let new_candidates_available = new_candidates.len();

            // M2.4: Use policy's allowance_final directly (floor already applied in stage 4)
            // Don't cap further with intro_budget - that would double-apply the floor
            let intro_cap = new_items_limit;

            // Compute budgets with hard reservation for intro
            let actual_intro_budget = intro_cap.min(session_size);
            let due_budget = session_size.saturating_sub(actual_intro_budget);

            // Step 1: Select new items up to intro_cap (hard reservation)
            let new_to_select = intro_cap.min(new_candidates_available);
            let selected_new: Vec<i64> = new_candidates
                .iter()
                .take(new_to_select)
                .map(|c| c.id)
                .collect();

            // Step 2: Select due items up to due_budget (now sorted by most overdue first)
            let due_to_select = due_budget.min(due_candidates_available);
            let selected_due: Vec<i64> = due_candidates
                .iter()
                .take(due_to_select)
                .map(|c| c.id)
                .collect();

            // Step 3: Compute spillover
            // If new_selected < intro_cap, spill unused intro slots to due
            let unused_intro_slots = intro_cap.saturating_sub(selected_new.len());
            let spill_to_due_count =
                unused_intro_slots.min(due_candidates_available.saturating_sub(selected_due.len()));

            // M2.2 HARD CAP: Do NOT spill unused due slots to new items
            // intro_cap is a MAXIMUM, not a target - prevents accidental over-introduction
            // If due candidates are few, we just do fewer reviews, NOT more new intros
            let spill_to_new_count = 0usize; // DISABLED: was causing intro > intro_cap

            // Step 4: Apply spillover - only to due items (already sorted by overdue priority)
            let additional_due: Vec<i64> = due_candidates
                .iter()
                .skip(selected_due.len())
                .take(spill_to_due_count)
                .map(|c| c.id)
                .collect();

            // Combine into final session (no additional_new since spill_to_new is disabled)
            let mut session: Vec<i64> = Vec::with_capacity(session_size);
            session.extend(selected_new.iter().cloned());
            session.extend(selected_due.iter().cloned());
            session.extend(additional_due.iter().cloned());

            // Final session stats for trace (no additional_new since spill_to_new disabled)
            let final_new_selected = selected_new.len(); // No spillover to new
            let final_due_selected = selected_due.len() + additional_due.len();
            let final_spill_to_due = spill_to_due_count;
            let final_spill_to_new = spill_to_new_count;

            // M2.3: Log actual session partition sizes for funnel diagnosis
            tracing::info!(
                "M2.3 SessionPartition day={}: new_candidates_actual={}, due_candidates_actual={}, new_to_select={}, selected_new={}",
                day, new_candidates_available, due_candidates_available, new_to_select, final_new_selected
            );

            debug!(
                "M2.2 BudgetSession day={}: session_size={}, intro_cap={}, due_budget={}, new_selected={}, due_selected={}, spill_due={}, spill_new={}",
                day, session_size, intro_cap, due_budget, final_new_selected, final_due_selected, final_spill_to_due, final_spill_to_new
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

            // Store selection stats for trace (via closure capture trick - use thread_local or pass)
            // For now we'll recompute these values in the trace block later

            (session, allowance)
        };

        // Destructure (session, allowance) tuple
        let (session_items, allowance) = session_result;

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

            // M1.2: Collect gate diagnostics for trace
            let mut gate_diag = crate::config::GateDiagnostics::default();
            let intro_rate = Self::compute_sustainable_intro_rate(
                &all_states,
                &brain.params,
                Some(&mut gate_diag),
            );

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

        // M1.2: Add gate trace row unconditionally (outside event block)
        // This ensures --trace works without --emit-events
        if let Some(trace) = gate_trace.as_mut() {
            // Get states and compute values needed for trace
            let states = user_repo.get_memory_states_batch_sync(user_id, goal_items);
            let active_items = states.values().filter(|s| s.review_count > 0).count();
            let all_states: Vec<_> = states.values().cloned().collect();

            // Collect gate diagnostics from compute_sustainable_intro_rate
            let mut gate_diag = crate::config::GateDiagnostics::default();
            let _intro_rate = Self::compute_sustainable_intro_rate(
                &all_states,
                &brain.params,
                Some(&mut gate_diag),
            );

            // M2.1: Correct intro tracking using introduction_order delta
            // introduced_total is authoritative (introduction_order.len())
            let introduced_total = introduction_order.len();
            let single_review_items = states.values().filter(|s| s.review_count == 1).count();

            // Get previous introduced_total from trace rows to compute delta
            let prev_introduced_total =
                trace.rows().last().map(|r| r.introduced_total).unwrap_or(0);
            let introduced_today = introduced_total.saturating_sub(prev_introduced_total);

            // M2.2: Compute candidate availability
            // New candidates = unreviewed goal items not yet introduced
            let new_candidates_available = goal_items.len().saturating_sub(introduced_total);
            // Due candidates ~ active items (approximation - actual would need session block data)
            let due_candidates_available = active_items;

            // M2.2: Budget enforcement stats
            let intro_budget = brain.params.intro_min_per_day;
            let new_items_limit_today = if gate_diag.gate_blocked {
                0
            } else {
                brain
                    .params
                    .cluster_expansion_batch_size
                    .min(brain.params.max_working_set.saturating_sub(active_items))
            };
            let intro_cap = intro_budget.min(new_items_limit_today);
            let due_budget = session_size.saturating_sub(intro_cap.min(session_size));

            // Actual selection (intro_today = new_selected with M2.2 enforcement)
            let new_selected = introduced_today;
            let due_selected = session_len.saturating_sub(new_selected);

            // Spillover computation
            let spill_to_due = intro_cap
                .saturating_sub(new_selected)
                .min(due_candidates_available.saturating_sub(due_selected));
            let spill_to_new = due_budget
                .saturating_sub(due_selected)
                .min(new_candidates_available.saturating_sub(new_selected));

            let capacity_budget = brain.params.session_capacity as usize;
            let budget_delta = capacity_budget as i32 - session_len as i32;

            // M2.5: Compute effective max working-set for trace
            let effective_max_ws_for_trace = brain
                .params
                .compute_effective_max_working_set(goal_items.len());

            // M2.6: Compute p90 due age for backlog-aware decisions
            let now_ts = (start_time + Duration::days(day as i64)).timestamp_millis();
            let active_due_ages: Vec<f64> = states
                .values()
                .filter(|s| s.review_count > 0)
                .map(|s| {
                    (now_ts - s.last_reviewed.timestamp_millis()) as f64
                        / (24.0 * 60.0 * 60.0 * 1000.0)
                })
                .collect();
            let p90_due_age = if active_due_ages.is_empty() {
                0.0
            } else {
                compute_p90(&active_due_ages)
            };

            // M2.6: Compute budgeted working set and backlog-aware floor
            let max_ws_budget = brain
                .params
                .compute_budgeted_working_set(session_len, goal_items.len());
            let backlog_severe = brain.params.is_backlog_severe(p90_due_age);
            let intro_floor_effective = brain.params.compute_effective_intro_floor(p90_due_age);

            // M2.7: Compute overdue candidate stats
            // overdue = items with due_age > 0 (last_reviewed is in the past beyond their interval)
            let overdue_candidates_count = active_due_ages.iter().filter(|&&age| age > 0.0).count();
            // Since we don't have detailed selecteditem tracking here, approximate:
            // overdue_selected ≈ min(session_len, overdue_candidates_count) when sorted by due_age DESC
            let overdue_selected_count = session_len.min(overdue_candidates_count);
            // max_due_age_selected = max due age in the session (first item after sort)
            let max_due_age_selected = active_due_ages.iter().cloned().fold(0.0_f64, f64::max);

            trace.add_row(GateTraceRow {
                day,
                due_reviews: session_len,
                actual_reviews: session_len,
                capacity_budget,
                budget_delta,
                introduced_today,
                introduced_total,
                single_review_items,
                new_items_limit_today,
                total_active: active_items,
                max_new_gate_param: brain.params.max_working_set,
                cluster_energy: gate_diag.cluster_energy,
                gate_blocked: gate_diag.gate_blocked,
                gate_reason: gate_diag.gate_reason,
                threshold: gate_diag.threshold,
                working_set_factor: gate_diag.working_set_factor,
                capacity_used: gate_diag.capacity_used,
                session_size,
                due_budget,
                intro_budget,
                due_selected,
                new_selected,
                due_candidates_available,
                new_candidates_available,
                intro_cap,
                spill_to_due,
                spill_to_new,
                // M2.3: Candidate funnel diagnostics
                goal_total: goal_items.len(),
                unintroduced_total: goal_items.len().saturating_sub(introduced_total),
                new_from_get_candidates: goal_items.len().saturating_sub(introduced_total), // ISS returns all
                new_pass_cluster_filter: new_candidates_available, // After cluster filter
                new_candidates_in_session: new_candidates_available, // Same as partition
                // M2.4: Introduction policy explicit clamp stages
                gate_expand_mode: allowance.gate_expand_mode,
                threshold_low: allowance.threshold_low,
                threshold_high: allowance.threshold_high,
                allowance_raw: allowance.allowance_raw,
                allowance_after_capacity: allowance.allowance_after_capacity,
                allowance_after_workingset: allowance.allowance_after_workingset,
                allowance_after_gate: allowance.allowance_after_gate,
                allowance_final: allowance.allowance_final,
                intro_min_per_day: brain.params.intro_min_per_day,
                intro_bootstrap_until_active: brain.params.intro_bootstrap_until_active,
                max_working_set_effective: effective_max_ws_for_trace,
                // M2.6: Backlog-aware working set + floor
                max_ws_budget,
                target_reviews_per_active: brain.params.target_reviews_per_active,
                intro_floor_effective,
                p90_due_age_days_trace: p90_due_age,
                max_p90_due_age_days: brain.params.max_p90_due_age_days,
                backlog_severe,
                // M2.7: Overdue fairness diagnostics
                overdue_candidates_count,
                overdue_selected_count,
                max_due_age_selected,
            });
        }

        // M3: Compute and add memory health trace row
        if let Some(mh) = memory_health {
            // Get states for aggregate computation
            let states = user_repo.get_memory_states_batch_sync(user_id, goal_items);
            let now_ts = (start_time + Duration::days(day as i64)).timestamp_millis();

            // Collect energy values from active items
            let active_states: Vec<_> = states.values().filter(|s| s.review_count > 0).collect();

            if !active_states.is_empty() {
                let energies: Vec<f64> = active_states.iter().map(|s| s.energy.into()).collect();
                let stabilities: Vec<f64> =
                    active_states.iter().map(|s| s.stability.into()).collect();

                // Retrievability at today (R(S, 0) = 1.0 for just-reviewed items)
                // For items not reviewed today, compute R using FSRS formula
                let retrievabilities: Vec<f64> = active_states
                    .iter()
                    .map(|s| {
                        let elapsed_days = (now_ts - s.last_reviewed.timestamp_millis()) as f64
                            / (24.0 * 60.0 * 60.0 * 1000.0);
                        let eff_stab = if s.stability < 0.1 {
                            1.0
                        } else {
                            s.stability as f64
                        };
                        (1.0 + elapsed_days / (9.0 * eff_stab)).powf(-1.0)
                    })
                    .collect();

                // Due age: days since last review (for backlog severity)
                let due_ages: Vec<f64> = active_states
                    .iter()
                    .map(|s| {
                        (now_ts - s.last_reviewed.timestamp_millis()) as f64
                            / (24.0 * 60.0 * 60.0 * 1000.0)
                    })
                    .collect();

                // Items reviewed today: check if last_reviewed is within today
                let items_reviewed_today = active_states
                    .iter()
                    .filter(|s| {
                        let days_since = (now_ts - s.last_reviewed.timestamp_millis()) as f64
                            / (24.0 * 60.0 * 60.0 * 1000.0);
                        days_since < 1.0
                    })
                    .count();

                // Mean p_recall for items reviewed today
                let reviewed_today_recalls: Vec<f64> = active_states
                    .iter()
                    .filter(|s| {
                        let days_since = (now_ts - s.last_reviewed.timestamp_millis()) as f64
                            / (24.0 * 60.0 * 60.0 * 1000.0);
                        days_since < 1.0
                    })
                    .map(|s| {
                        // For just-reviewed items, use energy as proxy for recall
                        s.energy as f64
                    })
                    .collect();

                let total_active = active_states.len();

                // M2.8: Compute at-risk metrics (R-based backlog severity)
                const R_RISK_THRESHOLD: f64 = 0.80;
                let at_risk_count = retrievabilities
                    .iter()
                    .filter(|&&r| r < R_RISK_THRESHOLD)
                    .count();
                let at_risk_ratio = if total_active > 0 {
                    at_risk_count as f64 / total_active as f64
                } else {
                    0.0
                };
                let p10_r_today = compute_p10(&retrievabilities);

                // p90 due age among at-risk items only
                let at_risk_due_ages: Vec<f64> = active_states
                    .iter()
                    .zip(retrievabilities.iter())
                    .filter(|(_, &r)| r < R_RISK_THRESHOLD)
                    .map(|(s, _)| {
                        (now_ts - s.last_reviewed.timestamp_millis()) as f64
                            / (24.0 * 60.0 * 60.0 * 1000.0)
                    })
                    .collect();
                let p90_due_age_at_risk = compute_p90(&at_risk_due_ages);

                mh.add_row(MemoryHealthRow {
                    day,
                    mean_energy: compute_mean(&energies),
                    p10_energy: compute_p10(&energies),
                    mean_stability: compute_mean(&stabilities),
                    p10_stability: compute_p10(&stabilities),
                    mean_retrievability_today: compute_mean(&retrievabilities),
                    mean_p_recall_reviewed_today: if reviewed_today_recalls.is_empty() {
                        0.0
                    } else {
                        compute_mean(&reviewed_today_recalls)
                    },
                    mean_reviews_per_active_item_today: if total_active > 0 {
                        items_reviewed_today as f64 / total_active as f64
                    } else {
                        0.0
                    },
                    p50_due_age_days: compute_p50(&due_ages),
                    p90_due_age_days: compute_p90(&due_ages),
                    total_active,
                    items_reviewed_today,
                    items_due_today: 0, // TODO: compute from FSRS intervals
                    // M2.8: At-risk backlog metrics
                    at_risk_count,
                    at_risk_ratio,
                    p10_r_today,
                    p90_due_age_at_risk,
                    // M2.9: Weaker at-risk threshold (R < 0.90)
                    at_risk_count_0_9: retrievabilities.iter().filter(|&&r| r < 0.90).count(),
                    at_risk_ratio_0_9: if total_active > 0 {
                        retrievabilities.iter().filter(|&&r| r < 0.90).count() as f64
                            / total_active as f64
                    } else {
                        0.0
                    },
                });
            }
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
    /// M1.2: Optionally fills `out_diag` with gate decision diagnostics for trace output.
    fn compute_sustainable_intro_rate(
        items: &[MemoryState],
        student_params: &crate::brain::StudentParams,
        out_diag: Option<&mut crate::config::GateDiagnostics>,
    ) -> f64 {
        let active_count = items.iter().filter(|i| i.review_count > 0).count();

        // Compute cluster energy upfront (needed for diagnostics)
        let cluster_energy = Self::compute_cluster_energy(items);
        let threshold = student_params.cluster_stability_threshold;
        let max_working_set = student_params.max_working_set;

        // Helper to fill diagnostics
        macro_rules! fill_diag {
            ($gate_blocked:expr, $gate_reason:expr, $wsf:expr, $cap_used:expr) => {
                if let Some(diag) = out_diag {
                    diag.cluster_energy = cluster_energy;
                    diag.threshold = threshold;
                    diag.working_set_factor = $wsf;
                    diag.capacity_used = $cap_used;
                    diag.gate_blocked = $gate_blocked;
                    diag.gate_reason = $gate_reason;
                    diag.active_count = active_count;
                    diag.max_working_set = max_working_set;
                }
            };
        }

        if active_count == 0 {
            // No maintenance burden yet, introduce freely but conservatively
            fill_diag!(false, GateReason::None, 1.0, 0.0);
            return student_params.session_capacity * 0.8;
        }

        // =========================================================================
        // ISS v2.6: Working Set Limit Check
        // =========================================================================
        if active_count >= student_params.max_working_set {
            fill_diag!(true, GateReason::WorkingSetFull, 0.0, 1.0);
            return 0.0; // At limit - consolidate existing items
        }

        // Compute working set factor: 1.0 when plenty of room, 0.0 when at limit
        let working_set_factor =
            Self::compute_working_set_factor(active_count, student_params.max_working_set);

        // =========================================================================
        // ISS v2.9: Bootstrap exception based on intro_bootstrap_until_active
        // =========================================================================
        let bootstrap_threshold = if student_params.intro_bootstrap_until_active > 0 {
            student_params.intro_bootstrap_until_active
        } else {
            10 // Default original value
        };
        let skip_cluster_gate = active_count < bootstrap_threshold;

        // Compute capacity-related values upfront (needed for override and rate)
        let avg_review_interval = Self::compute_actual_avg_interval(items);
        let maintenance_burden = active_count as f64 / avg_review_interval;
        let capacity_used = maintenance_burden / student_params.session_capacity;

        // If cluster energy is below threshold and not in bootstrap phase, check for override
        if !skip_cluster_gate && cluster_energy < student_params.cluster_stability_threshold {
            // =========================================================================
            // ISS v2.9: Budget-based intro override
            // =========================================================================
            if student_params.intro_override_enabled {
                if capacity_used < student_params.intro_slack_ratio {
                    // Override: allow introductions despite weak cluster
                    fill_diag!(false, GateReason::None, working_set_factor, capacity_used);
                    let override_rate = student_params.max_new_items_per_day as f64;
                    return override_rate.max(student_params.intro_min_per_day as f64);
                }
            }

            // =========================================================================
            // ISS v2.9: Floor mechanism
            // =========================================================================
            if student_params.intro_min_per_day > 0 {
                fill_diag!(
                    true,
                    GateReason::ClusterWeak,
                    working_set_factor,
                    capacity_used
                );
                return student_params.intro_min_per_day as f64;
            }

            fill_diag!(
                true,
                GateReason::ClusterWeak,
                working_set_factor,
                capacity_used
            );
            return 0.0; // Cluster weak, no override, no floor - consolidate
        }

        // =========================================================================
        // Original capacity-based logic (ISS v2.3/v2.4)
        // =========================================================================

        // If heavily over capacity (>110%), return minimum.
        if capacity_used >= 1.1 {
            fill_diag!(
                true,
                GateReason::CapacityExceeded,
                working_set_factor,
                capacity_used
            );
            return student_params.intro_min_per_day.max(1) as f64;
        }

        // Apply headroom reserve (keep buffer for urgent items)
        let available_capacity = (1.0 - capacity_used) * (1.0 - student_params.headroom_reserve);

        // Compute review burden per new item
        let review_burden_per_new =
            student_params.reviews_to_stability / student_params.days_to_stability;

        let raw_intro_rate = available_capacity / review_burden_per_new;

        // Calculate current coverage for adaptive damping (mean Retrievability)
        let introduced_items: Vec<_> = items.iter().filter(|i| i.review_count > 0).collect();
        let current_coverage = if !introduced_items.is_empty() {
            introduced_items.iter().map(|i| i.energy).sum::<f64>() / introduced_items.len() as f64
        } else {
            0.0
        };

        // Apply adaptive sigmoid damping
        let capacity_margin = 1.0 - capacity_used;
        let damping = Self::adaptive_sigmoid_damping(capacity_margin, current_coverage);

        // Apply both damping and working set factor
        let mut rate =
            raw_intro_rate * damping * student_params.session_capacity * working_set_factor;

        // ISS v2.9: Apply max_new_items_per_day cap
        if student_params.intro_override_enabled || student_params.max_new_items_per_day > 0 {
            rate = rate.min(student_params.max_new_items_per_day as f64);
        }

        // ISS v2.9: Apply intro_min_per_day floor
        rate = rate.max(student_params.intro_min_per_day as f64);

        // Ensure at least 1 if we have capacity and damping didn't kill it
        if rate < 1.0 && capacity_margin > 0.05 && working_set_factor > 0.0 {
            fill_diag!(false, GateReason::None, working_set_factor, capacity_used);
            return 1.0_f64.max(student_params.intro_min_per_day as f64);
        }

        // Normal path - no gate blocked
        fill_diag!(false, GateReason::None, working_set_factor, capacity_used);
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
        // Catch errors (e.g., missing table) and fall through to parsing
        let _items = match self.content_repo.get_nodes_for_goal(goal_id).await {
            Ok(items) if !items.is_empty() => return Ok(items),
            Ok(_) => (), // Empty result, fall through
            Err(e) => {
                debug!("node_goals query failed (falling back to parsing): {}", e);
                // Fall through to parsing
            }
        };

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
        use iqrah_core::domain::node_id as nid;

        let mut node_ids = Vec::new();

        for surah_num in start_surah..=end_surah {
            let verses = self.content_repo.get_verses_for_chapter(surah_num).await?;

            for verse in verses {
                // Use encoded verse IDs that exercises can decode with nid::decode_verse
                // The DB stores Knowledge node IDs (TYPE=5) which have a different format
                node_ids.push(nid::encode_verse(
                    verse.chapter_number as u8,
                    verse.verse_number as u16,
                ));
            }
        }

        Ok(node_ids)
    }

    // =========================================================================
    // ISS v2.8: Exercise Framework Integration
    // =========================================================================

    /// Run scheduled exercises for the current day.
    ///
    /// This is called at the end of each simulated day to evaluate the student
    /// using axis-appropriate exercises (memory: trials-based, translation: accuracy).
    fn run_scheduled_exercises(
        &self,
        _scenario: &Scenario,
        schedules: &mut HashMap<String, crate::exercises::ExerciseSchedule>,
        exercises: &[Box<dyn crate::exercises::Exercise>],
        memory_states: &HashMap<i64, MemoryState>,
        goal_items: &[i64],
        brain: &StudentBrain,
        day: u32,
    ) {
        use crate::events::SimulationEvent;

        // Find exercises due today
        let due_exercises: Vec<(String, usize)> = schedules
            .iter()
            .enumerate()
            .filter(|(_, (_, schedule))| schedule.is_due(day))
            .map(|(idx, (name, _))| (name.clone(), idx))
            .collect();

        if due_exercises.is_empty() {
            return;
        }

        // Execute each due exercise
        for (name, idx) in due_exercises {
            if idx >= exercises.len() {
                continue;
            }

            let exercise = &exercises[idx];

            // Evaluate exercise
            match exercise.evaluate(memory_states, goal_items, brain, day) {
                Ok(result) => {
                    let metadata = exercise.metadata();

                    // Emit ExerciseEvaluation event
                    self.event_sender
                        .record(SimulationEvent::ExerciseEvaluation {
                            day,
                            exercise_name: name.clone(),
                            exercise_axis: metadata.axis,
                            score: result.score,
                            grade: format!("{:?}", result.grade),
                            items_tested: match &result.details {
                                crate::exercises::ExerciseDetails::Memory {
                                    items_tested, ..
                                } => *items_tested,
                                crate::exercises::ExerciseDetails::Translation {
                                    items_tested,
                                    ..
                                } => *items_tested,
                            },
                            summary: result.summary.clone(),
                        });

                    debug!(
                        "Exercise {} completed: score={:.2}, grade={:?}",
                        name, result.score, result.grade
                    );
                }
                Err(e) => {
                    warn!("Exercise {} failed on day {}: {}", name, day, e);
                }
            }

            // Update schedule
            if let Some(schedule) = schedules.get_mut(&name) {
                schedule.record_run(day);
            }
        }
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
