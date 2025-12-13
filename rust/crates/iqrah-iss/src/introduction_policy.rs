//! Introduction Policy Engine (M2.4)
//!
//! Implements explicit 4-stage clamp order for new item introduction:
//! 1. Capacity throttle
//! 2. Working-set clamp (HARD - cannot be bypassed)
//! 3. Gate clamp (based on cluster energy with hysteresis)
//! 4. Floor (intro_min_per_day, but cannot exceed working-set decision)

use crate::brain::StudentParams;

/// Result of introduction allowance computation with all clamp stages visible.
#[derive(Debug, Clone)]
pub struct IntroductionAllowance {
    /// Stage 0: Base batch size (raw allowance)
    pub allowance_raw: usize,
    /// Stage 1: After capacity throttle (may reduce if capacity_used is high)
    pub allowance_after_capacity: usize,
    /// Stage 2: After working-set clamp (HARD: 0 if at max working set)
    pub allowance_after_workingset: usize,
    /// Stage 3: After gate clamp (0 if gate_expand_mode is false)
    pub allowance_after_gate: usize,
    /// Stage 4: After floor (max with intro_min, but only if working-set has slack)
    pub allowance_after_floor: usize,
    /// Final allowance (min of stage 4 and stage 2 for safety)
    pub allowance_final: usize,

    // Hysteresis state
    /// Current gate expand mode (true = allow expansion, false = consolidate)
    pub gate_expand_mode: bool,
    /// Threshold low boundary = threshold - hysteresis
    pub threshold_low: f64,
    /// Threshold high boundary = threshold + hysteresis
    pub threshold_high: f64,
}

/// Update gate_expand_mode based on hysteresis.
///
/// The mode only flips when energy crosses the hysteresis boundaries:
/// - If energy >= threshold_high: set mode to true (allow expansion)
/// - If energy <= threshold_low: set mode to false (consolidate)
/// - Otherwise: keep previous mode (prevents flapping)
pub fn update_expand_mode(
    current_mode: bool,
    cluster_energy: f64,
    threshold: f64,
    hysteresis: f64,
) -> bool {
    // Clamp hysteresis to valid range
    let hysteresis = hysteresis.max(0.0);
    let threshold_low = (threshold - hysteresis).max(0.0);
    let threshold_high = (threshold + hysteresis).min(1.0);

    if cluster_energy >= threshold_high {
        true // Energy high enough, allow expansion
    } else if cluster_energy <= threshold_low {
        false // Energy too low, consolidate
    } else {
        current_mode // In dead zone, keep previous state
    }
}

