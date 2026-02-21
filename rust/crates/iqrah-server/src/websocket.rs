use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    protocol::{Command, Event},
    AppState,
};
use iqrah_core::domain::node_id as nid;

/// Session state for a running exercise
#[derive(Debug, Clone)]
pub struct ExerciseSession {
    #[allow(dead_code)]
    pub session_id: Uuid,
    pub exercise_type: String,
    pub node_id: String,
    #[allow(dead_code)]
    pub user_id: String,
    /// Exercise-specific state (word energies for Memorization, etc.)
    pub state: serde_json::Value,
}

/// Shared session storage
pub type SessionMap = Arc<RwLock<HashMap<Uuid, ExerciseSession>>>;

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, app_state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let sessions: SessionMap = Arc::new(RwLock::new(HashMap::new()));
    let mut current_session_id: Option<Uuid> = None;

    // Default user for testing (in production, this would come from auth)
    let user_id = "test_user".to_string();

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        };

        if let Message::Text(text) = msg {
            tracing::debug!("Received command: {}", text);

            // Parse the command
            let command: Command = match serde_json::from_str(&text) {
                Ok(cmd) => cmd,
                Err(e) => {
                    let error_event = Event::Error {
                        message: format!("Invalid command: {}", e),
                    };
                    if let Err(e) = send_event(&mut sender, &error_event).await {
                        tracing::error!("Failed to send error event: {}", e);
                        break;
                    }
                    continue;
                }
            };

            // Handle the command
            let events = handle_command(
                command,
                &user_id,
                &app_state,
                Arc::clone(&sessions),
                current_session_id,
            )
            .await;

            // Send all response events and update current session if needed
            for event in &events {
                // Track the current session ID
                if let Event::SessionStarted { session_id, .. } = event {
                    current_session_id = Some(*session_id);
                }

                if let Err(e) = send_event(&mut sender, event).await {
                    tracing::error!("Failed to send event: {}", e);
                    break;
                }
            }
        } else if let Message::Close(_) = msg {
            tracing::info!("WebSocket closed");
            break;
        }
    }

    // Clean up any remaining sessions
    sessions.write().await.clear();
}

/// Handle a single command and return response events
async fn handle_command(
    command: Command,
    user_id: &str,
    app_state: &AppState,
    sessions: SessionMap,
    current_session_id: Option<Uuid>,
) -> Vec<Event> {
    match command {
        Command::StartExercise {
            exercise_type,
            node_id,
            axis: _, // Note: axis parameter not used here, but available for future enhancements
        } => handle_start_exercise(user_id, exercise_type, node_id, app_state, sessions).await,
        Command::SubmitAnswer { session_id, answer } => {
            let sid = session_id.or(current_session_id);
            handle_submit_answer(sid, answer, sessions).await
        }
        Command::UpdateMemorizationWord {
            session_id,
            word_node_id,
            action,
        } => {
            let sid = session_id.or(current_session_id);
            handle_update_memorization_word(sid, word_node_id, action, app_state, sessions).await
        }
        Command::StartEchoRecall { ayah_node_ids } => {
            handle_start_echo_recall(user_id, ayah_node_ids, app_state, sessions).await
        }
        Command::SubmitEchoRecall {
            session_id,
            word_node_id,
            recall_time_ms,
        } => {
            handle_submit_echo_recall(
                session_id,
                word_node_id,
                recall_time_ms,
                user_id,
                app_state,
                sessions,
            )
            .await
        }
        Command::EndSession { session_id } => {
            let sid = session_id.or(current_session_id);
            handle_end_session(sid, user_id, app_state, sessions).await
        }
        Command::GetDueItems {
            limit,
            axis,
            is_high_yield_mode,
        } => handle_get_due_items(user_id, limit, axis, is_high_yield_mode, app_state).await,
        Command::GenerateExercise {
            node_id,
            axis,
            format,
        } => handle_generate_exercise(&node_id, axis, format, app_state).await,
        Command::CheckAnswer { node_id, answer } => {
            handle_check_answer(&node_id, &answer, app_state).await
        }
    }
}

