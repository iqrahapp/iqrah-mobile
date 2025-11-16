use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client-to-Server commands for WebSocket communication
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Command {
    /// Start a new exercise session
    StartExercise {
        exercise_type: String,
        node_id: String,
    },
    /// Submit an answer for a generic exercise (MCQ, etc.)
    SubmitAnswer {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<Uuid>,
        answer: serde_json::Value,
    },
    /// Update a word's state in Memorization Mode (MVP)
    UpdateMemorizationWord {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<Uuid>,
        word_node_id: String,
        action: String, // "Tap", "LongPress", etc.
    },
    /// End the current session and save state
    EndSession {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<Uuid>,
    },
}

/// Server-to-Client events for WebSocket communication
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Event {
    /// Session has started successfully
    SessionStarted {
        session_id: Uuid,
        initial_state: serde_json::Value,
    },
    /// State has been updated
    StateUpdated {
        new_state: serde_json::Value,
    },
    /// Feedback on a submitted answer
    Feedback {
        is_correct: bool,
        correct_answer: Option<String>,
    },
    /// Session has finished
    SessionFinished {
        final_state: serde_json::Value,
        summary: serde_json::Value,
    },
    /// An error occurred
    Error {
        message: String,
    },
}