/// Compute the introduction allowance using explicit 4-stage clamp order.
///
/// # Clamp Order (fixed, documented)
/// 1. **Capacity throttle**: raw → reduced by capacity_used
/// 2. **Working-set clamp (HARD)**: if active >= max → 0 (always, never bypassed)
/// 3. **Gate clamp**: if !gate_expand_mode → 0 (consolidate mode)
/// 4. **Floor**: apply intro_min_per_day (but CANNOT exceed working-set decision)
///
/// This function internally calls `update_expand_mode()` to update the gate mode
/// based on hysteresis, ensuring no stale mode is passed by the caller.
///
/// This ensures:
/// - Working-set full is always a hard stop
/// - Floor only applies when there's slack in working set
/// - Hysteresis prevents threshold flapping
/// - No stale gate mode can be passed (computed internally)
/// - M2.6: Floor can be dynamically disabled when backlog is severe
pub fn compute_allowance(
    params: &StudentParams,
    active_count: usize,
    effective_max_working_set: usize, // M2.5: derived from ratio-of-goal
    cluster_energy: f64,
    current_mode: bool,
    capacity_used: f64,
    intro_floor_effective: usize, // M2.6: may be 0 if backlog severe
) -> IntroductionAllowance {
    let hysteresis = params.cluster_gate_hysteresis.max(0.0);
    let threshold_low = (params.cluster_stability_threshold - hysteresis).max(0.0);
    let threshold_high = (params.cluster_stability_threshold + hysteresis).min(1.0);

    // Compute updated gate mode using hysteresis (no stale mode trap)
    let gate_expand_mode = update_expand_mode(
        current_mode,
        cluster_energy,
        params.cluster_stability_threshold,
        hysteresis,
    );

    // Stage 0: Raw batch size (base allowance)
    let allowance_raw = params.cluster_expansion_batch_size;

    // Stage 1: Capacity throttle
    // If heavily over capacity (>110%), reduce to floor (respects floor=0)
    // If moderately over capacity (>90%), reduce by half
    // M2.6: Use intro_floor_effective instead of raw intro_min_per_day
    let allowance_after_capacity = if capacity_used >= 1.1 {
        // Over capacity - reduce to intro_floor_effective (could be 0 if backlog severe)
        intro_floor_effective
    } else if capacity_used >= 0.9 {
        // Moderate capacity - reduce by half, but don't go below floor
        (allowance_raw / 2).max(intro_floor_effective)
    } else {
        allowance_raw
    };

    // Stage 2: Working-set clamp (HARD - cannot be bypassed by floor)
    // Uses effective_max_working_set which may be derived from ratio-of-goal (M2.5)
    let allowance_after_workingset = if active_count >= effective_max_working_set {
        0 // At working set limit - HARD STOP, no exceptions
    } else {
        // Within limit - respect remaining capacity
        let remaining_capacity = effective_max_working_set - active_count;
        allowance_after_capacity.min(remaining_capacity)
    };

    // Stage 3: Gate clamp (based on cluster energy)
    let allowance_after_gate = if gate_expand_mode {
        // Cluster is stable, allow full allotment
        allowance_after_workingset
    } else {
        // Cluster is weak, consolidate (reduce to 0)
        0
    };

    // Stage 4: Floor (intro_floor_effective)
    // Floor can ONLY apply if working-set has slack (stage 2 > 0)
    // M2.6: intro_floor_effective may be 0 if backlog is severe
    let allowance_after_floor = if allowance_after_workingset > 0 {
        // Working set has slack, apply floor
        allowance_after_gate.max(intro_floor_effective)
    } else {
        // Working set full, floor cannot override
        0
    };

    // Final: Safety check - never exceed working-set decision
    let allowance_final = allowance_after_floor.min(allowance_after_workingset);

    IntroductionAllowance {
        allowance_raw,
        allowance_after_capacity,
        allowance_after_workingset,
        allowance_after_gate,
        allowance_after_floor,
        allowance_final,
        gate_expand_mode,
        threshold_low,
        threshold_high,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_params(
        max_working_set: usize,
        intro_min_per_day: usize,
        batch_size: usize,
        threshold: f64,
        hysteresis: f64,
    ) -> StudentParams {
        let mut params = StudentParams::default();
        params.max_working_set = max_working_set;
        params.intro_min_per_day = intro_min_per_day;
        params.cluster_expansion_batch_size = batch_size;
        params.cluster_stability_threshold = threshold;
        params.cluster_gate_hysteresis = hysteresis;
        params
    }

    #[test]
    fn test_hysteresis_crossing_high_boundary() {
        // Start false, energy sequence crosses threshold_high (0.11)
        // threshold=0.10, hysteresis=0.01 → low=0.09, high=0.11
        let threshold = 0.10;
        let hysteresis = 0.01;

        let mut mode = false;

        // 0.095: below high, stays false
        mode = update_expand_mode(mode, 0.095, threshold, hysteresis);
        assert!(!mode, "0.095 should not flip (below 0.11)");

        // 0.112: above high, flips true
        mode = update_expand_mode(mode, 0.112, threshold, hysteresis);
        assert!(mode, "0.112 should flip to true (above 0.11)");

        // 0.105: in dead zone, stays true
        mode = update_expand_mode(mode, 0.105, threshold, hysteresis);
        assert!(mode, "0.105 should stay true (dead zone)");

        // 0.088: below low (0.09), flips false
        mode = update_expand_mode(mode, 0.088, threshold, hysteresis);
        assert!(!mode, "0.088 should flip to false (below 0.09)");

        // 0.095: in dead zone, stays false
        mode = update_expand_mode(mode, 0.095, threshold, hysteresis);
        assert!(!mode, "0.095 should stay false (dead zone)");
    }

    #[test]
    fn test_hysteresis_starting_true() {
        // Start true, confirm symmetric behavior
        let threshold = 0.10;
        let hysteresis = 0.01;

        let mut mode = true;

        // 0.105: in dead zone, stays true
        mode = update_expand_mode(mode, 0.105, threshold, hysteresis);
        assert!(mode, "0.105 should stay true (dead zone)");

        // 0.088: below low, flips false
        mode = update_expand_mode(mode, 0.088, threshold, hysteresis);
        assert!(!mode, "0.088 should flip to false");

        // 0.095: in dead zone, stays false
        mode = update_expand_mode(mode, 0.095, threshold, hysteresis);
        assert!(!mode, "0.095 should stay false (dead zone)");

        // 0.115: above high, flips true
        mode = update_expand_mode(mode, 0.115, threshold, hysteresis);
        assert!(mode, "0.115 should flip to true");
    }

    #[test]
    fn test_working_set_hard_stop() {
        // At max working set, allowance_final must be 0 even if intro_floor > 0
        let params = make_test_params(
            100,  // max_working_set
            5,    // intro_min_per_day (floor)
            15,   // batch_size
            0.10, // threshold
            0.01, // hysteresis
        );

        let allowance = compute_allowance(
            &params, 100,  // active_count = max (at limit)
            100,  // effective_max_working_set (same as param for test)
            0.15, // high energy (would normally allow)
            true, // gate_expand_mode = true
            0.5,  // low capacity_used
            5,    // intro_floor_effective (same as intro_min_per_day)
        );

        assert_eq!(
            allowance.allowance_after_workingset, 0,
            "Stage 2 should be 0 at max working set"
        );
        assert_eq!(
            allowance.allowance_final, 0,
            "Final allowance must be 0 at max working set (floor cannot override)"
        );
    }

    #[test]
    fn test_floor_applies_when_gate_blocks_but_workingset_has_slack() {
        // Gate says consolidate (expand_mode=false), but working set has slack
        // Floor should apply
        let params = make_test_params(
            100, // max_working_set
            5,   // intro_min_per_day (floor)
            15,  // batch_size
            0.10, 0.01,
        );

        let allowance = compute_allowance(
            &params, 50,    // active_count < max (has slack)
            100,   // effective_max_working_set
            0.05,  // low energy
            false, // gate_expand_mode = false (consolidate)
            0.5, 5, // intro_floor_effective (same as intro_min_per_day)
        );

        assert!(
            allowance.allowance_after_workingset > 0,
            "Stage 2 should be > 0 with slack"
        );
        assert_eq!(
            allowance.allowance_after_gate, 0,
            "Stage 3 should be 0 when consolidating"
        );
        assert_eq!(
            allowance.allowance_after_floor, 5,
            "Stage 4 should apply floor since working set has slack"
        );
        assert_eq!(allowance.allowance_final, 5, "Final should be floor (5)");
    }

    #[test]
    fn test_floor_cannot_exceed_working_set_remaining() {
        // Floor > remaining capacity in working set
        let params = make_test_params(
            100, // max_working_set
            10,  // intro_min_per_day (floor = 10)
            15,  // batch_size
            0.10, 0.01,
        );

        let allowance = compute_allowance(
            &params, 97,    // active = 97, remaining = 3
            100,   // effective_max_working_set
            0.05,  // low energy
            false, // consolidate
            0.5, 10, // intro_floor_effective (same as intro_min_per_day)
        );

        assert_eq!(
            allowance.allowance_after_workingset, 3,
            "Stage 2 should be 3 (100 - 97)"
        );
        assert_eq!(
            allowance.allowance_final, 3,
            "Final should be 3 (floor capped by working set remaining)"
        );
    }
}
