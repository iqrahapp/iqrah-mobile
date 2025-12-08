//! Scheduler v2.0 Event Emission (Invariant O1)
//!
//! This module provides observability into the scheduler pipeline via event emission.
//! Events are emitted at key decision points to enable debugging and monitoring.

use std::sync::Mutex;

// ============================================================================
// EVENT TYPES
// ============================================================================

/// Scheduler pipeline events for observability
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// Item was filtered out of candidate set
    CandidateFiltered { node_id: i64, reason: FilterReason },

    /// Priority score computed for a candidate
    PriorityComputed {
        node_id: i64,
        components: ScoreBreakdown,
    },

    /// Session composition completed
    SessionComposed {
        mode: SessionModeEvent,
        buckets: BucketAllocation,
    },

    /// Fairness/coverage boost applied
    FairnessCorrection {
        node_id: i64,
        coverage_factor: f32,
        fairness_additive: f32,
    },

    /// Item blocked by prerequisite gate
    PrerequisiteGateFailed {
        node_id: i64,
        unsatisfied_parents: Vec<i64>,
    },
}

/// Reason for candidate filtering
#[derive(Debug, Clone)]
pub enum FilterReason {
    /// Filtered due to high energy + far due date (optional optimization)
    HighEnergyNotDue { energy: f32, days_until_due: f32 },
    /// User/content disabled
    Disabled,
}

/// Session mode for event reporting (mirrors SessionMode but without dependencies)
#[derive(Debug, Clone, Copy)]
pub enum SessionModeEvent {
    Revision,
    MixedLearning,
}

/// Breakdown of priority score components
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub urgency_factor: f32,
    pub coverage_factor: f32,
    pub readiness: f32,
    pub foundational: f32,
    pub influence: f32,
    pub fairness_additive: f32,
    pub final_score: f64,
}

impl ScoreBreakdown {
    /// Create a new score breakdown
    pub fn new(
        urgency_factor: f32,
        coverage_factor: f32,
        readiness: f32,
        foundational: f32,
        influence: f32,
        fairness_additive: f32,
        final_score: f64,
    ) -> Self {
        Self {
            urgency_factor,
            coverage_factor,
            readiness,
            foundational,
            influence,
            fairness_additive,
            final_score,
        }
    }
}

/// Bucket allocation for session composition
#[derive(Debug, Clone, Default)]
pub struct BucketAllocation {
    pub new: usize,
    pub almost_mastered: usize,
    pub almost_there: usize,
    pub struggling: usize,
    pub really_struggling: usize,
}

impl BucketAllocation {
    /// Create allocation for revision mode (difficulty-based)
    pub fn revision(easy: usize, medium: usize, hard: usize) -> Self {
        Self {
            new: 0,
            almost_mastered: easy,
            almost_there: medium,
            struggling: hard,
            really_struggling: 0,
        }
    }

    /// Create allocation for mixed learning mode (mastery-based)
    pub fn mixed_learning(
        new: usize,
        almost_mastered: usize,
        almost_there: usize,
        struggling: usize,
        really_struggling: usize,
    ) -> Self {
        Self {
            new,
            almost_mastered,
            almost_there,
            struggling,
            really_struggling,
        }
    }
}

// ============================================================================
// EVENT SINK TRAIT
// ============================================================================

/// Trait for consuming scheduler events
pub trait SchedulerEventSink: Send + Sync {
    /// Emit a scheduler event
    fn emit(&self, event: SchedulerEvent);
}

// ============================================================================
// NULL EVENT SINK (for ISS performance)
// ============================================================================

/// No-op sink for ISS simulator (zero overhead)
pub struct NullEventSink;

impl SchedulerEventSink for NullEventSink {
    #[inline]
    fn emit(&self, _event: SchedulerEvent) {
        // No-op for performance
    }
}

// ============================================================================
// LOGGING EVENT SINK (for production)
// ============================================================================

/// Logging sink that emits events via tracing
pub struct LoggingEventSink;

impl SchedulerEventSink for LoggingEventSink {
    fn emit(&self, event: SchedulerEvent) {
        tracing::debug!(?event, "scheduler_event");
    }
}

// ============================================================================
// COLLECTING EVENT SINK (for tests)
// ============================================================================

/// Collecting sink that stores events for test assertions
pub struct CollectingEventSink {
    events: Mutex<Vec<SchedulerEvent>>,
}

impl CollectingEventSink {
    /// Create a new collecting sink
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    /// Get all collected events
    pub fn events(&self) -> Vec<SchedulerEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Clear collected events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    /// Count events of a specific type
    pub fn count_priority_computed(&self) -> usize {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| matches!(e, SchedulerEvent::PriorityComputed { .. }))
            .count()
    }

    /// Count prerequisite gate failures
    pub fn count_gate_failures(&self) -> usize {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| matches!(e, SchedulerEvent::PrerequisiteGateFailed { .. }))
            .count()
    }

    /// Check if session composed event was emitted
    pub fn has_session_composed(&self) -> bool {
        self.events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, SchedulerEvent::SessionComposed { .. }))
    }
}

impl Default for CollectingEventSink {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerEventSink for CollectingEventSink {
    fn emit(&self, event: SchedulerEvent) {
        self.events.lock().unwrap().push(event);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_sink_does_nothing() {
        let sink = NullEventSink;
        sink.emit(SchedulerEvent::PriorityComputed {
            node_id: 1,
            components: ScoreBreakdown::new(1.0, 1.0, 1.0, 0.5, 0.3, 0.0, 1.8),
        });
        // Just verify no panic
    }

    #[test]
    fn test_collecting_sink_captures_events() {
        let sink = CollectingEventSink::new();

        sink.emit(SchedulerEvent::PriorityComputed {
            node_id: 1,
            components: ScoreBreakdown::new(1.0, 1.0, 1.0, 0.5, 0.3, 0.0, 1.8),
        });
        sink.emit(SchedulerEvent::PrerequisiteGateFailed {
            node_id: 2,
            unsatisfied_parents: vec![3, 4],
        });

        assert_eq!(sink.events().len(), 2);
        assert_eq!(sink.count_priority_computed(), 1);
        assert_eq!(sink.count_gate_failures(), 1);
    }

    #[test]
    fn test_collecting_sink_clear() {
        let sink = CollectingEventSink::new();

        sink.emit(SchedulerEvent::PriorityComputed {
            node_id: 1,
            components: ScoreBreakdown::new(1.0, 1.0, 1.0, 0.5, 0.3, 0.0, 1.8),
        });

        assert_eq!(sink.events().len(), 1);
        sink.clear();
        assert_eq!(sink.events().len(), 0);
    }

    #[test]
    fn test_bucket_allocation_revision() {
        let alloc = BucketAllocation::revision(3, 2, 1);
        assert_eq!(alloc.new, 0);
        assert_eq!(alloc.almost_mastered, 3);
        assert_eq!(alloc.almost_there, 2);
        assert_eq!(alloc.struggling, 1);
    }

    #[test]
    fn test_bucket_allocation_mixed_learning() {
        let alloc = BucketAllocation::mixed_learning(2, 1, 3, 2, 1);
        assert_eq!(alloc.new, 2);
        assert_eq!(alloc.almost_mastered, 1);
        assert_eq!(alloc.almost_there, 3);
        assert_eq!(alloc.struggling, 2);
        assert_eq!(alloc.really_struggling, 1);
    }
}
