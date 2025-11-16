use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

/// Run an interactive exercise session
pub async fn run(server_url: &str, exercise_type: &str, node_id: &str) -> Result<()> {
    // Convert HTTP URL to WebSocket URL
    let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");
    let url = Url::parse(&format!("{}/ws", ws_url))?;

    tracing::info!("Connecting to {}", url);
    let (ws_stream, _) = connect_async(url).await?;
    tracing::info!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // Start the exercise
    let start_command = json!({
        "type": "StartExercise",
        "exercise_type": exercise_type,
        "node_id": node_id,
    });

    write
        .send(Message::Text(start_command.to_string()))
        .await?;

    // Spawn a task to read from stdin and send commands
    let write_handle = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse the line as JSON
            let msg = if line.trim().starts_with('{') {
                // Raw JSON command
                line
            } else {
                // Simple text input - not implemented for now
                tracing::warn!("Expected JSON command, got: {}", line);
                continue;
            };

            if let Err(e) = write.send(Message::Text(msg)).await {
                tracing::error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Read responses from the server
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Pretty print the JSON response
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("{}", serde_json::to_string_pretty(&json)?);

                    // Check if session finished
                    if json.get("type").and_then(|t| t.as_str()) == Some("SessionFinished") {
                        tracing::info!("Session finished");
                        break;
                    }
                } else {
                    println!("{}", text);
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket closed by server");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Clean up the write task
    write_handle.abort();

    Ok(())
}

/// Start an Echo Recall exercise session
pub async fn start(server_url: &str, exercise_type: &str, ayah_node_ids: &[String]) -> Result<()> {
    // Convert HTTP URL to WebSocket URL
    let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");
    let url = Url::parse(&format!("{}/ws", ws_url))?;

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send StartEchoRecall command
    let command = json!({
        "type": "StartEchoRecall",
        "ayah_node_ids": ayah_node_ids,
    });

    write.send(Message::Text(command.to_string())).await?;

    // Wait for response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", text);
        }
    }

    Ok(())
}

/// Submit an action in an Echo Recall exercise
pub async fn action(
    server_url: &str,
    _exercise_type: &str,
    session_id: &str,
    word_node_id: &str,
    recall_time_ms: u32,
) -> Result<()> {
    // Convert HTTP URL to WebSocket URL
    let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");
    let url = Url::parse(&format!("{}/ws", ws_url))?;

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send SubmitEchoRecall command
    let command = json!({
        "type": "SubmitEchoRecall",
        "session_id": session_id,
        "word_node_id": word_node_id,
        "recall_time_ms": recall_time_ms,
    });

    write.send(Message::Text(command.to_string())).await?;

    // Wait for response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", text);
        }
    }

    Ok(())
}

/// End an exercise session
pub async fn end(server_url: &str, session_id: &str) -> Result<()> {
    // Convert HTTP URL to WebSocket URL
    let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");
    let url = Url::parse(&format!("{}/ws", ws_url))?;

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send EndSession command
    let command = json!({
        "type": "EndSession",
        "session_id": session_id,
    });

    write.send(Message::Text(command.to_string())).await?;

    // Wait for response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", text);
        }
    }

    Ok(())
}
