//! Statistical utilities for ISS v0.5.
//!
//! This module provides:
//! - MetricStats: aggregated metrics with 95% confidence intervals
//! - Timeline aggregation for learning curves
//! - Welch's t-test for statistical significance
//! - Difficulty bucket analysis

use serde::Serialize;
use std::f64::consts::PI;

// ============================================================================
// MetricStats with Confidence Intervals
// ============================================================================

/// Aggregated statistics for a single metric with confidence interval.
///
/// The 95% CI uses normal approximation: mean ± 1.96 * std / sqrt(n)
#[derive(Debug, Clone, Serialize)]
pub struct MetricStats {
    /// Sample mean
    pub mean: f64,
    /// Sample standard deviation
    pub std: f64,
    /// Lower bound of 95% confidence interval
    pub ci_lower: f64,
    /// Upper bound of 95% confidence interval
    pub ci_upper: f64,
    /// Sample size
    pub n: usize,
}

impl MetricStats {
    /// Compute statistics from a vector of values.
    pub fn from_values(values: &[f64]) -> Self {
        let n = values.len();
        if n == 0 {
            return Self::empty();
        }

        let n_f64 = n as f64;
        let mean = values.iter().sum::<f64>() / n_f64;
        let std = if n > 1 {
            let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        // 95% CI using z = 1.96
        let margin = if n > 1 {
            1.96 * std / (n_f64).sqrt()
        } else {
            0.0
        };

        Self {
            mean,
            std,
            ci_lower: mean - margin,
            ci_upper: mean + margin,
            n,
        }
    }

    /// Create empty stats.
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            std: 0.0,
            ci_lower: 0.0,
            ci_upper: 0.0,
            n: 0,
        }
    }
}

// ============================================================================
// Timeline / Learning Curve Aggregation
// ============================================================================

/// A single point on the learning curve timeline.
#[derive(Debug, Clone, Serialize)]
pub struct TimelinePoint {
    /// Day index (0-indexed)
    pub day: u32,
    /// Mean coverage percentage across students
    pub coverage_mean: f64,
    /// Mean number of items mastered
    pub items_mastered_mean: f64,
    /// Mean cumulative reviews
    pub reviews_mean: f64,
}

/// Per-student daily snapshot for timeline collection.
#[derive(Debug, Clone)]
pub struct StudentDailyPoint {
    pub day: u32,
    pub coverage_pct: f64,
    pub items_mastered: usize,
    pub cumulative_reviews: u32,
}

/// Aggregate per-student daily points into timeline.
pub fn aggregate_timeline(
    student_timelines: &[Vec<StudentDailyPoint>],
    target_days: u32,
) -> Vec<TimelinePoint> {
    if student_timelines.is_empty() {
        return vec![];
    }

    let n_students = student_timelines.len() as f64;
    let mut result = Vec::with_capacity(target_days as usize);

    for day in 0..target_days {
        let mut coverage_sum = 0.0;
        let mut mastered_sum = 0.0;
        let mut reviews_sum = 0.0;
        let mut count = 0usize;

        for student in student_timelines {
            if let Some(point) = student.iter().find(|p| p.day == day) {
                coverage_sum += point.coverage_pct;
                mastered_sum += point.items_mastered as f64;
                reviews_sum += point.cumulative_reviews as f64;
                count += 1;
            }
        }

        if count > 0 {
            result.push(TimelinePoint {
                day,
                coverage_mean: coverage_sum / count as f64,
                items_mastered_mean: mastered_sum / count as f64,
                reviews_mean: reviews_sum / count as f64,
            });
        }
    }

    result
}

// ============================================================================
// Statistical Significance Tests
// ============================================================================

/// Result of a statistical significance test between two variants.
#[derive(Debug, Clone, Serialize)]
pub struct SignificanceResult {
    /// First variant name
    pub variant_a: String,
    /// Second variant name
    pub variant_b: String,
    /// Metric being compared
    pub metric: String,
    /// p-value from the test
    pub p_value: f64,
    /// Whether a > b (positive effect direction)
    pub a_greater: bool,
}

