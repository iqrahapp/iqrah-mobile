use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client-to-Server commands for WebSocket communication
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Command {
    /// Start a new exercise session
    /// Optional `axis` parameter for knowledge axis filtering (Phase 4)
    StartExercise {
        exercise_type: String,
        node_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        axis: Option<String>, // "memorization", "translation", etc.
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
    /// Start an Echo Recall session with one or more ayahs
    StartEchoRecall { ayah_node_ids: Vec<String> },
    /// Submit a word recall in Echo Recall mode
    SubmitEchoRecall {
        session_id: Uuid,
        word_node_id: String,
        recall_time_ms: u32,
    },
    /// End the current session and save state
    EndSession {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<Uuid>,
    },
    /// Get due items for a session (Phase 4)
    GetDueItems {
        limit: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        axis: Option<String>, // Optional axis filter
        #[serde(default)]
        is_high_yield_mode: bool,
    },
    /// Generate an exercise for a node (Phase 4.3)
    GenerateExercise {
        node_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        axis: Option<String>, // Optional axis override
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>, // "mcq_ar_to_en", "mcq_en_to_ar", or None for default
    },
    /// Check answer for an exercise (Phase 4.3)
    CheckAnswer { node_id: String, answer: String },
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
    StateUpdated { new_state: serde_json::Value },
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
    Error { message: String },
    /// Due items response (Phase 4)
    DueItems {
        items: Vec<serde_json::Value>, // ScoredItems serialized as JSON
    },
    /// Exercise generated (Phase 4.3)
    ExerciseGenerated {
        node_id: String,
        exercise_type: String,
        question: String,
        hint: Option<String>,
        /// For MCQ exercises, the answer options
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<Vec<String>>,
    },
    /// Answer checked (Phase 4.3)
    AnswerChecked {
        is_correct: bool,
        hint: Option<String>,
        correct_answer: Option<String>,
        /// For MCQ exercises, the answer options
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<Vec<String>>,
        /// Semantic grading label (Excellent/Partial/Incorrect)
        #[serde(skip_serializing_if = "Option::is_none")]
        semantic_grade: Option<String>,
        /// Semantic similarity score (0.0 to 1.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        similarity_score: Option<f32>,
    },
}