/// Start a new exercise session
async fn handle_start_exercise(
    user_id: &str,
    exercise_type: String,
    node_id: String,
    app_state: &AppState,
    sessions: SessionMap,
) -> Vec<Event> {
    let session_id = Uuid::new_v4();

    // Get the node to ensure it exists
    // Get the node to ensure it exists
    let nid_val = match nid::from_ukey(&node_id) {
        Some(id) => id,
        None => {
            return vec![Event::Error {
                message: format!("Invalid node ID format: {}", node_id),
            }];
        }
    };

    let _node = match app_state.content_repo.get_node(nid_val).await {
        Ok(Some(node)) => node,
        Ok(None) => {
            return vec![Event::Error {
                message: format!("Node not found: {}", node_id),
            }];
        }
        Err(e) => {
            return vec![Event::Error {
                message: format!("Failed to get node: {}", e),
            }];
        }
    };

    // Initialize exercise-specific state
    let initial_state = if exercise_type == "MemorizationAyah" {
        // For Memorization mode, we need to get all words in the verse
        match initialize_memorization_state(user_id, &node_id, app_state).await {
            Ok(state) => state,
            Err(e) => {
                return vec![Event::Error {
                    message: format!("Failed to initialize memorization state: {}", e),
                }];
            }
        }
    } else {
        // Generic exercise state
        json!({
            "exercise_type": exercise_type,
            "node_id": node_id,
        })
    };

    // Create the session
    let session = ExerciseSession {
        session_id,
        exercise_type: exercise_type.clone(),
        node_id: node_id.clone(),
        user_id: user_id.to_string(),
        state: initial_state.clone(),
    };

    // Store the session
    sessions.write().await.insert(session_id, session);

    vec![Event::SessionStarted {
        session_id,
        initial_state,
    }]
}

/// Initialize state for Memorization mode
async fn initialize_memorization_state(
    user_id: &str,
    verse_node_id: &str,
    app_state: &AppState,
) -> anyhow::Result<serde_json::Value> {
    use iqrah_core::MemorizationAyahExercise;

    // Create exercise using iqrah-core
    let exercise = MemorizationAyahExercise::new(
        user_id,
        verse_node_id,
        app_state.content_repo.as_ref(),
        app_state.user_repo.as_ref(),
    )
    .await?;

    // Return the state as JSON
    Ok(exercise.state().to_json())
}

/// Submit an answer for a generic exercise
async fn handle_submit_answer(
    session_id: Option<Uuid>,
    _answer: serde_json::Value,
    sessions: SessionMap,
) -> Vec<Event> {
    let sid = match session_id {
        Some(id) => id,
        None => {
            return vec![Event::Error {
                message: "No session ID provided and no active session".to_string(),
            }];
        }
    };

    let sessions_lock = sessions.read().await;
    let _session = match sessions_lock.get(&sid) {
        Some(s) => s,
        None => {
            return vec![Event::Error {
                message: format!("Session not found: {}", sid),
            }];
        }
    };

    // Placeholder for generic answer submission
    // In a real implementation, this would evaluate the answer
    vec![Event::Feedback {
        is_correct: true,
        correct_answer: None,
    }]
}

/// Update a word's state in Memorization mode (MVP)
async fn handle_update_memorization_word(
    session_id: Option<Uuid>,
    word_node_id: String,
    action: String,
    _app_state: &AppState,
    sessions: SessionMap,
) -> Vec<Event> {
    let sid = match session_id {
        Some(id) => id,
        None => {
            return vec![Event::Error {
                message: "No session ID provided and no active session".to_string(),
            }];
        }
    };

    let mut sessions_lock = sessions.write().await;
    let session = match sessions_lock.get_mut(&sid) {
        Some(s) => s,
        None => {
            return vec![Event::Error {
                message: format!("Session not found: {}", sid),
            }];
        }
    };

    // Ensure this is a Memorization exercise
    if session.exercise_type != "MemorizationAyah" {
        return vec![Event::Error {
            message: "This command is only valid for Memorization exercises".to_string(),
        }];
    }

    // MVP: Simple fixed energy increase per tap
    let energy_delta = match action.as_str() {
        "Tap" => 0.05,
        "LongPress" => 0.10,
        _ => {
            return vec![Event::Error {
                message: format!("Unknown action: {}", action),
            }];
        }
    };

    // Update the word's energy in the session state
    if let Some(words) = session.state["words"].as_array_mut() {
        for word in words.iter_mut() {
            if word["node_id"] == word_node_id {
                let current_energy = word["energy"].as_f64().unwrap_or(0.0);
                let new_energy = (current_energy + energy_delta).min(1.0); // Cap at 1.0
                word["energy"] = json!(new_energy);

                return vec![Event::StateUpdated {
                    new_state: session.state.clone(),
                }];
            }
        }
    }

    vec![Event::Error {
        message: format!("Word not found in session: {}", word_node_id),
    }]
}

