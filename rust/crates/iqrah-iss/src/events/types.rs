//! Event tracking for ISS simulation diagnostics.
//!
//! Captures all state transitions, scheduling decisions, and student reactions
//! in a structured, AI-digestible format for comparative analysis.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Energy buckets for tracking state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnergyBucket {
    /// Energy = 0.00 (never reviewed)
    Unknown,
    /// 0.00 < E <= 0.10 (just introduced)
    Aware,
    /// 0.10 < E <= 0.30 (beginning to learn)
    Beginner,
    /// 0.30 < E <= 0.60 (making progress)
    Intermediate,
    /// 0.60 < E <= 0.85 (nearly mastered)
    Advanced,
    /// 0.85 < E <= 1.00 (mastered)
    Mastered,
}

impl EnergyBucket {
    /// Convert an energy value to its bucket.
    pub fn from_energy(energy: f32) -> Self {
        if energy <= 0.0 {
            EnergyBucket::Unknown
        } else if energy <= 0.10 {
            EnergyBucket::Aware
        } else if energy <= 0.30 {
            EnergyBucket::Beginner
        } else if energy <= 0.60 {
            EnergyBucket::Intermediate
        } else if energy <= 0.85 {
            EnergyBucket::Advanced
        } else {
            EnergyBucket::Mastered
        }
    }

    /// Get display name for the bucket.
    pub fn name(&self) -> &'static str {
        match self {
            EnergyBucket::Unknown => "unknown",
            EnergyBucket::Aware => "aware",
            EnergyBucket::Beginner => "beginner",
            EnergyBucket::Intermediate => "intermediate",
            EnergyBucket::Advanced => "advanced",
            EnergyBucket::Mastered => "mastered",
        }
    }
}

/// Cause of an energy transition.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransitionCause {
    /// Energy drift (decay between reviews)
    Decay,
    /// Successful review
    ReviewSuccess,
    /// Failed review
    ReviewFail,
    /// Item first introduced
    Introduction,
}

/// Reason an urgent item was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkipReason {
    /// Session capacity reached
    SessionFull,
    /// Item had lower priority than selected items
    LowPriority,
    /// Mix percentage cap reached for this category
    MixCapReached,
    /// Item not eligible for scheduling
    NotEligible,
}

/// Category of an item in the session (for mix tracking).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SessionCategory {
    New,
    AlmostMastered,
    AlmostThere,
    Struggling,
    ReallyStruggling,
    Due,
}

/// Energy distribution histogram.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnergyHistogram {
    pub unknown: u32,
    pub aware: u32,
    pub beginner: u32,
    pub intermediate: u32,
    pub advanced: u32,
    pub mastered: u32,
}

impl EnergyHistogram {
    pub fn from_energies(energies: &[f32]) -> Self {
        let mut hist = EnergyHistogram::default();
        for &e in energies {
            match EnergyBucket::from_energy(e) {
                EnergyBucket::Unknown => hist.unknown += 1,
                EnergyBucket::Aware => hist.aware += 1,
                EnergyBucket::Beginner => hist.beginner += 1,
                EnergyBucket::Intermediate => hist.intermediate += 1,
                EnergyBucket::Advanced => hist.advanced += 1,
                EnergyBucket::Mastered => hist.mastered += 1,
            }
        }
        hist
    }

    pub fn total(&self) -> u32 {
        self.unknown
            + self.aware
            + self.beginner
            + self.intermediate
            + self.advanced
            + self.mastered
    }
}

/// Summary of session mix for a day.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMixSummary {
    pub new_count: u32,
    pub review_count: u32,
    pub total_sessions: u32,
}

