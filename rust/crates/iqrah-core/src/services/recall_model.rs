/// Calculates the energy change based on recall time using a shifted exponential decay model.
///
/// This model rewards fast recalls with positive energy gains and penalizes slow recalls
/// with energy losses. The relationship follows a smooth exponential decay curve centered
/// around an optimal recall time, shifted to allow negative values.
///
/// # Parameters
/// - `recall_time_ms`: The time taken to recall the word (in milliseconds)
///
/// # Formula
/// `delta = 0.16 * exp(-0.001 * (recall_time_ms - 700.0)) - 0.06`
///
/// The result is clamped to the range [-0.06, 0.1] to prevent extreme changes.
///
/// # Behavior
/// - Very fast recall (~0-500ms): +0.10 energy (maximum gain, clamped)
/// - Optimal recall (~700ms): ~0.10 energy (good performance)
/// - Moderate recall (~1500ms): ~0.04 energy (small gain)
/// - Slow recall (~3000ms+): Approaches -0.06 energy (penalty for struggling)
///
/// # Returns
/// The energy delta to add to the current energy level (clamped to [-0.06, 0.1])
pub fn calculate_energy_change(recall_time_ms: u32) -> f64 {
    const MAX_GAIN: f64 = 0.1;
    const MAX_LOSS: f64 = -0.06;
    const DECAY_RATE: f64 = 0.001;
    const OPTIMAL_TIME: f64 = 700.0;
    const SCALE: f64 = 0.16;
    const SHIFT: f64 = -0.06;

    let time = recall_time_ms as f64;
    let delta = SCALE * (-DECAY_RATE * (time - OPTIMAL_TIME)).exp() + SHIFT;

    delta.clamp(MAX_LOSS, MAX_GAIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_very_fast_recall_gives_maximum_gain() {
        // Very fast recalls should give near maximum gain
        let delta = calculate_energy_change(100);
        assert!(delta > 0.09);
        assert!(delta <= 0.1);
    }

    #[test]
    fn test_optimal_recall_time() {
        // At optimal time (700ms), should give near maximum gain
        let delta = calculate_energy_change(700);
        assert!(delta > 0.09);
        assert!(delta <= 0.1);
    }

    #[test]
    fn test_moderate_recall_gives_positive_gain() {
        // Moderate recall should still give positive gain, but less
        let delta = calculate_energy_change(1500);
        assert!(delta > 0.0);
        assert!(delta < 0.1);
    }

    #[test]
    fn test_slow_recall_gives_small_loss() {
        // Very slow recall should approach maximum loss
        let delta = calculate_energy_change(5000);
        assert!(delta < 0.0);
        assert!(delta >= -0.06);
    }

    #[test]
    fn test_extremely_slow_recall_clamped() {
        // Extremely slow recall should be clamped at max loss
        let delta = calculate_energy_change(10000);
        // Use approximate equality for floating point
        assert!((delta - (-0.06)).abs() < 0.001);
    }

    #[test]
    fn test_delta_decreases_with_time() {
        // Energy gain should decrease as recall time increases
        let delta_100 = calculate_energy_change(100);
        let delta_500 = calculate_energy_change(500);
        let delta_1000 = calculate_energy_change(1000);
        let delta_2000 = calculate_energy_change(2000);
        let delta_3000 = calculate_energy_change(3000);

        assert!(delta_100 >= delta_500);
        assert!(delta_500 > delta_1000);
        assert!(delta_1000 > delta_2000);
        assert!(delta_2000 > delta_3000);
    }

    #[test]
    fn test_crossover_point() {
        // There should be a crossover point where delta goes from positive to negative
        // Crossover occurs when: 0.16 * exp(-0.001 * (time - 700)) = 0.06
        // Solving: time ≈ 700 + 981 ≈ 1681ms
        let delta_1500 = calculate_energy_change(1500);
        let delta_2000 = calculate_energy_change(2000);

        assert!(delta_1500 > 0.0);
        assert!(delta_2000 < 0.0);
    }

    #[test]
    fn test_clamping_at_boundaries() {
        // Test that results are always within bounds
        for time_ms in [0, 100, 500, 700, 1000, 2000, 5000, 10000] {
            let delta = calculate_energy_change(time_ms);
            assert!(delta >= -0.06);
            assert!(delta <= 0.1);
        }
    }

    #[test]
    fn test_specific_time_points() {
        // Test specific time points to verify formula accuracy
        let delta_0 = calculate_energy_change(0);
        let delta_700 = calculate_energy_change(700);
        let delta_1500 = calculate_energy_change(1500);

        // At 0ms: 0.16 * exp(-0.001 * (0 - 700)) - 0.06 = 0.16 * exp(0.7) - 0.06 ≈ 0.262, clamped to 0.1
        assert_eq!(delta_0, 0.1);

        // At 700ms: 0.16 * exp(0) - 0.06 = 0.16 - 0.06 = 0.1
        assert!((delta_700 - 0.1).abs() < 0.001);

        // At 1500ms: 0.16 * exp(-0.8) - 0.06 ≈ 0.16 * 0.449 - 0.06 ≈ 0.012
        assert!((delta_1500 - 0.012).abs() < 0.005);
    }

    #[test]
    fn test_smooth_continuous_curve() {
        // The function should produce a smooth curve without discontinuities
        let mut prev_delta = calculate_energy_change(0);

        for time_ms in (100..5000).step_by(100) {
            let delta = calculate_energy_change(time_ms);
            // The change between consecutive points should be small and gradual
            let change = (delta - prev_delta).abs();
            assert!(change < 0.02, "Discontinuity detected at {}ms", time_ms);
            prev_delta = delta;
        }
    }
}
