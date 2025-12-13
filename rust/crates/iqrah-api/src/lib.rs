#![allow(unexpected_cfgs)]
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
// Flutter bridge API
pub mod api;
// Telemetry v1: Rust→Dart streaming (M2.9.4+)
pub mod telemetry;

// Re-export for FRB
pub use api::*;
pub use telemetry::{emit_daily_health, emit_panic, emit_session_complete, TelemetryEvent};

// Re-export types needed by generated code
pub use iqrah_core::{
    exercises::{ExerciseData, ExerciseService},
    ContentRepository, LearningService, SessionService, UserRepository,
};