/// All simulation events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum SimulationEvent {
    /// Energy bucket transition.
    EnergyTransition {
        day: u32,
        item_id: i64,
        from_bucket: EnergyBucket,
        to_bucket: EnergyBucket,
        energy: f32,
        cause: TransitionCause,
    },

    /// New item introduced to the student.
    ItemIntroduced {
        day: u32,
        item_id: i64,
        session_idx: u32,
    },

    /// Item selected for scheduling.
    ItemScheduled {
        day: u32,
        item_id: i64,
        urgency_score: f32,
        energy: f32,
        recall: f32,
        category: SessionCategory,
    },

    /// Urgent item skipped (not scheduled despite high urgency).
    ItemSkipped {
        day: u32,
        item_id: i64,
        urgency_score: f32,
        energy: f32,
        reason: SkipReason,
    },

    /// Review outcome for an item.
    ReviewOutcome {
        day: u32,
        item_id: i64,
        success: bool,
        recall_before: f32,
        recall_after: f32,
        energy_before: f32,
        energy_after: f32,
    },

    /// Frustration spike (large increase).
    FrustrationSpike {
        day: u32,
        frustration: f32,
        delta: f32,
        cause: String,
    },

    /// Student gave up.
    GaveUp {
        day: u32,
        frustration: f32,
        trigger: String,
    },

    /// Daily snapshot of simulation state.
    DaySnapshot {
        day: u32,
        energy_distribution: EnergyHistogram,
        coverage_mean_r: f32,
        introduced_count: u32,
        reviewed_count: u32,
        urgent_backlog: u32,
        session_mix: SessionMixSummary,
        // ISS v2.3: Capacity control metrics
        active_items: usize,
        capacity_utilization: f32,
        sustainable_intro_rate: f32,
        // ISS v2.4: Capacity diagnostics
        avg_review_interval: f32,
        predicted_maintenance: f32,
        actual_session_size: f32,
        // ISS v2.6: Cluster stability metrics
        cluster_size: usize,
        cluster_energy: f32,
        cluster_threshold: f32,
        intro_gated_by_cluster: bool,
        intro_gated_by_working_set: bool,
    },
}

/// Event sender for recording simulation events.
/// Clone this to pass to multiple producers.
#[derive(Debug, Clone)]
pub struct EventSender {
    tx: std::sync::mpsc::Sender<SimulationEvent>,
    enabled: bool,
}

impl EventSender {
    /// Check if event logging is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record an event (no-op if disabled).
    pub fn record(&self, event: SimulationEvent) {
        if !self.enabled {
            return;
        }
        let _ = self.tx.send(event); // Ignore send errors
    }

    /// Record an energy transition if buckets changed.
    pub fn record_energy_transition(
        &self,
        day: u32,
        item_id: i64,
        old_energy: f32,
        new_energy: f32,
        cause: TransitionCause,
    ) {
        if !self.enabled {
            return;
        }
        let from_bucket = EnergyBucket::from_energy(old_energy);
        let to_bucket = EnergyBucket::from_energy(new_energy);
        if from_bucket != to_bucket {
            self.record(SimulationEvent::EnergyTransition {
                day,
                item_id,
                from_bucket,
                to_bucket,
                energy: new_energy,
                cause,
            });
        }
    }
}

/// Event receiver for collecting simulation events.
pub struct EventReceiver {
    rx: std::sync::mpsc::Receiver<SimulationEvent>,
}

impl EventReceiver {
    /// Collect all received events.
    pub fn collect(self) -> Vec<SimulationEvent> {
        self.rx.try_iter().collect()
    }
}

/// Create an event channel for simulation diagnostics.
/// Returns (sender, receiver) pair. Clone sender for multiple producers.
pub fn event_channel(enabled: bool) -> (EventSender, EventReceiver) {
    let (tx, rx) = std::sync::mpsc::channel();
    (EventSender { tx, enabled }, EventReceiver { rx })
}

/// Write events to a JSONL file.
pub fn write_events_jsonl(events: &[SimulationEvent], path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for event in events {
        let json = serde_json::to_string(event)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(writer, "{}", json)?;
    }

    writer.flush()?;
    Ok(())
}