/// End a session and save state to database
async fn handle_end_session(
    session_id: Option<Uuid>,
    user_id: &str,
    app_state: &AppState,
    sessions: SessionMap,
) -> Vec<Event> {
    let sid = match session_id {
        Some(id) => id,
        None => {
            return vec![Event::Error {
                message: "No session ID provided and no active session".to_string(),
            }];
        }
    };

    let session = {
        let mut sessions_lock = sessions.write().await;
        match sessions_lock.remove(&sid) {
            Some(s) => s,
            None => {
                return vec![Event::Error {
                    message: format!("Session not found: {}", sid),
                }];
            }
        }
    };

    // Save state to database based on exercise type
    if session.exercise_type == "MemorizationAyah" || session.exercise_type == "EchoRecall" {
        // Save word energies to the database
        if let Some(words) = session.state["words"].as_array() {
            for word in words {
                let node_id_str = word["node_id"].as_str().unwrap_or("");
                let energy = word["energy"].as_f64().unwrap_or(0.0);

                if let Some(nid_val) = nid::from_ukey(node_id_str) {
                    // Update energy in the database
                    if let Err(e) = app_state
                        .user_repo
                        .update_energy(user_id, nid_val, energy)
                        .await
                    {
                        tracing::error!("Failed to update energy for {}: {}", node_id_str, e);
                        // Don't return error, just log it and continue
                    }
                }
            }
        }
    }

    // Generate summary
    let summary = json!({
        "session_id": session_id,
        "exercise_type": session.exercise_type,
        "node_id": session.node_id,
    });

    vec![Event::SessionFinished {
        final_state: session.state,
        summary,
    }]
}

/// Start an Echo Recall session
async fn handle_start_echo_recall(
    user_id: &str,
    ayah_node_ids: Vec<String>,
    app_state: &AppState,
    sessions: SessionMap,
) -> Vec<Event> {
    use iqrah_core::EchoRecallExercise;

    let session_id = Uuid::new_v4();

    // Create the exercise using iqrah-core
    let exercise = match EchoRecallExercise::new(
        user_id,
        ayah_node_ids.clone(),
        app_state.content_repo.as_ref(),
        app_state.user_repo.as_ref(),
    )
    .await
    {
        Ok(e) => e,
        Err(e) => {
            return vec![Event::Error {
                message: format!("Failed to create Echo Recall exercise: {}", e),
            }];
        }
    };

    // Serialize the state
    let initial_state = match serde_json::to_value(exercise.state()) {
        Ok(v) => v,
        Err(e) => {
            return vec![Event::Error {
                message: format!("Failed to serialize state: {}", e),
            }];
        }
    };

    // Create and store session
    let session = ExerciseSession {
        session_id,
        exercise_type: "EchoRecall".to_string(),
        node_id: ayah_node_ids.join(","),
        user_id: user_id.to_string(),
        state: initial_state.clone(),
    };

    sessions.write().await.insert(session_id, session);

    vec![Event::SessionStarted {
        session_id,
        initial_state,
    }]
}

/// Submit a word recall in Echo Recall mode
async fn handle_submit_echo_recall(
    session_id: Uuid,
    word_node_id: String,
    recall_time_ms: u32,
    _user_id: &str,
    _app_state: &AppState,
    sessions: SessionMap,
) -> Vec<Event> {
    use iqrah_core::{EchoRecallExercise, EchoRecallState};

    let mut sessions_lock = sessions.write().await;
    let session = match sessions_lock.get_mut(&session_id) {
        Some(s) => s,
        None => {
            return vec![Event::Error {
                message: format!("Session not found: {}", session_id),
            }];
        }
    };

    // Ensure this is an Echo Recall exercise
    if session.exercise_type != "EchoRecall" {
        return vec![Event::Error {
            message: "This command is only valid for Echo Recall exercises".to_string(),
        }];
    }

    // Deserialize and create exercise from state
    let state: EchoRecallState = match serde_json::from_value(session.state.clone()) {
        Ok(s) => s,
        Err(e) => {
            return vec![Event::Error {
                message: format!("Failed to deserialize state: {}", e),
            }];
        }
    };

    // Get ayah IDs from session node_id (they were joined with ",")
    let ayah_node_ids: Vec<String> = session.node_id.split(',').map(|s| s.to_string()).collect();
    let mut exercise = EchoRecallExercise::from_state(&session.user_id, ayah_node_ids, state);

    // Submit the recall using the core exercise logic
    if let Err(e) = exercise.submit_recall(&word_node_id, recall_time_ms) {
        return vec![Event::Error {
            message: e.to_string(),
        }];
    }

    // Serialize updated state
    let new_state = match serde_json::to_value(exercise.state()) {
        Ok(v) => v,
        Err(e) => {
            return vec![Event::Error {
                message: format!("Failed to serialize state: {}", e),
            }];
        }
    };

    // Update session state
    session.state = new_state.clone();

    vec![Event::StateUpdated { new_state }]
}

