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

use crate::{protocol::{Command, Event}, AppState};

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
        } => {
            handle_start_exercise(user_id, exercise_type, node_id, app_state, sessions).await
        }
        Command::SubmitAnswer {
            session_id,
            answer,
        } => {
            let sid = session_id.or(current_session_id);
            handle_submit_answer(sid, answer, sessions).await
        }
        Command::UpdateMemorizationWord {
            session_id,
            word_node_id,
            action,
        } => {
            let sid = session_id.or(current_session_id);
            handle_update_memorization_word(
                sid,
                word_node_id,
                action,
                app_state,
                sessions,
            )
            .await
        }
        Command::EndSession { session_id } => {
            let sid = session_id.or(current_session_id);
            handle_end_session(sid, user_id, app_state, sessions).await
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
    let _node = match app_state.content_repo.get_node(&node_id).await {
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
        match initialize_memorization_state(&node_id, app_state).await {
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
    verse_node_id: &str,
    app_state: &AppState,
) -> anyhow::Result<serde_json::Value> {
    // Get all word children of this verse
    let edges = app_state.content_repo.get_edges_from(verse_node_id).await?;

    // Get the word nodes and their current energies
    let mut words = Vec::new();
    for edge in edges {
        if edge.target_id.starts_with("WORD:") {
            let _word_node = app_state
                .content_repo
                .get_node(&edge.target_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Word node not found: {}", edge.target_id))?;

            let word_text = app_state
                .content_repo
                .get_quran_text(&edge.target_id)
                .await?
                .unwrap_or_default();

            // Get current energy from user state (default to 0.0)
            let memory_state = app_state
                .user_repo
                .get_memory_state("test_user", &edge.target_id)
                .await?;
            let energy = memory_state.map(|s| s.energy).unwrap_or(0.0);

            words.push(json!({
                "node_id": edge.target_id,
                "text": word_text,
                "energy": energy,
            }));
        }
    }

    Ok(json!({
        "verse_node_id": verse_node_id,
        "words": words,
    }))
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
    if session.exercise_type == "MemorizationAyah" {
        // Save word energies to the database
        if let Some(words) = session.state["words"].as_array() {
            for word in words {
                let node_id = word["node_id"].as_str().unwrap_or("");
                let energy = word["energy"].as_f64().unwrap_or(0.0);

                // Update energy in the database
                if let Err(e) = app_state
                    .user_repo
                    .update_energy(user_id, node_id, energy)
                    .await
                {
                    tracing::error!("Failed to update energy for {}: {}", node_id, e);
                    return vec![Event::Error {
                        message: format!("Failed to save state: {}", e),
                    }];
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

/// Helper to send an event to the client
async fn send_event(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    event: &Event,
) -> Result<(), axum::Error> {
    let json = serde_json::to_string(event).unwrap();
    tracing::debug!("Sending event: {}", json);
    sender.send(Message::Text(json)).await
}