/// Perform Welch's t-test for two independent samples.
///
/// Returns the p-value (two-tailed).
/// Uses Welch-Satterthwaite approximation for degrees of freedom.
pub fn welchs_t_test(a: &[f64], b: &[f64]) -> f64 {
    if a.len() < 2 || b.len() < 2 {
        return 1.0; // Not enough data
    }

    let n1 = a.len() as f64;
    let n2 = b.len() as f64;

    let mean1 = a.iter().sum::<f64>() / n1;
    let mean2 = b.iter().sum::<f64>() / n2;

    let var1 = a.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let var2 = b.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);

    // Avoid division by zero
    if var1 == 0.0 && var2 == 0.0 {
        // If both variances are zero, means are either equal or clearly different
        return if (mean1 - mean2).abs() < 1e-10 {
            1.0
        } else {
            0.0
        };
    }

    let s1_n = var1 / n1;
    let s2_n = var2 / n2;

    // t-statistic
    let t = (mean1 - mean2) / (s1_n + s2_n).sqrt();

    // Welch-Satterthwaite degrees of freedom
    let df = (s1_n + s2_n).powi(2) / (s1_n.powi(2) / (n1 - 1.0) + s2_n.powi(2) / (n2 - 1.0));

    // Two-tailed p-value using Student's t CDF approximation
    t_cdf_two_tailed(t.abs(), df)
}

/// Approximate two-tailed p-value from t-distribution.
/// Uses the regularized incomplete beta function approximation.
fn t_cdf_two_tailed(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return 1.0;
    }

    // Use approximation: for large df, t-dist ≈ normal
    if df > 100.0 {
        // Normal approximation
        return 2.0 * (1.0 - normal_cdf(t));
    }

    // Beta function approximation for t-distribution CDF
    // F(t) = 1 - 0.5 * I_{x}(df/2, 0.5) where x = df/(df + t²)
    let x = df / (df + t * t);
    let p = regularized_incomplete_beta(df / 2.0, 0.5, x);

    p.clamp(0.0, 1.0)
}

/// Standard normal CDF approximation (Abramowitz and Stegun).
fn normal_cdf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs() / 2.0_f64.sqrt();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    0.5 * (1.0 + sign * y)
}

/// Regularized incomplete beta function approximation.
/// This is a simple approximation suitable for small to moderate values.
fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction approximation
    // For t-distribution with a=df/2, b=0.5, this gives reasonable results
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp()
    };

    if x < (a + 1.0) / (a + b + 2.0) {
        bt * beta_cf(a, b, x) / a
    } else {
        1.0 - bt * beta_cf(b, a, 1.0 - x) / b
    }
}

/// Continued fraction for incomplete beta function.
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let max_iter = 100;
    let eps = 1e-10;

    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;

    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < 1e-30 {
        d = 1e-30;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=max_iter {
        let m = m as f64;
        let m2 = 2.0 * m;

        // Even step
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        h *= d * c;

        // Odd step
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < eps {
            break;
        }
    }

    h
}

/// Log gamma function approximation (Lanczos).
fn ln_gamma(x: f64) -> f64 {
    let g = 7;
    let coeffs = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    if x < 0.5 {
        // Reflection formula
        PI.ln() - (PI * x).sin().ln() - ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut sum = coeffs[0];
        for (i, &c) in coeffs.iter().skip(1).enumerate() {
            sum += c / (x + i as f64 + 1.0);
        }
        let t = x + g as f64 + 0.5;
        0.5 * (2.0 * PI).ln() + (t).ln() * (x + 0.5) - t + sum.ln()
    }
}

// ============================================================================
// Difficulty Bucket Metrics
// ============================================================================

