//! Sanity logging for ISS v2.1 per spec §7.
//!
//! This module provides structured sanity summaries that are SEPARATE from
//! the evaluation metrics. They are used for debugging and verification only.
//!
//! **Invariant L1:** For every official comparison run, ISS must emit at least
//! one structured JSON summary per variant containing required fields.

use serde::Serialize;
use std::collections::HashMap;

/// Energy histogram with 5 buckets (per §7.1).
#[derive(Debug, Clone, Default, Serialize)]
pub struct EnergyHistogram {
    /// Items with energy in [0.0, 0.2)
    pub bucket_0_0_2: u32,
    /// Items with energy in [0.2, 0.4)
    pub bucket_0_2_0_4: u32,
    /// Items with energy in [0.4, 0.6)
    pub bucket_0_4_0_6: u32,
    /// Items with energy in [0.6, 0.8)
    pub bucket_0_6_0_8: u32,
    /// Items with energy in [0.8, 1.0]
    pub bucket_0_8_1_0: u32,
}

impl EnergyHistogram {
    /// Add an energy value to the appropriate bucket.
    pub fn add(&mut self, energy: f64) {
        if energy < 0.2 {
            self.bucket_0_0_2 += 1;
        } else if energy < 0.4 {
            self.bucket_0_2_0_4 += 1;
        } else if energy < 0.6 {
            self.bucket_0_4_0_6 += 1;
        } else if energy < 0.8 {
            self.bucket_0_6_0_8 += 1;
        } else {
            self.bucket_0_8_1_0 += 1;
        }
    }

    /// Merge another histogram into this one.
    pub fn merge(&mut self, other: &Self) {
        self.bucket_0_0_2 += other.bucket_0_0_2;
        self.bucket_0_2_0_4 += other.bucket_0_2_0_4;
        self.bucket_0_4_0_6 += other.bucket_0_4_0_6;
        self.bucket_0_6_0_8 += other.bucket_0_6_0_8;
        self.bucket_0_8_1_0 += other.bucket_0_8_1_0;
    }

    /// Get total count across all buckets.
    pub fn total(&self) -> u32 {
        self.bucket_0_0_2
            + self.bucket_0_2_0_4
            + self.bucket_0_4_0_6
            + self.bucket_0_6_0_8
            + self.bucket_0_8_1_0
    }
}

/// Retrievability histogram for analyzing coverage (v2.2 Step 3).
#[derive(Debug, Clone, Default, Serialize)]
pub struct RetrievabilityHistogram {
    /// R < 0.5 (Failed/Forgotten)
    pub bucket_0_0_5: u32,
    /// 0.5 <= R < 0.7 (Weak)
    pub bucket_0_5_0_7: u32,
    /// 0.7 <= R < 0.8 (Moderate)
    pub bucket_0_7_0_8: u32,
    /// 0.8 <= R < 0.9 (Good)
    pub bucket_0_8_0_9: u32,
    /// R >= 0.9 (Excellent/Mastered)
    pub bucket_0_9_1_0: u32,
}

impl RetrievabilityHistogram {
    /// Add a retrievability value.
    pub fn add(&mut self, retrievability: f64) {
        if retrievability < 0.5 {
            self.bucket_0_0_5 += 1;
        } else if retrievability < 0.7 {
            self.bucket_0_5_0_7 += 1;
        } else if retrievability < 0.8 {
            self.bucket_0_7_0_8 += 1;
        } else if retrievability < 0.9 {
            self.bucket_0_8_0_9 += 1;
        } else {
            self.bucket_0_9_1_0 += 1;
        }
    }

    /// Merge another histogram.
    pub fn merge(&mut self, other: &Self) {
        self.bucket_0_0_5 += other.bucket_0_0_5;
        self.bucket_0_5_0_7 += other.bucket_0_5_0_7;
        self.bucket_0_7_0_8 += other.bucket_0_7_0_8;
        self.bucket_0_8_0_9 += other.bucket_0_8_0_9;
        self.bucket_0_9_1_0 += other.bucket_0_9_1_0;
    }
}

/// Daily review count histogram for Invariant S1 verification.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyReviewHistogram {
    /// Reviews in range [0, 5)
    pub bucket_0_5: u32,
    /// Reviews in range [5, 10)
    pub bucket_5_10: u32,
    /// Reviews in range [10, 15)
    pub bucket_10_15: u32,
    /// Reviews in range [15, 20)
    pub bucket_15_20: u32,
    /// Reviews in range [20, 25)
    pub bucket_20_25: u32,
    /// Reviews in range [25, 30]
    pub bucket_25_30: u32,
    /// Reviews > 30
    pub bucket_30_plus: u32,
}

impl DailyReviewHistogram {
    /// Add a daily review count to the appropriate bucket.
    pub fn add(&mut self, count: usize) {
        if count < 5 {
            self.bucket_0_5 += 1;
        } else if count < 10 {
            self.bucket_5_10 += 1;
        } else if count < 15 {
            self.bucket_10_15 += 1;
        } else if count < 20 {
            self.bucket_15_20 += 1;
        } else if count < 25 {
            self.bucket_20_25 += 1;
        } else if count <= 30 {
            self.bucket_25_30 += 1;
        } else {
            self.bucket_30_plus += 1;
        }
    }