/// Get due items for a session (Phase 4)
async fn handle_get_due_items(
    user_id: &str,
    limit: u32,
    axis: Option<String>,
    is_high_yield_mode: bool,
    app_state: &AppState,
) -> Vec<Event> {
    use iqrah_core::KnowledgeAxis;

    // Parse axis if provided
    let axis_filter = axis.and_then(|a| KnowledgeAxis::parse(&a).ok());

    // Get due items from session service
    let items = match app_state
        .session_service
        .get_due_items(
            user_id,
            chrono::Utc::now(),
            limit,
            is_high_yield_mode,
            axis_filter,
        )
        .await
    {
        Ok(items) => items,
        Err(e) => {
            return vec![Event::Error {
                message: format!("Failed to get due items: {}", e),
            }];
        }
    };

    // Serialize items to JSON
    let serialized_items: Vec<serde_json::Value> = items
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "node_id": item.node.id,
                "node_type": item.node.node_type,
                "knowledge_axis": item.knowledge_axis.map(|a| a.as_ref().to_string()),
                "priority_score": item.priority_score,
                "days_overdue": item.days_overdue,
                "mastery_gap": item.mastery_gap,
                "energy": item.memory_state.energy,
                "stability": item.memory_state.stability,
                "difficulty": item.memory_state.difficulty,
            })
        })
        .collect();

    vec![Event::DueItems {
        items: serialized_items,
    }]
}

/// Generate an exercise for a node (Phase 4.3)
async fn handle_generate_exercise(
    node_id: &str,
    axis: Option<String>,
    format: Option<String>,
    app_state: &AppState,
) -> Vec<Event> {
    use iqrah_core::{KnowledgeAxis, McqExercise};

    // Generate exercise based on format, axis, or auto-detect
    // Generate exercise based on format, axis, or auto-detect
    let nid_val = match nid::from_ukey(node_id) {
        Some(id) => id,
        None => {
            return vec![Event::Error {
                message: format!("Invalid node ID: {}", node_id),
            }]
        }
    };

    let exercise_result = if let Some(fmt) = format {
        // Generate based on explicit format
        match fmt.as_str() {
            "mcq_ar_to_en" => {
                app_state
                    .exercise_service
                    .generate_mcq_ar_to_en(nid_val, node_id)
                    .await
            }
            "mcq_en_to_ar" => {
                app_state
                    .exercise_service
                    .generate_mcq_en_to_ar(nid_val, node_id)
                    .await
            }
            _ => {
                return vec![Event::Error {
                    message: format!("Invalid format: {}", fmt),
                }];
            }
        }
    } else if let Some(axis_str) = axis {
        // Parse axis and generate for specific axis
        if let Ok(axis_enum) = KnowledgeAxis::parse(&axis_str) {
            app_state
                .exercise_service
                .generate_exercise_for_axis(nid_val, node_id, axis_enum)
                .await
        } else {
            return vec![Event::Error {
                message: format!("Invalid axis: {}", axis_str),
            }];
        }
    } else {
        // Auto-detect axis from node ID
        app_state
            .exercise_service
            .generate_exercise(nid_val, node_id)
            .await
    };

    match exercise_result {
        Ok(exercise_type) => {
            let exercise = exercise_type.as_exercise();

            // Try to get MCQ options if it's an MCQ exercise
            let options = (exercise as &dyn std::any::Any)
                .downcast_ref::<McqExercise>()
                .map(|mcq| mcq.get_options().to_vec());

            vec![Event::ExerciseGenerated {
                node_id: node_id.to_string(),
                exercise_type: exercise.get_type_name().to_string(),
                question: exercise.generate_question(),
                hint: exercise.get_hint(),
                options,
            }]
        }
        Err(e) => vec![Event::Error {
            message: format!("Failed to generate exercise: {}", e),
        }],
    }
}

/// Check answer for an exercise (Phase 4.3)
async fn handle_check_answer(node_id: &str, answer: &str, app_state: &AppState) -> Vec<Event> {
    // Generate exercise first (we need it to check the answer)
    let nid_val = match nid::from_ukey(node_id) {
        Some(id) => id,
        None => {
            return vec![Event::Error {
                message: format!("Invalid node ID: {}", node_id),
            }]
        }
    };
    let exercise_result = app_state
        .exercise_service
        .generate_exercise(nid_val, node_id)
        .await;

    match exercise_result {
        Ok(exercise_type) => {
            let exercise = exercise_type.as_exercise();
            let response = app_state.exercise_service.check_answer(exercise, answer);

            vec![Event::AnswerChecked {
                is_correct: response.is_correct,
                hint: response.hint,
                correct_answer: response.correct_answer,
                options: response.options,
                semantic_grade: response.semantic_grade,
                similarity_score: response.similarity_score,
            }]
        }
        Err(e) => vec![Event::Error {
            message: format!("Failed to check answer: {}", e),
        }],
    }
}

/// Helper to send an event to the client
async fn send_event(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    event: &Event,
) -> Result<(), axum::Error> {
    let json = serde_json::to_string(event).unwrap();
    tracing::debug!("Sending event: {}", json);
    sender.send(Message::Text(json)).await
}
