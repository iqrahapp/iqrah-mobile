//! ISS Telemetry v1 - Rust→Dart Events
//!
//! Architecture:
//! - Scheduler (Rust) is source of truth → emits events to internal buffer
//! - Dart polls periodically to drain buffer
//! - Events stored as JSON strings for flexibility

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Buffer capacity (drop oldest on overflow)
const BUFFER_CAPACITY: usize = 256;

/// Monotonic sequence counter
static SEQ_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Build SHA for tracing
pub static BUILD_SHA: &str = env!("CARGO_PKG_VERSION");

/// Global event buffer
static EVENT_BUFFER: Mutex<VecDeque<TelemetryEvent>> = Mutex::new(VecDeque::new());

/// Telemetry event - single struct for FRB bridging
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event type: "iss.daily_health", "rust.panic", etc.
    pub event: String,
    /// Unix timestamp in milliseconds
    pub timestamp_ms: i64,
    /// Monotonic sequence number (per process)
    pub seq: u64,
    /// JSON payload (flexible schema)
    pub payload_json: String,
    /// Build SHA for traceability
    pub build_sha: String,
}

impl TelemetryEvent {
    /// Create new event with auto-populated fields
    pub fn new(event: &str, payload: impl Serialize) -> Self {
        Self {
            event: event.to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            seq: SEQ_COUNTER.fetch_add(1, Ordering::SeqCst),
            payload_json: serde_json::to_string(&payload).unwrap_or_default(),
            build_sha: BUILD_SHA.to_string(),
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ============================================================================
// Payload Types
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyHealthPayload {
    pub day: u32,
    pub total_active: u32,
    pub introduced_today: u32,
    pub reviewed_today: u32,
    pub mean_r_today: f64,
    pub p10_r_today: f64,
    pub at_risk_ratio_0_9: f64,
    pub p90_due_age_days: f64,
    pub goal_size: u32,
    pub introduced_ratio: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionCompletePayload {
    pub session_id: String,
    pub items_reviewed: u32,
    pub session_duration_min: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanicPayload {
    pub function_name: String,
    pub message: String,
    pub location: Option<String>,
}

// ============================================================================
// Emission Functions
// ============================================================================

/// Push event to buffer (non-blocking, drops oldest on overflow)
fn emit_internal(event: TelemetryEvent) {
    if let Ok(mut buffer) = EVENT_BUFFER.lock() {
        if buffer.len() >= BUFFER_CAPACITY {
            buffer.pop_front(); // Drop oldest
        }
        buffer.push_back(event);
    }
}

/// Drain all events from buffer (call from Dart)
pub fn drain_events() -> Vec<String> {
    if let Ok(mut buffer) = EVENT_BUFFER.lock() {
        buffer.drain(..).map(|e| e.to_json()).collect()
    } else {
        Vec::new()
    }
}

/// Get pending event count
pub fn pending_event_count() -> usize {
    EVENT_BUFFER.lock().map(|b| b.len()).unwrap_or(0)
}

/// Emit daily health summary
#[allow(clippy::too_many_arguments)]
pub fn emit_daily_health(
    day: u32,
    total_active: u32,
    introduced_today: u32,
    reviewed_today: u32,
    mean_r_today: f64,
    p10_r_today: f64,
    at_risk_ratio_0_9: f64,
    p90_due_age_days: f64,
    goal_size: u32,
) {
    let introduced_ratio = if goal_size > 0 {
        total_active as f64 / goal_size as f64
    } else {
        0.0
    };

    let payload = DailyHealthPayload {
        day,
        total_active,
        introduced_today,
        reviewed_today,
        mean_r_today,
        p10_r_today,
        at_risk_ratio_0_9,
        p90_due_age_days,
        goal_size,
        introduced_ratio,
    };

    emit_internal(TelemetryEvent::new("iss.daily_health", payload));
}

/// Emit session complete summary
pub fn emit_session_complete(session_id: String, items_reviewed: u32, session_duration_min: f64) {
    let payload = SessionCompletePayload {
        session_id,
        items_reviewed,
        session_duration_min,
    };
    emit_internal(TelemetryEvent::new("iss.session_complete", payload));
}

/// Emit panic event
pub fn emit_panic(function_name: &str, message: &str, location: Option<&str>) {
    let payload = PanicPayload {
        function_name: function_name.to_string(),
        message: message.to_string(),
        location: location.map(|s| s.to_string()),
    };
    emit_internal(TelemetryEvent::new("rust.panic", payload));
}

// ============================================================================
// Macros
// ============================================================================

/// Catch unwind wrapper for FFI functions
#[macro_export]
macro_rules! ffi_safe {
    ($func_name:expr, $body:expr) => {{
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(result) => result,
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };

                $crate::telemetry::emit_panic(
                    $func_name,
                    &msg,
                    Some(concat!(file!(), ":", line!())),
                );

                Err(anyhow::anyhow!("Rust panic in {}: {}", $func_name, msg))
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = TelemetryEvent::new(
            "iss.daily_health",
            DailyHealthPayload {
                day: 30,
                total_active: 100,
                introduced_today: 5,
                reviewed_today: 20,
                mean_r_today: 0.95,
                p10_r_today: 0.90,
                at_risk_ratio_0_9: 0.05,
                p90_due_age_days: 30.0,
                goal_size: 564,
                introduced_ratio: 0.18,
            },
        );

        assert_eq!(event.event, "iss.daily_health");
        assert!(event.payload_json.contains("total_active"));
    }

    #[test]
    fn test_buffer_overflow() {
        // Fill buffer beyond capacity
        for i in 0..(BUFFER_CAPACITY + 10) {
            emit_daily_health(i as u32, 100, 5, 20, 0.95, 0.90, 0.05, 30.0, 564);
        }

        // Should only have BUFFER_CAPACITY events
        assert_eq!(pending_event_count(), BUFFER_CAPACITY);

        // Drain all
        let events = drain_events();
        assert_eq!(events.len(), BUFFER_CAPACITY);
        assert_eq!(pending_event_count(), 0);
    }

    #[test]
    fn test_emit_no_panic() {
        emit_daily_health(1, 100, 5, 20, 0.95, 0.90, 0.05, 30.0, 564);
        drain_events(); // Clear
    }
}