    /// Merge another histogram into this one.
    pub fn merge(&mut self, other: &Self) {
        self.bucket_0_5 += other.bucket_0_5;
        self.bucket_5_10 += other.bucket_5_10;
        self.bucket_10_15 += other.bucket_10_15;
        self.bucket_15_20 += other.bucket_15_20;
        self.bucket_20_25 += other.bucket_20_25;
        self.bucket_25_30 += other.bucket_25_30;
        self.bucket_30_plus += other.bucket_30_plus;
    }
}

/// Per-student sanity data collected during simulation.
#[derive(Debug, Clone, Default)]
pub struct StudentSanityData {
    /// Unique items seen by this student
    pub unique_items_seen: usize,
    /// Unique items that reached mastery threshold
    pub unique_items_mastered: usize,
    /// Per-item review counts
    pub per_item_reviews: HashMap<i64, u32>,
    /// Energy histogram at end of simulation
    pub energy_histogram: EnergyHistogram,
    /// Retrievability histogram at end of simulation (v2.2)
    pub retrievability_histogram: RetrievabilityHistogram,
    /// Number of days the student was active (not skipped, not gave up)
    pub days_active: u32,
    /// Daily review counts for this student
    pub daily_review_counts: Vec<usize>,
}

impl StudentSanityData {
    /// Record a review for an item.
    pub fn record_review(&mut self, node_id: i64) {
        *self.per_item_reviews.entry(node_id).or_insert(0) += 1;
    }

    /// Record daily review count.
    pub fn record_day(&mut self, reviews: usize) {
        self.daily_review_counts.push(reviews);
    }

    /// Compute average reviews per seen item.
    pub fn avg_reviews_per_seen_item(&self) -> f64 {
        if self.unique_items_seen == 0 {
            return 0.0;
        }
        let total: u32 = self.per_item_reviews.values().sum();
        total as f64 / self.unique_items_seen as f64
    }

    /// Compute median reviews per seen item.
    pub fn median_reviews_per_seen_item(&self) -> f64 {
        let mut counts: Vec<u32> = self.per_item_reviews.values().copied().collect();
        if counts.is_empty() {
            return 0.0;
        }
        counts.sort_unstable();
        let mid = counts.len() / 2;
        if counts.len() % 2 == 0 && counts.len() > 1 {
            (counts[mid - 1] + counts[mid]) as f64 / 2.0
        } else {
            counts[mid] as f64
        }
    }
}

/// Aggregated sanity summary for a variant (per §7.1 Invariant L1).
#[derive(Debug, Clone, Serialize)]
pub struct SanitySummary {
    /// Variant name (e.g., "iqrah_default")
    pub variant: String,
    /// Mean unique items seen across students
    pub unique_items_seen_mean: f64,
    /// Mean unique items mastered across students
    pub unique_items_mastered_mean: f64,
    /// Mean average reviews per seen item
    pub avg_reviews_per_seen_item_mean: f64,
    /// Median reviews per seen item (aggregated)
    pub median_reviews_per_seen_item: f64,
    /// Aggregated energy histogram
    pub energy_histogram: EnergyHistogram,
    /// Aggregated retrievability histogram (v2.2)
    pub retrievability_histogram: RetrievabilityHistogram,
    /// Mean days active across students
    pub days_active_mean: f64,
    /// Aggregated daily review histogram
    pub daily_review_histogram: DailyReviewHistogram,
    /// Number of students in this summary
    pub student_count: usize,
    /// Mean daily reviews per student
    pub daily_reviews_mean: f64,
    /// Min daily reviews across all students
    pub daily_reviews_min: usize,
    /// Max daily reviews across all students
    pub daily_reviews_max: usize,
}