/// Compute statistics from events.
pub fn compute_stats(events: &[SimulationEvent]) -> EventStats {
    let mut stats = EventStats::default();

    for event in events {
        match event {
            SimulationEvent::EnergyTransition { cause, .. } => match cause {
                TransitionCause::Decay => stats.decay_transitions += 1,
                TransitionCause::ReviewSuccess => stats.success_transitions += 1,
                TransitionCause::ReviewFail => stats.fail_transitions += 1,
                TransitionCause::Introduction => stats.intro_transitions += 1,
            },
            SimulationEvent::ItemIntroduced { .. } => stats.items_introduced += 1,
            SimulationEvent::ItemScheduled { .. } => stats.items_scheduled += 1,
            SimulationEvent::ItemSkipped { reason, .. } => match reason {
                SkipReason::SessionFull => stats.skipped_session_full += 1,
                SkipReason::LowPriority => stats.skipped_low_priority += 1,
                SkipReason::MixCapReached => stats.skipped_mix_cap += 1,
                SkipReason::NotEligible => stats.skipped_not_eligible += 1,
            },
            SimulationEvent::ReviewOutcome { success, .. } => {
                if *success {
                    stats.reviews_success += 1;
                } else {
                    stats.reviews_fail += 1;
                }
            }
            SimulationEvent::FrustrationSpike { .. } => stats.frustration_spikes += 1,
            SimulationEvent::GaveUp { day, .. } => stats.gave_up_day = Some(*day),
            SimulationEvent::DaySnapshot { day, .. } => stats.days_completed = *day,
        }
    }

    stats
}

/// Quick statistics from event log.
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    pub decay_transitions: u32,
    pub success_transitions: u32,
    pub fail_transitions: u32,
    pub intro_transitions: u32,
    pub items_introduced: u32,
    pub items_scheduled: u32,
    pub skipped_session_full: u32,
    pub skipped_low_priority: u32,
    pub skipped_mix_cap: u32,
    pub skipped_not_eligible: u32,
    pub reviews_success: u32,
    pub reviews_fail: u32,
    pub frustration_spikes: u32,
    pub gave_up_day: Option<u32>,
    pub days_completed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_bucket_from_energy() {
        assert_eq!(EnergyBucket::from_energy(0.0), EnergyBucket::Unknown);
        assert_eq!(EnergyBucket::from_energy(0.05), EnergyBucket::Aware);
        assert_eq!(EnergyBucket::from_energy(0.10), EnergyBucket::Aware);
        assert_eq!(EnergyBucket::from_energy(0.15), EnergyBucket::Beginner);
        assert_eq!(EnergyBucket::from_energy(0.30), EnergyBucket::Beginner);
        assert_eq!(EnergyBucket::from_energy(0.45), EnergyBucket::Intermediate);
        assert_eq!(EnergyBucket::from_energy(0.60), EnergyBucket::Intermediate);
        assert_eq!(EnergyBucket::from_energy(0.75), EnergyBucket::Advanced);
        assert_eq!(EnergyBucket::from_energy(0.85), EnergyBucket::Advanced);
        assert_eq!(EnergyBucket::from_energy(0.90), EnergyBucket::Mastered);
        assert_eq!(EnergyBucket::from_energy(1.0), EnergyBucket::Mastered);
    }

    #[test]
    fn test_event_channel_disabled() {
        let (sender, receiver) = event_channel(false);
        sender.record(SimulationEvent::ItemIntroduced {
            day: 1,
            item_id: 123,
            session_idx: 0,
        });
        let events = receiver.collect();
        assert!(events.is_empty());
    }

    #[test]
    fn test_event_channel_enabled() {
        let (sender, receiver) = event_channel(true);
        sender.record(SimulationEvent::ItemIntroduced {
            day: 1,
            item_id: 123,
            session_idx: 0,
        });
        let events = receiver.collect();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_energy_transition_recording() {
        let (sender, receiver) = event_channel(true);
        // Same bucket - should not record
        sender.record_energy_transition(1, 100, 0.05, 0.08, TransitionCause::ReviewSuccess);

        // Different buckets - should record
        sender.record_energy_transition(1, 100, 0.05, 0.15, TransitionCause::ReviewSuccess);

        let events = receiver.collect();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_energy_histogram() {
        let energies = vec![0.0, 0.0, 0.05, 0.15, 0.45, 0.75, 0.90];
        let hist = EnergyHistogram::from_energies(&energies);
        assert_eq!(hist.unknown, 2);
        assert_eq!(hist.aware, 1);
        assert_eq!(hist.beginner, 1);
        assert_eq!(hist.intermediate, 1);
        assert_eq!(hist.advanced, 1);
        assert_eq!(hist.mastered, 1);
        assert_eq!(hist.total(), 7);
    }
}
