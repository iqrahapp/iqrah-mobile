use chrono::{DateTime, Utc};
use iqrah_core::ReviewGrade;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct RunDebugReport {
    pub scenario_name: String,
    pub timestamp: DateTime<Utc>,
    pub variants: Vec<VariantDebugReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantDebugReport {
    pub variant_name: String,
    pub students: Vec<StudentDebugSummary>,
    pub scheduler: SchedulerDebugStats,
    pub brain: BrainDebugStats,
    pub fsrs: FsrsDebugStats,
    pub plan: PlanDebugStats,
}

impl VariantDebugReport {
    pub fn from_summaries(variant_name: &str, summaries: &[StudentDebugSummary]) -> Self {
        let mut scheduler_stats = SchedulerDebugStats::default();
        let mut brain_stats = BrainDebugStats::default();
        let mut fsrs_stats = FsrsDebugStats::default();
        let plan_stats = PlanDebugStats::default();

        let n = summaries.len() as f64;

        // Collect stability values across all students for aggregation
        let mut stab_1: Vec<f64> = Vec::new();
        let mut stab_2_4: Vec<f64> = Vec::new();
        let mut stab_5_9: Vec<f64> = Vec::new();
        let mut stab_10_plus: Vec<f64> = Vec::new();

        for s in summaries {
            // Scheduler Aggregation
            scheduler_stats.total_reviews += s.total_reviews as u64;
            scheduler_stats.r_bucket_counts.merge(&s.r_buckets);
            scheduler_stats
                .inter_review_delays_days
                .merge(&s.delay_buckets);

            // Brain Aggregation
            brain_stats.avg_mean_frustration += s.mean_frustration;
            brain_stats.avg_max_frustration += s.max_frustration;
            brain_stats.avg_final_weighted_failure += s.final_weighted_failure_score;
            if s.gave_up {
                brain_stats.avg_give_up_rate += 1.0;
                if let Some(day) = s.day_of_give_up {
                    brain_stats.give_up_day_histogram.push((day, 1));
                }
            }
            brain_stats.avg_skip_rate += s.skip_rate;

            // FSRS Aggregation
            fsrs_stats.total_reviews += s.total_reviews as u64;
            fsrs_stats.grade_counts.merge(&s.grade_counts);

            // Collect stability values for averaging
            stab_1.extend(&s.stability_after_1);
            stab_2_4.extend(&s.stability_after_2_4);
            stab_5_9.extend(&s.stability_after_5_9);
            stab_10_plus.extend(&s.stability_after_10_plus);
        }

        if n > 0.0 {
            brain_stats.avg_mean_frustration /= n;
            brain_stats.avg_max_frustration /= n;
            brain_stats.avg_final_weighted_failure /= n;
            brain_stats.avg_give_up_rate /= n;
            brain_stats.avg_skip_rate /= n;
        }

        // Compute average stabilities
        fsrs_stats.avg_stability_after_1_review = if stab_1.is_empty() {
            0.0
        } else {
            stab_1.iter().sum::<f64>() / stab_1.len() as f64
        };
        fsrs_stats.avg_stability_after_2_4_reviews = if stab_2_4.is_empty() {
            0.0
        } else {
            stab_2_4.iter().sum::<f64>() / stab_2_4.len() as f64
        };
        fsrs_stats.avg_stability_after_5_9_reviews = if stab_5_9.is_empty() {
            0.0
        } else {
            stab_5_9.iter().sum::<f64>() / stab_5_9.len() as f64
        };
        fsrs_stats.avg_stability_after_10_plus_reviews = if stab_10_plus.is_empty() {
            0.0
        } else {
            stab_10_plus.iter().sum::<f64>() / stab_10_plus.len() as f64
        };

        Self {
            variant_name: variant_name.to_string(),
            students: summaries.to_vec(),
            scheduler: scheduler_stats,
            brain: brain_stats,
            fsrs: fsrs_stats,
            plan: plan_stats,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentDebugSummary {
    pub student_index: usize,
    pub gave_up: bool,
    pub day_of_give_up: Option<u32>,
    pub days_active: u32,
    pub days_skipped: u32,
    pub total_minutes: f64,
    pub total_reviews: u32,
    pub avg_items_per_session: f64,
    pub fail_rate: f64, // Again / total_reviews
    pub grade_counts: GradeCounts,
    pub avg_r_at_review: f64,     // mean retrievability at review time
    pub frac_reviews_r_low: f64,  // r < 0.4
    pub frac_reviews_r_mid: f64,  // 0.4 <= r < 0.7
    pub frac_reviews_r_good: f64, // 0.7 <= r < 0.9
    pub frac_reviews_r_over: f64, // r >= 0.9
    pub mean_frustration: f64,
    pub max_frustration: f64,
    pub final_weighted_failure_score: f64,
    pub skip_rate: f64,              // days_skipped / target_days
    pub r_buckets: RBuckets,         // Raw counts for aggregation
    pub delay_buckets: DelayBuckets, // Raw counts for aggregation
    // Per-item review counts for fairness evaluation
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub per_item_reviews: HashMap<i64, u32>,
    // Stability tracking by review count bucket
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stability_after_1: Vec<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stability_after_2_4: Vec<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stability_after_5_9: Vec<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stability_after_10_plus: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GradeCounts {
    pub again: u32,
    pub hard: u32,
    pub good: u32,
    pub easy: u32,
}

impl GradeCounts {
    pub fn inc(&mut self, grade: ReviewGrade) {
        match grade {
            ReviewGrade::Again => self.again += 1,
            ReviewGrade::Hard => self.hard += 1,
            ReviewGrade::Good => self.good += 1,
            ReviewGrade::Easy => self.easy += 1,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.again += other.again;
        self.hard += other.hard;
        self.good += other.good;
        self.easy += other.easy;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SchedulerDebugStats {
    pub total_reviews: u64,
    pub r_bucket_counts: RBuckets, // aggregated across all students
    pub inter_review_delays_days: DelayBuckets,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RBuckets {
    pub r_0_0_3: u64,     // 0.0–0.3
    pub r_0_3_0_6: u64,   // 0.3–0.6
    pub r_0_6_0_85: u64,  // 0.6–0.85
    pub r_0_85_0_95: u64, //0.85–0.95
    pub r_0_95_1_0: u64,  //0.95–1.0
}

impl RBuckets {
    pub fn inc(&mut self, r: f64) {
        if r < 0.3 {
            self.r_0_0_3 += 1;
        } else if r < 0.6 {
            self.r_0_3_0_6 += 1;
        } else if r < 0.85 {
            self.r_0_6_0_85 += 1;
        } else if r < 0.95 {
            self.r_0_85_0_95 += 1;
        } else {
            self.r_0_95_1_0 += 1;
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.r_0_0_3 += other.r_0_0_3;
        self.r_0_3_0_6 += other.r_0_3_0_6;
        self.r_0_6_0_85 += other.r_0_6_0_85;
        self.r_0_85_0_95 += other.r_0_85_0_95;
        self.r_0_95_1_0 += other.r_0_95_1_0;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DelayBuckets {
    pub d_0: u64, // same day
    pub d_1: u64, // 1 day
    pub d_2_3: u64,
    pub d_4_7: u64,
    pub d_8_14: u64,
    pub d_15_30: u64,
    pub d_31_plus: u64,
}

impl DelayBuckets {
    pub fn inc(&mut self, delay: u32) {
        match delay {
            0 => self.d_0 += 1,
            1 => self.d_1 += 1,
            2..=3 => self.d_2_3 += 1,
            4..=7 => self.d_4_7 += 1,
            8..=14 => self.d_8_14 += 1,
            15..=30 => self.d_15_30 += 1,
            _ => self.d_31_plus += 1,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.d_0 += other.d_0;
        self.d_1 += other.d_1;
        self.d_2_3 += other.d_2_3;
        self.d_4_7 += other.d_4_7;
        self.d_8_14 += other.d_8_14;
        self.d_15_30 += other.d_15_30;
        self.d_31_plus += other.d_31_plus;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BrainDebugStats {
    pub avg_skip_rate: f64,
    pub avg_give_up_rate: f64,
    pub avg_mean_frustration: f64,
    pub p90_mean_frustration: f64,
    pub avg_max_frustration: f64,
    pub p90_max_frustration: f64,
    pub avg_final_weighted_failure: f64,
    pub give_up_day_histogram: Vec<(u32, u32)>, // (day, count)
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct FsrsDebugStats {
    pub total_reviews: u64,
    pub grade_counts: GradeCounts,
    pub avg_stability_after_1_review: f64,
    pub avg_stability_after_2_4_reviews: f64,
    pub avg_stability_after_5_9_reviews: f64,
    pub avg_stability_after_10_plus_reviews: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PlanDebugStats {
    pub plan_faithfulness: f64,
    pub avg_first_seen_day_high_priority: f64, // top 25% priority
    pub avg_first_seen_day_mid_priority: f64,  // middle 50%
    pub avg_first_seen_day_low_priority: f64,  // bottom 25%
    pub pct_high_priority_seen_first_10pct_days: f64,
}

// Helper to calculate P90
pub fn percentile_90(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (v.len() as f64 * 0.9) as usize;
    v.get(idx).copied().unwrap_or(0.0)
}

pub struct StudentDebugAccumulator {
    pub student_index: usize,
    pub grade_counts: GradeCounts,
    pub r_buckets: RBuckets,
    pub reviews: u32,
    pub total_minutes: f64,
    pub active_days: u32,
    pub skipped_days: u32,
    pub give_up: bool,
    pub day_of_give_up: Option<u32>,
    pub items_per_session_sum: f64,
    pub session_count: u32,
    pub frustration_daily_sum: f64,
    pub frustration_max: f64,
    pub frustration_days: u32,
    pub weighted_failure: f64,
    pub r_sum: f64,
    pub r_count: u32,
    pub delay_buckets: DelayBuckets,
    // Per-item review counts for fairness evaluation
    pub per_item_reviews: HashMap<i64, u32>,
    // Stability tracking by review count bucket
    pub stability_after_1: Vec<f64>,
    pub stability_after_2_4: Vec<f64>,
    pub stability_after_5_9: Vec<f64>,
    pub stability_after_10_plus: Vec<f64>,
}

impl StudentDebugAccumulator {
    pub fn new(student_index: usize) -> Self {
        Self {
            student_index,
            grade_counts: Default::default(),
            r_buckets: Default::default(),
            reviews: 0,
            total_minutes: 0.0,
            active_days: 0,
            skipped_days: 0,
            give_up: false,
            day_of_give_up: None,
            items_per_session_sum: 0.0,
            session_count: 0,
            frustration_daily_sum: 0.0,
            frustration_max: 0.0,
            frustration_days: 0,
            weighted_failure: 0.0,
            r_sum: 0.0,
            r_count: 0,
            delay_buckets: Default::default(),
            per_item_reviews: HashMap::new(),
            stability_after_1: Vec::new(),
            stability_after_2_4: Vec::new(),
            stability_after_5_9: Vec::new(),
            stability_after_10_plus: Vec::new(),
        }
    }

    pub fn record_review(&mut self, grade: ReviewGrade, r: f64, elapsed_days: u32, node_id: i64) {
        self.reviews += 1;
        self.grade_counts.inc(grade);
        self.r_buckets.inc(r);
        self.delay_buckets.inc(elapsed_days);
        self.r_sum += r;
        self.r_count += 1;
        // Track per-item review counts for fairness evaluation
        *self.per_item_reviews.entry(node_id).or_insert(0) += 1;
    }

    /// Record stability value after a review, bucketed by review count.
    pub fn record_stability(&mut self, stability: f64, review_count: u32) {
        match review_count {
            1 => self.stability_after_1.push(stability),
            2..=4 => self.stability_after_2_4.push(stability),
            5..=9 => self.stability_after_5_9.push(stability),
            _ => self.stability_after_10_plus.push(stability),
        }
    }

    pub fn record_session(&mut self, items: usize, minutes: f64) {
        self.session_count += 1;
        self.items_per_session_sum += items as f64;
        self.total_minutes += minutes;
        self.active_days += 1;
    }

    pub fn record_day_end(&mut self, _skipped: bool, frustration: f64, failure_score: f64) {
        // skipped logic is handled by caller inc skipped_days?
        // If skipped, we shouldn't record frustration day? Or frustration is 0?
        // Assuming caller calls this only if active day or handles skipping logic.
        // Actually simulator loop handles skipping.
        self.frustration_daily_sum += frustration;
        if frustration > self.frustration_max {
            self.frustration_max = frustration;
        }
        self.frustration_days += 1;
        // weighted failure is a running score, we just take the final value
        self.weighted_failure = failure_score;
    }

    pub fn finish(self) -> StudentDebugSummary {
        let avg_items = if self.session_count > 0 {
            self.items_per_session_sum / self.session_count as f64
        } else {
            0.0
        };
        let fail_rate = if self.reviews > 0 {
            self.grade_counts.again as f64 / self.reviews as f64
        } else {
            0.0
        };
        let avg_r = if self.r_count > 0 {
            self.r_sum / self.r_count as f64
        } else {
            0.0
        };

        let total_r_bucket = self.reviews as f64;
        let frac = |cnt: u64| {
            if total_r_bucket > 0.0 {
                cnt as f64 / total_r_bucket
            } else {
                0.0
            }
        };

        let mean_frustration = if self.frustration_days > 0 {
            self.frustration_daily_sum / self.frustration_days as f64
        } else {
            0.0
        };

        let skip_rate = if (self.active_days + self.skipped_days) > 0 {
            self.skipped_days as f64 / (self.active_days + self.skipped_days) as f64
        } else {
            0.0
        };

        StudentDebugSummary {
            student_index: self.student_index,
            gave_up: self.give_up,
            day_of_give_up: self.day_of_give_up,
            days_active: self.active_days,
            days_skipped: self.skipped_days,
            total_minutes: self.total_minutes,
            total_reviews: self.reviews,
            avg_items_per_session: avg_items,
            fail_rate,
            grade_counts: self.grade_counts,
            avg_r_at_review: avg_r,
            frac_reviews_r_low: frac(self.r_buckets.r_0_0_3),
            frac_reviews_r_mid: frac(self.r_buckets.r_0_3_0_6),
            frac_reviews_r_good: frac(self.r_buckets.r_0_6_0_85),
            frac_reviews_r_over: frac(self.r_buckets.r_0_95_1_0),
            mean_frustration,
            max_frustration: self.frustration_max,
            final_weighted_failure_score: self.weighted_failure,
            skip_rate,
            r_buckets: self.r_buckets,
            delay_buckets: self.delay_buckets,
            per_item_reviews: self.per_item_reviews,
            stability_after_1: self.stability_after_1,
            stability_after_2_4: self.stability_after_2_4,
            stability_after_5_9: self.stability_after_5_9,
            stability_after_10_plus: self.stability_after_10_plus,
        }
    }
}