/// Metrics for a single difficulty bucket.
#[derive(Debug, Clone, Serialize)]
pub struct DifficultyBucketMetrics {
    /// Bucket name ("easy", "medium", "hard")
    pub bucket: String,
    /// Coverage percentage for this bucket
    pub coverage_pct: f64,
    /// Number of items mastered in this bucket
    pub items_mastered: usize,
    /// Total items in this bucket
    pub items_total: usize,
}

/// Difficulty bucket classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyBucket {
    Easy,   // [1.0, 3.0)
    Medium, // [3.0, 6.0)
    Hard,   // [6.0, 10.0]
}

impl DifficultyBucket {
    /// Classify a difficulty value into a bucket.
    pub fn from_difficulty(difficulty: f64) -> Self {
        if difficulty < 3.0 {
            DifficultyBucket::Easy
        } else if difficulty < 6.0 {
            DifficultyBucket::Medium
        } else {
            DifficultyBucket::Hard
        }
    }

    /// Get the bucket name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            DifficultyBucket::Easy => "easy",
            DifficultyBucket::Medium => "medium",
            DifficultyBucket::Hard => "hard",
        }
    }
}

/// Compute difficulty bucket metrics from item difficulties and mastery states.
///
/// # Arguments
/// * `items` - Vec of (node_id, difficulty, is_mastered)
pub fn compute_difficulty_buckets(items: &[(i64, f64, bool)]) -> Vec<DifficultyBucketMetrics> {
    let mut easy_mastered = 0;
    let mut easy_total = 0;
    let mut medium_mastered = 0;
    let mut medium_total = 0;
    let mut hard_mastered = 0;
    let mut hard_total = 0;

    for &(_node_id, difficulty, is_mastered) in items {
        match DifficultyBucket::from_difficulty(difficulty) {
            DifficultyBucket::Easy => {
                easy_total += 1;
                if is_mastered {
                    easy_mastered += 1;
                }
            }
            DifficultyBucket::Medium => {
                medium_total += 1;
                if is_mastered {
                    medium_mastered += 1;
                }
            }
            DifficultyBucket::Hard => {
                hard_total += 1;
                if is_mastered {
                    hard_mastered += 1;
                }
            }
        }
    }

    vec![
        DifficultyBucketMetrics {
            bucket: "easy".to_string(),
            coverage_pct: if easy_total > 0 {
                easy_mastered as f64 / easy_total as f64
            } else {
                0.0
            },
            items_mastered: easy_mastered,
            items_total: easy_total,
        },
        DifficultyBucketMetrics {
            bucket: "medium".to_string(),
            coverage_pct: if medium_total > 0 {
                medium_mastered as f64 / medium_total as f64
            } else {
                0.0
            },
            items_mastered: medium_mastered,
            items_total: medium_total,
        },
        DifficultyBucketMetrics {
            bucket: "hard".to_string(),
            coverage_pct: if hard_total > 0 {
                hard_mastered as f64 / hard_total as f64
            } else {
                0.0
            },
            items_mastered: hard_mastered,
            items_total: hard_total,
        },
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_stats_from_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = MetricStats::from_values(&values);

        assert_eq!(stats.n, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!(stats.std > 0.0);
        assert!(stats.ci_lower < stats.mean);
        assert!(stats.ci_upper > stats.mean);
    }

    #[test]
    fn test_metric_stats_single_value() {
        let values = vec![5.0];
        let stats = MetricStats::from_values(&values);

        assert_eq!(stats.n, 1);
        assert_eq!(stats.mean, 5.0);
        assert_eq!(stats.std, 0.0);
        assert_eq!(stats.ci_lower, 5.0);
        assert_eq!(stats.ci_upper, 5.0);
    }

    #[test]
    fn test_metric_stats_empty() {
        let values: Vec<f64> = vec![];
        let stats = MetricStats::from_values(&values);

        assert_eq!(stats.n, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_welchs_t_test_identical() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p = welchs_t_test(&a, &b);

        // Identical samples should have high p-value
        assert!(p > 0.9, "p-value for identical samples: {}", p);
    }

    #[test]
    fn test_welchs_t_test_different() {
        let a = vec![0.0; 20];
        let b = vec![10.0; 20];
        let p = welchs_t_test(&a, &b);

        // Clearly different samples should have tiny p-value
        assert!(p < 0.001, "p-value for different samples: {}", p);
    }

    #[test]
    fn test_welchs_t_test_insufficient_data() {
        let a = vec![1.0];
        let b = vec![2.0, 3.0];
        let p = welchs_t_test(&a, &b);

        assert_eq!(p, 1.0);
    }

    #[test]
    fn test_difficulty_bucket_classification() {
        assert_eq!(
            DifficultyBucket::from_difficulty(1.0),
            DifficultyBucket::Easy
        );
        assert_eq!(
            DifficultyBucket::from_difficulty(2.9),
            DifficultyBucket::Easy
        );
        assert_eq!(
            DifficultyBucket::from_difficulty(3.0),
            DifficultyBucket::Medium
        );
        assert_eq!(
            DifficultyBucket::from_difficulty(5.9),
            DifficultyBucket::Medium
        );
        assert_eq!(
            DifficultyBucket::from_difficulty(6.0),
            DifficultyBucket::Hard
        );
        assert_eq!(
            DifficultyBucket::from_difficulty(10.0),
            DifficultyBucket::Hard
        );
    }

    #[test]
    fn test_compute_difficulty_buckets() {
        let items = vec![
            (1, 1.0, true),  // easy, mastered
            (2, 2.0, true),  // easy, mastered
            (3, 4.0, true),  // medium, mastered
            (4, 5.0, false), // medium, not mastered
            (5, 7.0, false), // hard, not mastered
            (6, 8.0, true),  // hard, mastered
        ];

        let buckets = compute_difficulty_buckets(&items);

        assert_eq!(buckets.len(), 3);

        // Easy: 2/2 = 100%
        assert_eq!(buckets[0].items_mastered, 2);
        assert_eq!(buckets[0].items_total, 2);
        assert!((buckets[0].coverage_pct - 1.0).abs() < 0.001);

        // Medium: 1/2 = 50%
        assert_eq!(buckets[1].items_mastered, 1);
        assert_eq!(buckets[1].items_total, 2);
        assert!((buckets[1].coverage_pct - 0.5).abs() < 0.001);

        // Hard: 1/2 = 50%
        assert_eq!(buckets[2].items_mastered, 1);
        assert_eq!(buckets[2].items_total, 2);
    }

    #[test]
    fn test_timeline_aggregation() {
        let student1 = vec![
            StudentDailyPoint {
                day: 0,
                coverage_pct: 0.1,
                items_mastered: 1,
                cumulative_reviews: 5,
            },
            StudentDailyPoint {
                day: 1,
                coverage_pct: 0.2,
                items_mastered: 2,
                cumulative_reviews: 10,
            },
        ];
        let student2 = vec![
            StudentDailyPoint {
                day: 0,
                coverage_pct: 0.3,
                items_mastered: 3,
                cumulative_reviews: 8,
            },
            StudentDailyPoint {
                day: 1,
                coverage_pct: 0.4,
                items_mastered: 4,
                cumulative_reviews: 15,
            },
        ];

        let timeline = aggregate_timeline(&[student1, student2], 2);

        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].day, 0);
        assert!((timeline[0].coverage_mean - 0.2).abs() < 0.001); // (0.1 + 0.3) / 2
        assert!((timeline[0].items_mastered_mean - 2.0).abs() < 0.001); // (1 + 3) / 2
        assert!((timeline[0].reviews_mean - 6.5).abs() < 0.001); // (5 + 8) / 2
    }
}