impl SanitySummary {
    /// Create summary from student data.
    pub fn from_students(variant: &str, students: &[StudentSanityData]) -> Self {
        if students.is_empty() {
            return Self {
                variant: variant.to_string(),
                unique_items_seen_mean: 0.0,
                unique_items_mastered_mean: 0.0,
                avg_reviews_per_seen_item_mean: 0.0,
                median_reviews_per_seen_item: 0.0,
                energy_histogram: EnergyHistogram::default(),
                retrievability_histogram: RetrievabilityHistogram::default(),
                days_active_mean: 0.0,
                daily_review_histogram: DailyReviewHistogram::default(),
                student_count: 0,
                daily_reviews_mean: 0.0,
                daily_reviews_min: 0,
                daily_reviews_max: 0,
            };
        }

        let n = students.len() as f64;

        // Aggregate metrics
        let unique_items_seen_mean = students
            .iter()
            .map(|s| s.unique_items_seen as f64)
            .sum::<f64>()
            / n;
        let unique_items_mastered_mean = students
            .iter()
            .map(|s| s.unique_items_mastered as f64)
            .sum::<f64>()
            / n;
        let avg_reviews_per_seen_item_mean = students
            .iter()
            .map(|s| s.avg_reviews_per_seen_item())
            .sum::<f64>()
            / n;
        let days_active_mean = students.iter().map(|s| s.days_active as f64).sum::<f64>() / n;

        // Collect all median values for overall median
        let medians: Vec<f64> = students
            .iter()
            .map(|s| s.median_reviews_per_seen_item())
            .collect();
        let median_reviews_per_seen_item = if medians.is_empty() {
            0.0
        } else {
            let mut sorted = medians.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = sorted.len() / 2;
            if sorted.len() % 2 == 0 && sorted.len() > 1 {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            }
        };

        // Aggregate histograms
        let mut energy_histogram = EnergyHistogram::default();
        let mut retrievability_histogram = RetrievabilityHistogram::default();
        let mut daily_review_histogram = DailyReviewHistogram::default();
        for student in students {
            energy_histogram.merge(&student.energy_histogram);
            retrievability_histogram.merge(&student.retrievability_histogram);
            for &count in &student.daily_review_counts {
                daily_review_histogram.add(count);
            }
        }

        // Calculate daily review statistics
        let mut total_reviews = 0;
        let mut min_reviews = usize::MAX;
        let mut max_reviews = 0;
        let mut total_days = 0;

        for student in students {
            for &count in &student.daily_review_counts {
                total_reviews += count;
                min_reviews = min_reviews.min(count);
                max_reviews = max_reviews.max(count);
                total_days += 1;
            }
        }

        let daily_reviews_mean = if total_days > 0 {
            total_reviews as f64 / total_days as f64
        } else {
            0.0
        };

        if min_reviews == usize::MAX {
            min_reviews = 0;
        }

        Self {
            variant: variant.to_string(),
            unique_items_seen_mean,
            unique_items_mastered_mean,
            avg_reviews_per_seen_item_mean,
            median_reviews_per_seen_item,
            energy_histogram,
            retrievability_histogram,
            days_active_mean,
            daily_review_histogram,
            student_count: students.len(),
            daily_reviews_mean,
            daily_reviews_min: min_reviews,
            daily_reviews_max: max_reviews,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_histogram_add() {
        let mut hist = EnergyHistogram::default();
        hist.add(0.1);
        hist.add(0.3);
        hist.add(0.5);
        hist.add(0.7);
        hist.add(0.9);

        assert_eq!(hist.bucket_0_0_2, 1);
        assert_eq!(hist.bucket_0_2_0_4, 1);
        assert_eq!(hist.bucket_0_4_0_6, 1);
        assert_eq!(hist.bucket_0_6_0_8, 1);
        assert_eq!(hist.bucket_0_8_1_0, 1);
        assert_eq!(hist.total(), 5);
    }

    #[test]
    fn test_daily_review_histogram_add() {
        let mut hist = DailyReviewHistogram::default();
        hist.add(3);
        hist.add(7);
        hist.add(12);
        hist.add(18);
        hist.add(22);
        hist.add(28);
        hist.add(35);

        assert_eq!(hist.bucket_0_5, 1);
        assert_eq!(hist.bucket_5_10, 1);
        assert_eq!(hist.bucket_10_15, 1);
        assert_eq!(hist.bucket_15_20, 1);
        assert_eq!(hist.bucket_20_25, 1);
        assert_eq!(hist.bucket_25_30, 1);
        assert_eq!(hist.bucket_30_plus, 1);
    }

    #[test]
    fn test_student_sanity_data_avg_reviews() {
        let mut data = StudentSanityData::default();
        data.unique_items_seen = 3;
        data.per_item_reviews.insert(1, 5);
        data.per_item_reviews.insert(2, 3);
        data.per_item_reviews.insert(3, 4);

        assert!((data.avg_reviews_per_seen_item() - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_student_sanity_data_median_reviews() {
        let mut data = StudentSanityData::default();
        data.per_item_reviews.insert(1, 1);
        data.per_item_reviews.insert(2, 3);
        data.per_item_reviews.insert(3, 5);

        assert!((data.median_reviews_per_seen_item() - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_sanity_summary_from_empty() {
        let summary = SanitySummary::from_students("test", &[]);
        assert_eq!(summary.student_count, 0);
        assert_eq!(summary.unique_items_seen_mean, 0.0);
    }

    #[test]
    fn test_sanity_summary_from_students() {
        let mut student1 = StudentSanityData::default();
        student1.unique_items_seen = 10;
        student1.unique_items_mastered = 8;
        student1.days_active = 20;

        let mut student2 = StudentSanityData::default();
        student2.unique_items_seen = 12;
        student2.unique_items_mastered = 10;
        student2.days_active = 25;

        let summary = SanitySummary::from_students("test", &[student1, student2]);
        assert_eq!(summary.student_count, 2);
        assert!((summary.unique_items_seen_mean - 11.0).abs() < 0.01);
        assert!((summary.unique_items_mastered_mean - 9.0).abs() < 0.01);
        assert!((summary.days_active_mean - 22.5).abs() < 0.01);
    }
}
